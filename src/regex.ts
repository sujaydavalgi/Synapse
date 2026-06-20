/**
 * First-class regex compilation, validation, and runtime matching for Spanda.
 * @module
 */

import type { Span } from "./ast/nodes.js";

export type RegexPattern = {
  source: string;
  flags: string;
  span: Span;
};

export type CaptureResult = {
  full: string;
  groups: Record<string, string>;
};

export class RegexError extends Error {
  constructor(
    message: string,
    public line: number,
    public column: number,
  ) {
    super(message);
    this.name = "RegexError";
  }
}

export function compileRegex(pattern: RegexPattern): RegExp {
  // Compile the regex pattern into a JavaScript RegExp instance.
  //
  // Parameters:
  // - `pattern` — regex literal source and flags
  //
  // Returns:
  // Compiled RegExp, or throws RegexError on invalid flags or syntax.
  //
  // Options:
  // None.
  //
  // Example:
  // const re = compileRegex(pattern);

  let jsFlags = "";
  for (const flag of pattern.flags) {
    if (!"ims".includes(flag)) {
      throw new RegexError(
        `Invalid regex flag '${flag}'; supported flags are i, m, s. Suggestion: remove unsupported flags.`,
        pattern.span.start.line,
        pattern.span.start.column,
      );
    }
    if (!jsFlags.includes(flag)) {
      jsFlags += flag;
    }
  }

  try {
    return new RegExp(pattern.source, jsFlags);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    throw new RegexError(
      `Invalid regex syntax: ${message}. Suggestion: verify delimiters and escape sequences.`,
      pattern.span.start.line,
      pattern.span.start.column,
    );
  }
}

export function regexMatches(pattern: RegexPattern, text: string): boolean {
  // Return whether text matches the regex pattern anywhere in the string.
  //
  // Parameters:
  // - `pattern` — compiled regex source
  // - `text` — haystack string
  //
  // Returns:
  // Boolean match result.
  //
  // Options:
  // None.
  //
  // Example:
  // const ok = regexMatches(pattern, "robot-123");

  const re = compileRegex(pattern);
  return re.test(text);
}

export function regexFind(pattern: RegexPattern, text: string): string | null {
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
  // const found = regexFind(pattern, logLine);

  const re = compileRegex(pattern);
  const match = re.exec(text);
  return match ? match[0] : null;
}

export function regexReplace(pattern: RegexPattern, text: string, replacement: string): string {
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
  // const cleaned = regexReplace(pattern, line, "_");

  const re = compileRegex(pattern);
  return text.replace(re, replacement);
}

export function regexSplit(pattern: RegexPattern, text: string): string[] {
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
  // const parts = regexSplit(pattern, "a,b,c");

  const re = compileRegex(pattern);
  return text.split(re);
}

export function regexCapture(pattern: RegexPattern, text: string): CaptureResult | null {
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
  // const cap = regexCapture(pattern, logLine);

  const re = compileRegex(pattern);
  const match = re.exec(text);
  if (!match) {
    return null;
  }
  const groups: Record<string, string> = {};
  for (const name of Object.keys(match.groups ?? {})) {
    const value = match.groups?.[name];
    if (value !== undefined) {
      groups[name] = value;
    }
  }
  return { full: match[0], groups };
}

export function validateRegexLiteral(source: string, flags: string, span: Span): void {
  // Validate regex literal syntax at compile time.
  //
  // Parameters:
  // - `source` — pattern body without slashes
  // - `flags` — trailing flag letters
  // - `span` — source span for diagnostics
  //
  // Returns:
  // Nothing; throws RegexError when syntax is invalid.
  //
  // Options:
  // None.
  //
  // Example:
  // validateRegexLiteral("robot-[0-9]+", "", span);

  compileRegex({ source, flags, span });
}

export function regexFromLexeme(lexeme: string, span: Span): RegexPattern {
  // Convert a `/pattern/flags` lexeme into structured pattern data.
  //
  // Parameters:
  // - `lexeme` — raw regex literal text including slashes
  // - `span` — source span for diagnostics
  //
  // Returns:
  // Parsed regex pattern.
  //
  // Options:
  // None.
  //
  // Example:
  // const pattern = regexFromLexeme("/^robot-[0-9]+$/", span);

  const trimmed = lexeme.trimStart().slice(1);
  const slashIdx = trimmed.lastIndexOf("/");
  if (slashIdx < 0) {
    throw new RegexError(`Malformed regex literal '${lexeme}'`, span.start.line, span.start.column);
  }
  const source = trimmed.slice(0, slashIdx);
  const flags = trimmed.slice(slashIdx + 1);
  return { source, flags, span };
}
