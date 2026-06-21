# Secure Communication

Spanda extends its existing identity, capability, audit, and signed-topic security model with **optional encrypted communication** across buses, topics, services, and actions.

## Philosophy

| Boundary | Encryption | Authentication |
|----------|------------|----------------|
| In-process simulation | Optional | None |
| Inter-process (local) | Recommended | Signed (optional) |
| Inter-device / robot-to-robot | **Required** | Signed |
| Robot-to-cloud | **Required** | Mutual |
| Operator-to-robot | **Required** | **Mutual** |
| Actuator commands over network | Encrypted + signed + verified | Mutual |

Encryption is optional for internal simulation and development. Deployments crossing trust boundaries should declare explicit policies.

Publish-time **trusted-source enforcement** runs when `trusted_sources` is set on a secure topic. Inbound **receive** and transport poll paths verify trusted sources when publisher identity is available on the message envelope.

Mock TLS frames protect in-process transport simulation (`RoutingCommBus` encrypts/decrypts adapter payloads when bus encryption is enabled). Production deployments should wire real TLS, DDS-Security, or MQTT TLS adapters.

## Robot-wide policy

```spanda
secure_comm {
    encryption: required;
    authentication: mutual;
    integrity: required;
}
```

Modes:

- `encryption`: `none` | `optional` | `required`
- `authentication`: `none` | `signed` | `mutual`
- `integrity`: `none` | `required`

## Secure buses

```spanda
bus robot_mesh {
    transport: "dds";
    encryption: required;
    authentication: mutual;
}
```

Legacy shorthand remains supported: `bus ros2;`

## Secure topics, services, and actions

```spanda
topic lidar_scan: Topic<LidarScan> {
    secure {
        encryption required;
        signed required;
        trusted_sources [LidarFront];
    }
}

service GetBattery: Service<BatteryRequest, BatteryStatus> {
    secure {
        encryption required;
        authentication mutual;
    }
}
```

Signed messages protect integrity. Mutual authentication protects identity on operator and cloud links.

## Message types

- `EncryptedMessage<T>` — payload inaccessible until decrypted
- `SignedMessage<T>` — must be verified before trusted use
- `VerifiedMessage<T>` — signature-checked envelope for actuator paths
- `TrustedSource`, `Certificate`, `PublicKey`, `PrivateKey`, `SessionKey`

Actuator commands crossing trust boundaries must use `VerifiedMessage<SafeAction>`.

## Capabilities

Crypto and secure comm operations require declared capabilities:

- `crypto.encrypt`, `crypto.decrypt`, `crypto.sign`, `crypto.verify`
- `identity.read`, `secret.read`
- `secure_topic.publish`, `secure_topic.subscribe`

## CLI

```bash
spanda security check robot.sd
spanda security audit robot.sd
spanda run robot.sd --secure
spanda sim robot.sd --inject-security-faults
```

## Transport adapters

Spanda transport uses a versioned **wire frame** (`TransportWireFrame` v1) with JSON payload, optional `source_id`, and AES-256-GCM encryption when bus policy requires it. Frames on the wire are prefixed with `spanda/wire/v1:` followed by hex-encoded ciphertext.

Broker URLs using TLS schemes (`mqtts://`, `wss://`, `dds+sec://`) automatically upgrade encryption to `required`. Adapters validate that a negotiated TLS session exists before publishing.

Supported transports: `local`, `ros2`, `dds`, `mqtt`, `websocket`, `ble`, `wifi`, and `cellular`. Live broker integrations use the same wire frame and session material derived from configured cert/key secrets.

## Examples

See `examples/security/` for encrypted topics, robot-to-robot mesh, operator commands, cloud links, and fault injection scenarios.

## Related docs

- [Identity and trust](identity.md)
- [Secrets management](secrets.md)
- [Trust boundaries](trust-boundaries.md)
- [Security foundation](../security.md)
