//! First-class regex compilation, validation, and runtime matching for Spanda.
//!
use spanda_ast::nodes::Span;
use spanda_error::SpandaError;
pub use spanda_ast::{CaptureResult, RegexCompileError, RegexPattern};
use std::collections::HashMap;

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
    pattern.compile().map_err(SpandaError::from).map(|_| ())
}
