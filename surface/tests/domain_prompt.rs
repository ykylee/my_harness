use myharness::domain::prompt_for;
use myharness::engine::print::{PRINT_CMD_COMMENT, ephemeral_argv, format_cmd, oneshot_argv};

#[test]
fn twelve_verbs_resolve() {
    let cases = [
        ("code", "review"),
        ("code", "implement"),
        ("code", "test"),
        ("code", "commit"),
        ("server", "status"),
        ("server", "logs"),
        ("server", "deploy"),
        ("server", "config"),
        ("env", "setup"),
        ("env", "install"),
        ("env", "shell"),
        ("env", "diagnose"),
    ];
    for (d, v) in cases {
        assert!(prompt_for(d, v, &["x".into()]).is_some(), "{d} {v}");
    }
}

#[test]
fn oneshot_has_no_plugin_dir() {
    let argv = oneshot_argv("/bin/grok", "minimax", "hi");
    assert!(!argv.iter().any(|a| a == "--plugin-dir"));
    assert!(argv.iter().any(|a| a == "-p"));
    assert!(argv.iter().any(|a| a == "--always-approve"));
}

#[test]
fn ephemeral_is_plan_not_yolo() {
    let argv = ephemeral_argv("/bin/grok", "minimax", "hi");
    assert!(!argv.iter().any(|a| a == "--plugin-dir"));
    assert!(!argv.iter().any(|a| a == "--always-approve"));
    assert!(argv.windows(2).any(|w| w == ["--permission-mode", "plan"]));
    assert!(argv.iter().any(|a| a == "-p"));
}

#[test]
fn print_cmd_comment_is_stable() {
    assert_eq!(PRINT_CMD_COMMENT, "# no TTY, stderr piped");
    let argv = oneshot_argv("grok", "minimax", "hi");
    let line = format_cmd(&argv);
    assert!(line.contains("-p"));
    assert!(!line.contains("--plugin-dir"));
}
