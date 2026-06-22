//! Compiler and runtime error types shared across Spanda crates.
//!
pub use spanda_typecheck::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpandaError {
    #[error("{message} (line {line}, col {column})")]
    Lexer {
        message: String,
        line: u32,
        column: u32,
    },
    #[error("{message} (line {line}, col {column})")]
    Parse {
        message: String,
        line: u32,
        column: u32,
    },
    #[error("Type check failed")]
    TypeCheck { diagnostics: Vec<Diagnostic> },
    #[error("{message} (line {line})")]
    Runtime { message: String, line: u32 },
    #[error("Debug pause at line {line}: {reason}")]
    DebugPause { line: u32, reason: String },
}

impl From<spanda_runtime::RuntimeError> for SpandaError {
    fn from(err: spanda_runtime::RuntimeError) -> Self {
        Self::Runtime {
            message: err.message,
            line: err.line,
        }
    }
}

impl SpandaError {
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        // Collect human-readable diagnostics for this error.
        //
        // Parameters:
        // - `self` — error value
        //
        // Returns:
        // One or more source diagnostics.
        //
        // Options:
        // None.
        //
        // Example:
        // let lines = err.diagnostics();

        match self {
            SpandaError::Lexer {
                message,
                line,
                column,
            } => vec![Diagnostic {
                message: message.clone(),
                line: *line,
                column: *column,
            }],
            SpandaError::Parse {
                message,
                line,
                column,
            } => vec![Diagnostic {
                message: message.clone(),
                line: *line,
                column: *column,
            }],
            SpandaError::TypeCheck { diagnostics } => diagnostics.clone(),
            SpandaError::Runtime { message, line } => vec![Diagnostic {
                message: message.clone(),
                line: *line,
                column: 1,
            }],
            SpandaError::DebugPause { line, reason } => vec![Diagnostic {
                message: format!("Debug pause: {reason}"),
                line: *line,
                column: 1,
            }],
        }
    }
}

impl From<spanda_lexer::LexerError> for SpandaError {
    fn from(err: spanda_lexer::LexerError) -> Self {
        Self::Lexer {
            message: err.message,
            line: err.line,
            column: err.column,
        }
    }
}

impl From<spanda_ast::RegexCompileError> for SpandaError {
    fn from(err: spanda_ast::RegexCompileError) -> Self {
        Self::Parse {
            message: err.message,
            line: err.line,
            column: err.column,
        }
    }
}
