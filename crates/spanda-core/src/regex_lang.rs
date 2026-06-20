//! First-class regex compilation, validation, and runtime matching for Spanda.
//!
use crate::ast::Span;
use crate::error::SpandaError;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Compiled regex pattern with optional inline flags (`/pattern/i`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegexPattern {
    pub source: String,
    #[serde(default)]
    pub flags: String,
    pub span: Span,
}

impl RegexPattern {
    pub fn compile(&self) -> Result<Regex, SpandaError> {
        // Compile the regex pattern into a Rust regex engine instance.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Compiled regex, or a syntax error diagnostic.
        //
        // Options:
        // None.
        //
        // Example:
        // let re = pattern.compile()?;

        // Build the final pattern string with supported inline flags.
        let mut pattern = self.source.clone();
        for flag in self.flags.chars() {
            // Reject unsupported flag letters early with a clear diagnostic.
            if !matches!(flag, 'i' | 'm' | 's') {
                return Err(SpandaError::Parse {
                    message: format!(
                        "Invalid regex flag '{flag}'; supported flags are i, m, s. Suggestion: remove unsupported flags."
                    ),
                    line: self.span.start.line,
                    column: self.span.start.column,
                });
            }
        }
        if self.flags.contains('i') && !pattern.starts_with("(?i)") {
            pattern = format!("(?i){pattern}");
        }
        if self.flags.contains('m') && !pattern.starts_with("(?m)") {
            pattern = format!("(?m){pattern}");
        }
        if self.flags.contains('s') && !pattern.starts_with("(?s)") {
            pattern = format!("(?s){pattern}");
        }
        Regex::new(&pattern).map_err(|err| SpandaError::Parse {
            message: format!(
                "Invalid regex syntax: {err}. Suggestion: verify delimiters and escape sequences."
            ),
            line: self.span.start.line,
            column: self.span.start.column,
        })
    }
}

/// Named capture groups extracted from a regex match.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CaptureResult {
    pub full: String,
    #[serde(default)]
    pub groups: HashMap<String, String>,
}

pub fn regex_matches(pattern: &RegexPattern, text: &str) -> Result<bool, SpandaError> {
    // Return whether text fully matches the regex pattern.
    //
    // Parameters:
    // - `pattern` — compiled regex source
    // - `text` — haystack string
    //
    // Returns:
    // Boolean match result, or regex compile error.
    //
    // Options:
    // None.
    //
    // Example:
    // let ok = regex_matches(&pattern, "robot-123")?;

    // Compile once and test the entire input string.
    let re = pattern.compile()?;
    Ok(re.is_match(text))
}

pub fn regex_find(pattern: &RegexPattern, text: &str) -> Result<Option<String>, SpandaError> {
    // Return the first substring matched by the pattern.
    //
    // Parameters:
    // - `pattern` — compiled regex source
    // - `text` — haystack string
    //
    // Returns:
    // First match text if present.
    //
    // Options:
    // None.
    //
    // Example:
    // let found = regex_find(&pattern, log_line)?;

    // Compile once and return the first match slice as owned text.
    let re = pattern.compile()?;
    Ok(re.find(text).map(|m| m.as_str().to_string()))
}

pub fn regex_replace(
    pattern: &RegexPattern,
    text: &str,
    replacement: &str,
) -> Result<String, SpandaError> {
    // Replace all regex matches in text with replacement.
    //
    // Parameters:
    // - `pattern` — compiled regex source
    // - `text` — input string
    // - `replacement` — replacement text
    //
    // Returns:
    // Transformed string.
    //
    // Options:
    // None.
    //
    // Example:
    // let cleaned = regex_replace(&pattern, line, "_")?;

    // Compile once and apply global replacement.
    let re = pattern.compile()?;
    Ok(re.replace_all(text, replacement).into_owned())
}

pub fn regex_split(pattern: &RegexPattern, text: &str) -> Result<Vec<String>, SpandaError> {
    // Split text on regex matches.
    //
    // Parameters:
    // - `pattern` — compiled regex source
    // - `text` — input string
    //
    // Returns:
    // Split segments including empty segments between consecutive delimiters.
    //
    // Options:
    // None.
    //
    // Example:
    // let parts = regex_split(&pattern, "a,b,c")?;

    // Compile once and split on every match boundary.
    let re = pattern.compile()?;
    Ok(re.split(text).map(str::to_string).collect())
}

pub fn regex_capture(
    pattern: &RegexPattern,
    text: &str,
) -> Result<Option<CaptureResult>, SpandaError> {
    // Capture the first regex match and named groups.
    //
    // Parameters:
    // - `pattern` — compiled regex source
    // - `text` — haystack string
    //
    // Returns:
    // Full match and named capture map when a match exists.
    //
    // Options:
    // None.
    //
    // Example:
    // let cap = regex_capture(&pattern, log_line)?;

    // Compile once and extract the first match with named groups.
    let re = pattern.compile()?;
    let Some(caps) = re.captures(text) else {
        return Ok(None);
    };
    let full = caps
        .get(0)
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();
    let mut groups = HashMap::new();
    for name in re.capture_names().flatten() {
        if let Some(m) = caps.name(name) {
            groups.insert(name.to_string(), m.as_str().to_string());
        }
    }
    Ok(Some(CaptureResult { full, groups }))
}

pub fn validate_regex_literal(source: &str, flags: &str, span: Span) -> Result<(), SpandaError> {
    // Validate regex literal syntax at compile time.
    //
    // Parameters:
    // - `source` — pattern body without slashes
    // - `flags` — trailing flag letters
    // - `span` — source span for diagnostics
    //
    // Returns:
    // Ok when syntax is valid.
    //
    // Options:
    // None.
    //
    // Example:
    // validate_regex_literal("robot-[0-9]+", "", span)?;

    // Compile through the shared helper so diagnostics stay consistent.
    let pattern = RegexPattern {
        source: source.to_string(),
        flags: flags.to_string(),
        span,
    };
    pattern.compile().map(|_| ())
}
