# spanda-gps

Official Spanda package: **GPS/GNSS receiver adapters**

## Import

```spanda
import positioning.gps;
```

## Spoofing backend

Exports `spoofing_backend_name()` and `plausibility_contract_version()` for integration with `spanda spoof-check` and `spanda-spoofing` trace heuristics. Vendor-specific ML backends can replace the contract in future package versions.

## Capabilities

This package requires runtime capabilities declared in `spanda.toml`.

## Status

Scaffold package — implements the lean-core provider contract surface.
Core retains compatibility shims until callers migrate to this package.
