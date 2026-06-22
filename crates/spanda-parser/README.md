# spanda-parser

Spanda **parser** — token stream → `spanda_ast::nodes::Program`.

Extracted from `spanda-core` in Phase 9 (~8k LOC). Used by `spanda-driver::compile`, the formatter, and tests.

## API

```rust
use spanda_lexer::tokenize;
use spanda_parser::parse;

let program = parse(tokenize(source)?)?;
```

Errors use `spanda_error::SpandaError`.

## Related

- [spanda-lexer](../spanda-lexer/README.md)
- [spanda-ast](../spanda-ast/README.md)
