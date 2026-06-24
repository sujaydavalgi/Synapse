//! Man-page lookup and optional roff generation for Spanda CLI commands.

use crate::language_reference::generate_cli_man_pages;

/// Look up a man page by command name (`run`, `spanda-run`, or `spanda run`).
pub fn lookup_man_page(query: &str) -> Option<(String, String)> {
    // Resolve a CLI subcommand to its man-page markdown body.
    //
    // Parameters:
    // - `query` — command name or alias
    //
    // Returns:
    // `(page_name, markdown)` when found.
    //
    // Options:
    // None.
    //
    // Example:
    // let page = lookup_man_page("verify");

    let normalized = normalize_man_query(query);
    for (name, body) in generate_cli_man_pages() {
        let stem = name.strip_suffix(".md").unwrap_or(&name);
        let short = stem.strip_prefix("spanda-").unwrap_or(stem);
        if normalized == stem || normalized == short || normalized == format!("spanda-{short}") {
            return Some((name, body));
        }
    }
    None
}

/// List available man page names (without `.md` suffix).
pub fn list_man_pages() -> Vec<String> {
    generate_cli_man_pages()
        .into_iter()
        .map(|(name, _)| name.strip_suffix(".md").unwrap_or(&name).to_string())
        .collect()
}

/// Convert man-page markdown to minimal roff for `man(1)` viewers.
pub fn markdown_man_to_roff(markdown: &str, page_name: &str) -> String {
    let section = "1";
    let mut out = String::new();
    out.push_str(&format!(
        ".TH \"{}\" \"{}\" \"Spanda\" \"Spanda CLI\"\n",
        page_name.to_uppercase(),
        section
    ));
    let mut in_code = false;
    for line in markdown.lines() {
        if line.starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            out.push_str(".nf\n");
            out.push_str(&roff_escape(line));
            out.push_str("\n.fi\n");
            continue;
        }
        if let Some(rest) = line.strip_prefix("## ") {
            out.push_str(&format!(".SH {}\n", rest.to_uppercase()));
        } else if let Some(rest) = line.strip_prefix("# ") {
            out.push_str(&format!(".SH {}\n", rest.to_uppercase()));
        } else if let Some(rest) = line.strip_prefix("- ") {
            out.push_str(".IP \\(bu 2\n");
            out.push_str(&roff_escape(rest));
            out.push('\n');
        } else if !line.is_empty() {
            out.push_str(&roff_escape(line));
            out.push_str("\n.PP\n");
        }
    }
    out
}

fn normalize_man_query(query: &str) -> String {
    let q = query
        .trim()
        .trim_start_matches("spanda ")
        .trim_start_matches("spanda-");
    if q.is_empty() || q == "spanda" {
        "spanda".into()
    } else {
        format!("spanda-{q}")
    }
}

fn roff_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language_reference::CLI_COMMAND_NAMES;

    #[test]
    fn registered_commands_match_man_pages() {
        for cmd in CLI_COMMAND_NAMES {
            let key = cmd.strip_prefix("spanda-").unwrap_or(cmd);
            assert!(lookup_man_page(key).is_some(), "missing man page for {cmd}");
        }
    }

    #[test]
    fn lookup_verify_man_page() {
        let (name, body) = lookup_man_page("verify").expect("verify man page");
        assert_eq!(name, "spanda-verify.md");
        assert!(body.contains("## SYNOPSIS"));
        assert!(body.contains("## EXIT STATUS"));
    }

    #[test]
    fn roff_contains_th_header() {
        let (_, md) = lookup_man_page("check").unwrap();
        let roff = markdown_man_to_roff(&md, "spanda-check");
        assert!(roff.contains(".TH"));
        assert!(roff.contains(".SH SYNOPSIS"));
    }
}
