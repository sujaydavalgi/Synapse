# Identity

Spanda device and operator identity extends the audit `DeviceIdentity` model with trust metadata for secure communication.

## Robot identity

```spanda
identity RoverIdentity {
    id: "rover-001";
    public_key: "...";
    cert: "certs/rover-001.pem";
}
```

Required field: `id`. Optional: `public_key`, `cert`, `trust`.

## Operator / console identity

```spanda
identity OperatorConsole {
    id: "operator-console";
    trust: verified;
}
```

Trust tiers: `untrusted`, `restricted`, `trusted`, `certified`, `verified`.

## Runtime behavior

- Identity is required for signed or encrypted endpoints
- Signing uses `identity.sign` capability
- Verification uses `identity.verify` capability
- Certificates reference PEM paths — private keys never appear in source

## Audit integration

Identity feeds `spanda-audit` provenance and `security.*` audit events. Device IDs appear in signed envelopes; key material is never logged.

See also [Secure communication](secure-communication.md) and [Trust boundaries](trust-boundaries.md).
