/// Basic source formatter for Spanda programs.
/// Trims trailing whitespace and ensures a trailing newline.
pub fn format_source(source: &str) -> String {
    let mut out = String::new();
    for line in source.lines() {
        out.push_str(line.trim_end());
        out.push('\n');
    }
    while out.ends_with("\n\n") {
        out.pop();
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_trailing_whitespace_and_adds_final_newline() {
        let input = "robot R {  \n  actuator wheels: DifferentialDrive; \n}\n\n";
        let formatted = format_source(input);
        assert_eq!(
            formatted,
            "robot R {\n  actuator wheels: DifferentialDrive;\n}\n"
        );
    }
}
