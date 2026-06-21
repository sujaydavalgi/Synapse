# Trust Boundaries

Trust boundaries declare where Spanda applies stricter communication rules.

## Declarations

```spanda
trust_boundary robot_internal;
trust_boundary robot_to_robot;
trust_boundary robot_to_cloud;
trust_boundary operator_to_robot;
```

## Default rules

| Boundary | Encryption | Authentication | Actuator notes |
|----------|------------|----------------|----------------|
| `robot_internal` | Optional | None | In-process / same robot |
| `robot_to_robot` | **Required** | Signed recommended | Fleet mesh, DDS peers |
| `robot_to_cloud` | **Required** | Signed | Telemetry uplink |
| `operator_to_robot` | **Required** | **Mutual** | Human command paths |

## SafeAction over network

`SafeAction` messages crossing `robot_to_robot`, `robot_to_cloud`, or `operator_to_robot` must:

1. Use `encryption required` on the topic/service/action
2. Use `signed required` (integrity)
3. Pass `trusted_sources` / `reject_untrusted` when actuators are involved
4. Resolve to `VerifiedMessage<SafeAction>` at runtime trust gates

The compiler reports violations via `spanda security check`.

## Simulation

Use `simulate_compatibility` with security faults to test boundary behavior:

```spanda
simulate_compatibility {
    fault InvalidSignature at T+10s;
    fault ExpiredCertificate at T+20s;
    fault ReplayAttack at T+30s;
}
```

Or inject defaults: `spanda sim robot.sd --inject-security-faults`

See [Secure communication](secure-communication.md).
