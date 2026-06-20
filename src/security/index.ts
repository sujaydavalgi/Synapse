import { sha256 } from "@noble/hashes/sha256";
import { sha512 } from "@noble/hashes/sha512";
import * as ed from "@noble/ed25519";
import { bytesToHex, hexToBytes } from "@noble/hashes/utils";

ed.etc.sha512Sync = (...m: Uint8Array[]) => sha512(ed.etc.concatBytes(...m));

export type TrustLevel = "untrusted" | "restricted" | "trusted" | "certified";

const TRUST_RANK: Record<TrustLevel, number> = {
  untrusted: 0,
  restricted: 1,
  trusted: 2,
  certified: 3,
};

export function trustSatisfies(actual: TrustLevel, required: TrustLevel): boolean {
  return TRUST_RANK[actual] >= TRUST_RANK[required];
}

export function parseTrustLevel(level: string): TrustLevel | null {
  if (level in TRUST_RANK) return level as TrustLevel;
  return null;
}

function seedBytes(material: string): Uint8Array {
  return sha256(new TextEncoder().encode(material));
}

export function isHexPublicKey(key: string): boolean {
  return key.length === 64 && /^[0-9a-fA-F]+$/.test(key);
}

export function publicKeyFromMaterial(material: string): string {
  const priv = seedBytes(material);
  return bytesToHex(ed.getPublicKey(priv));
}

export function sha256Hex(data: string): string {
  return bytesToHex(sha256(new TextEncoder().encode(data)));
}

export async function signAsync(data: string, keyMaterial: string): Promise<string> {
  const priv = keyMaterial.length === 64 && isHexPublicKey(keyMaterial)
    ? hexToBytes(keyMaterial)
    : seedBytes(keyMaterial);
  const sig = await ed.signAsync(new TextEncoder().encode(data), priv);
  return bytesToHex(sig);
}

export function sign(data: string, keyMaterial: string): string {
  const priv = keyMaterial.length === 64 && isHexPublicKey(keyMaterial)
    ? hexToBytes(keyMaterial)
    : seedBytes(keyMaterial);
  return bytesToHex(ed.sign(new TextEncoder().encode(data), priv));
}

export function verifySignature(data: string, signature: string, key: string): boolean {
  try {
    const sig = hexToBytes(signature);
    if (sig.length !== 64) return false;
    const msg = new TextEncoder().encode(data);
    if (isHexPublicKey(key)) {
      return ed.verify(sig, msg, hexToBytes(key));
    }
    const priv = seedBytes(key);
    const pub = ed.getPublicKey(priv);
    return ed.verify(sig, msg, pub);
  } catch {
    return false;
  }
}

export type RobotIdentity = {
  id: string;
  publicKey: string;
  trust: TrustLevel;
  signingMaterial(): string;
  verifyingKeyHex(): string;
};

export function createRobotIdentity(
  id: string,
  publicKey: string,
  trust: TrustLevel = "trusted",
): RobotIdentity {
  return {
    id,
    publicKey,
    trust,
    signingMaterial() {
      if (!publicKey || isHexPublicKey(publicKey)) return `spanda-device-${id}`;
      return publicKey;
    },
    verifyingKeyHex() {
      if (isHexPublicKey(publicKey)) return publicKey;
      return publicKeyFromMaterial(this.signingMaterial());
    },
  };
}

export type SecurePolicy = {
  signed: boolean;
  minTrust: TrustLevel | null;
  requires: string[];
};

export class CapabilitySet {
  private granted = new Set<string>();
  private permissive = false;

  grant(cap: string): void {
    this.granted.add(cap);
  }

  grantAll(caps: string[]): void {
    for (const c of caps) this.grant(c);
  }

  has(cap: string): boolean {
    return this.permissive || this.granted.has(cap);
  }

  require(cap: string): void {
    if (!this.has(cap)) throw new Error(`capability denied: ${cap}`);
  }
}

export type SecretSource =
  | { source: "env"; var: string }
  | { source: "literal"; value: string };

export class SecretStore {
  private secrets = new Map<string, SecretSource>();

  register(name: string, source: SecretSource): void {
    this.secrets.set(name, source);
  }

  resolve(name: string): string {
    const src = this.secrets.get(name);
    if (!src) throw new Error(`secret not found: ${name}`);
    if (src.source === "literal") return src.value;
    const val = process.env[src.var];
    if (val === undefined) throw new Error(`environment variable '${src.var}' not set`);
    return val;
  }
}

export class SecureEndpointRegistry {
  private policies = new Map<string, SecurePolicy>();

  register(path: string, policy: SecurePolicy): void {
    this.policies.set(path, policy);
  }

  policyOrOpen(path: string): SecurePolicy {
    return this.policies.get(path) ?? { signed: false, minTrust: null, requires: [] };
  }
}

export class SecurityContext {
  identity: RobotIdentity | null = null;
  trust: TrustLevel = "trusted";
  secrets = new SecretStore();
  capabilities = new CapabilitySet();
  secureEndpoints = new SecureEndpointRegistry();
  strictPermissions = false;

  enableStrictPermissions(): void {
    this.strictPermissions = true;
  }

  grantIfNotStrict(cap: string): void {
    if (!this.strictPermissions) this.capabilities.grant(cap);
  }

  requireOperation(operation: string): void {
    const map: Record<string, string> = {
      "audit.record": "audit.write",
      "audit.read": "audit.read",
      sign: "identity.sign",
      "identity.sign": "identity.sign",
      "identity.verify": "identity.verify",
      "ledger.anchor": "ledger.anchor",
    };
    const cap = map[operation];
    if (cap) this.capabilities.require(cap);
  }

  signOutbound(path: string, payload: string): void {
    const policy = this.secureEndpoints.policyOrOpen(path);
    for (const cap of policy.requires) this.capabilities.require(cap);
    if (policy.minTrust && !trustSatisfies(this.trust, policy.minTrust)) {
      throw new Error(`trust level insufficient: required ${policy.minTrust}, have ${this.trust}`);
    }
    if (policy.signed || policy.minTrust || policy.requires.length > 0) {
      if (!this.identity) throw new Error(`identity required for ${path}`);
      if (policy.signed) this.capabilities.require("identity.sign");
    }
  }
}

export const KNOWN_CAPABILITIES = [
  "audit.write",
  "audit.read",
  "identity.sign",
  "identity.verify",
  "ledger.anchor",
  "network.outbound",
  "actuator.execute",
] as const;

export function isKnownCapability(cap: string): boolean {
  return (KNOWN_CAPABILITIES as readonly string[]).includes(cap);
}
