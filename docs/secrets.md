# Secrets Management

Spanda loads secrets at runtime from environment variables or files. **Never hardcode secrets in source.**

## Declaration syntax

Individual secrets:

```spanda
secret signing_key from env "ROVER_PRIVATE_KEY";
secret tls_cert from file "certs/rover.pem";
```

Block form:

```spanda
secrets {
    rover_private_key from env "ROVER_PRIVATE_KEY";
    tls_cert from file "certs/rover.pem";
}
```

## Sources

| Source | Syntax | Use case |
|--------|--------|----------|
| Environment | `from env "VAR"` | CI, containers, local dev |
| File | `from file "path"` | TLS certs, mounted key volumes |
| Literal | `from "value"` | **Tests only** — not for production |

Future: secret-manager backends (Vault, cloud KMS) without changing declaration names.

## Capabilities

Reading secrets requires `secret.read` in robot `permissions` or package manifest capabilities.

## Runtime rules

- Secrets resolve through `SecretStore` — values are never printed in logs
- Audit events record `secret:<name>` redacted labels only
- Encrypted buses and topics require key or certificate configuration when `encryption: required`

## CLI validation

```bash
spanda security check robot.sd   # flags secrets without capability
spanda security audit robot.sd   # records secret access patterns
```

See [Secure communication](secure-communication.md).
