# spanda-sdk

Official Rust SDK for [Spanda](https://github.com/Davalgi/Spanda) Control Center API v1.

Thin HTTP client over REST `/v1/*` — no duplicated platform logic. Optional native gRPC via the `grpc` feature.

## Install

```toml
[dependencies]
spanda-sdk = "0.4"
```

Optional tonic client:

```toml
spanda-sdk = { version = "0.4", features = ["grpc"] }
```

## Quick start

```rust
use spanda_sdk::SpandaClient;

fn main() -> Result<(), spanda_sdk::SpandaError> {
    let client = SpandaClient::local();
    let report = client.readiness("rover.sd")?;
    println!("score = {:?}", report.score);
    Ok(())
}
```

Start Control Center first:

```bash
spanda control-center serve --program examples/robotics/rover.sd
```

## Documentation

- [docs/sdk-rust.md](https://github.com/Davalgi/Spanda/blob/main/docs/sdk-rust.md)
- [docs/control-center-api.md](https://github.com/Davalgi/Spanda/blob/main/docs/control-center-api.md)
- [docs/sdk-publishing.md](https://github.com/Davalgi/Spanda/blob/main/docs/sdk-publishing.md)

## License

Same as the Spanda workspace (see repository root `LICENSE`).
