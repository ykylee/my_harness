//! Build grok -p argv. --plugin-dir is not valid here.

/// 12-verb / non-TTY CLI. `--always-approve` is allowed only here (K7).
pub fn oneshot_argv(grok: &str, model: &str, prompt: &str) -> Vec<String> {
    vec![
        grok.to_string(),
        "-m".into(),
        model.into(),
        "--always-approve".into(),
        "--output-format".into(),
        "plain".into(),
        "-p".into(),
        prompt.into(),
    ]
}

/// S3 debug TUI. plan mode, no YOLO, no `--plugin-dir`.
pub fn ephemeral_argv(grok: &str, model: &str, prompt: &str) -> Vec<String> {
    vec![
        grok.to_string(),
        "-m".into(),
        model.into(),
        "--permission-mode".into(),
        "plan".into(),
        "--output-format".into(),
        "plain".into(),
        "-p".into(),
        prompt.into(),
    ]
}

pub const PRINT_CMD_COMMENT: &str = "# no TTY, stderr piped";

pub fn format_cmd(argv: &[String]) -> String {
    argv.iter()
        .map(|a| shell_approx(a))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_approx(s: &str) -> String {
    if s.chars().any(|c| c.is_whitespace()) {
        format!("'{s}'")
    } else {
        s.to_string()
    }
}
