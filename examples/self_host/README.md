# Self-hosting bootstrap (Phase 22)

Experimental first step toward a Spanda-authored compiler: a minimal word tokenizer written in Spanda.

This does **not** replace the Rust lexer. It demonstrates that string processing and module exports work for bootstrap experiments listed in `docs/roadmap.md` (self-hosting milestone 3).

## Run

```bash
spanda check examples/self_host/word_tokenizer.sd
spanda run examples/self_host/word_tokenizer.sd
```

## Next milestones

1. Rust bootstrap (current) — complete
2. Spec stabilization — `docs/api-contract.json`
3. Spanda subset lexer — this example
4. Parser for minimal `.sd` subset
5. Full self-hosted compiler

See [compiler-backend-roadmap.md](../docs/compiler-backend-roadmap.md) and [tier-3-experimental.md](../docs/tier-3-experimental.md).
