/**
 * index module (security/index.ts).
 * @module
 */

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
  // TrustSatisfies.
  //
  // Parameters:
  // - `actual` — input value
  // - `required` — input value
  //
  // Returns:
  // `true` or `false`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = trustSatisfies(actual, required);
  return TRUST_RANK[actual] >= TRUST_RANK[required];
}

export function parseTrustLevel(level: string): TrustLevel | null {
  // ParseTrustLevel.
  //
  // Parameters:
  // - `level` — input value
  //
  // Returns:
  // `Some` / non-null value on success, otherwise `None` / null.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = parseTrustLevel(level);
  if (level in TRUST_RANK) return level as TrustLevel;
  return null;
}

function seedBytes(material: string): Uint8Array {
  // SeedBytes.
  //
  // Parameters:
  // - `material` — input value
  //
  // Returns:
  // `Uint8Array`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = seedBytes(material);
  return sha256(new TextEncoder().encode(material));
}

export function isHexPublicKey(key: string): boolean {
  // IsHexPublicKey.
  //
  // Parameters:
  // - `key` — input value
  //
  // Returns:
  // `true` or `false`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = isHexPublicKey(key);
  return key.length === 64 && /^[0-9a-fA-F]+$/.test(key);
}

export function publicKeyFromMaterial(material: string): string {
  // PublicKeyFromMaterial.
  //
  // Parameters:
  // - `material` — input value
  //
  // Returns:
  // Text result.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = publicKeyFromMaterial(material);
  const priv = seedBytes(material);
  return bytesToHex(ed.getPublicKey(priv));
}

export function sha256Hex(data: string): string {
  // Sha256Hex.
  //
  // Parameters:
  // - `data` — input value
  //
  // Returns:
  // Text result.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = sha256Hex(data);
  return bytesToHex(sha256(new TextEncoder().encode(data)));
}

export async function signAsync(data: string, keyMaterial: string): Promise<string> {
  // SignAsync.
  //
  // Parameters:
  // - `data` — input value
  // - `keyMaterial` — input value
  //
  // Returns:
  // Success value on completion, or an error.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = signAsync(data, keyMaterial);
  const priv = keyMaterial.length === 64 && isHexPublicKey(keyMaterial)
    ? hexToBytes(keyMaterial)
    : seedBytes(keyMaterial);
  const sig = await ed.signAsync(new TextEncoder().encode(data), priv);
  return bytesToHex(sig);
}

export function sign(data: string, keyMaterial: string): string {
  // Sign.
  //
  // Parameters:
  // - `data` — input value
  // - `keyMaterial` — input value
  //
  // Returns:
  // Text result.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = sign(data, keyMaterial);
  const priv = keyMaterial.length === 64 && isHexPublicKey(keyMaterial)
    ? hexToBytes(keyMaterial)
    : seedBytes(keyMaterial);
  return bytesToHex(ed.sign(new TextEncoder().encode(data), priv));
}

export function verifySignature(data: string, signature: string, key: string): boolean {
  // VerifySignature.
  //
  // Parameters:
  // - `data` — input value
  // - `signature` — input value
  // - `key` — input value
  //
  // Returns:
  // `true` or `false`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = verifySignature(data, signature, key);
  try {
    const sig = hexToBytes(signature);

    // continue when length differs from 64.
    if (sig.length !== 64) return false;
    const msg = new TextEncoder().encode(data);

    // continue when isHexPublicKey(key).
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
  // CreateRobotIdentity.
  //
  // Parameters:
  // - `id` — input value
  // - `publicKey` — input value
  // - `trust` — optional input
  //
  // Returns:
  // `RobotIdentity`.
  //
  // Options:
  // - `trust` — optional parameter
  //
  // Example:

  // const result = createRobotIdentity(id, publicKey, trust);
  return {
    id,
    publicKey,
    trust,
    signingMaterial() {
      //
      // Parameters:
      // None.
      //
      // Returns:
      //
      // Options:
      // None.
      //
      // Example:

      // continue when publicKey || isHexPublicKey is falsy.
      if (!publicKey || isHexPublicKey(publicKey)) return `spanda-device-${id}`;
      return publicKey;
    },
    verifyingKeyHex() {
      //
      // Parameters:
      // None.
      //
      // Returns:
      //
      // Options:
      // None.
      //
      // Example:

      // continue when isHexPublicKey(publicKey).
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
    // Grant.
    //
    // Parameters:
    // - `cap` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = grant(cap);

    this.granted.add(cap);
  }

  grantAll(caps: string[]): void {
    // GrantAll.
    //
    // Parameters:
    // - `caps` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = grantAll(caps);

    for (const c of caps) this.grant(c);
  }

  has(cap: string): boolean {
    // Has.
    //
    // Parameters:
    // - `cap` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = has(cap);

    return this.permissive || this.granted.has(cap);
  }

  require(cap: string): void {
    // Require.
    //
    // Parameters:
    // - `cap` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = require(cap);

    if (!this.has(cap)) throw new Error(`capability denied: ${cap}`);
  }
}

export type SecretSource =
  | { source: "env"; var: string }
  | { source: "literal"; value: string };

export class SecretStore {
  private secrets = new Map<string, SecretSource>();

  register(name: string, source: SecretSource): void {
    // Register the value.
    //
    // Parameters:
    // - `name` — input value
    // - `source` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = register(name, source);

    this.secrets.set(name, source);
  }

  resolve(name: string): string {
    // Resolve.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = resolve(name);

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
    // Register the value.
    //
    // Parameters:
    // - `path` — input value
    // - `policy` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = register(path, policy);

    this.policies.set(path, policy);
  }

  policyOrOpen(path: string): SecurePolicy {
    // PolicyOrOpen.
    //
    // Parameters:
    // - `path` — input value
    //
    // Returns:
    // SecurePolicy.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = policyOrOpen(path);

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
    // EnableStrictPermissions.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = enableStrictPermissions();

    this.strictPermissions = true;
  }

  grantIfNotStrict(cap: string): void {
    // GrantIfNotStrict.
    //
    // Parameters:
    // - `cap` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = grantIfNotStrict(cap);

    if (!this.strictPermissions) this.capabilities.grant(cap);
  }

  requireOperation(operation: string): void {
    // RequireOperation.
    //
    // Parameters:
    // - `operation` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = requireOperation(operation);

    const map: Record<string, string> = {
      "audit.record": "audit.write",
      "audit.read": "audit.read",
      sign: "identity.sign",
      "identity.sign": "identity.sign",
      "identity.verify": "identity.verify",
      "ledger.anchor": "ledger.anchor",
      "cellular.sim_identity": "cellular.connect",
    };
    const cap = map[operation];
    if (cap) this.capabilities.require(cap);
  }

  signOutbound(path: string, payload: string): void {
    // SignOutbound.
    //
    // Parameters:
    // - `path` — input value
    // - `payload` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = signOutbound(path, payload);

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
  // IsKnownCapability.
  //
  // Parameters:
  // - `cap` — input value
  //
  // Returns:
  // `true` or `false`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = isKnownCapability(cap);
  return (KNOWN_CAPABILITIES as readonly string[]).includes(cap);
}
