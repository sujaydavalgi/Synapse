//! Rust lexer parity checks for the self-host bootstrap milestone.

use spanda_lexer::{tokenize, TokenType};

#[test]
fn self_host_bootstrap_sample_tokenizes() {
    let tokens = tokenize("robot Rover { }").expect("tokenize bootstrap sample");
    assert!(
        tokens
            .iter()
            .any(|token| token.token_type == TokenType::Robot),
        "expected Robot keyword token"
    );
    assert!(
        tokens.iter().any(|token| token.lexeme == "Rover"),
        "expected Rover identifier token"
    );
    assert!(
        tokens
            .iter()
            .any(|token| token.token_type == TokenType::Lbrace),
        "expected opening brace"
    );
}

#[test]
fn self_host_world_model_keyword_tokenizes() {
    let tokens = tokenize("world_model { enabled; }").expect("tokenize world_model block");
    assert!(
        tokens.iter().any(|token| token.lexeme == "world_model"),
        "expected world_model identifier"
    );
    assert!(
        tokens.iter().any(|token| token.lexeme == "enabled"),
        "expected enabled flag"
    );
}
