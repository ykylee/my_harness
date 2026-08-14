//! S4b: child stdin/stdout/stderr must be piped (no TTY). Stub in S1.

#[test]
fn spawn_fds_contract_documented() {
    // PR-S4b / S7: Command fds 0/1/2 = Stdio::piped(), no PTY.
    // This file exists so the contract has a home before ACP.
    let contract = "stdin/stdout/stderr=piped";
    assert!(contract.contains("piped"));
}
