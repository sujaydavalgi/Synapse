# spanda-llvm

Experimental **SIR → LLVM IR** emitter and optional native compile via `clang` + `spanda-rt`.

## Dependencies

`spanda-sir`, `spanda-ast`, `spanda-driver` (no `spanda-core`).

## CLI

```bash
spanda ir program.sd
spanda llvm-ir program.sd
spanda compile-native program.sd --output bin/rover
```

## Related

- [docs/compiler-backend-roadmap.md](../../docs/compiler-backend-roadmap.md)
- [spanda-rt](../spanda-rt/README.md)
