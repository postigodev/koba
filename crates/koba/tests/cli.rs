use std::process::Command;

#[test]
fn cli_can_run_a_placeholder_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_koba"))
        .arg("scan")
        .output()
        .expect("failed to run koba binary");

    assert!(
        output.status.success(),
        "expected success, got status {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("koba scan"));
    assert!(stdout.contains("placeholder"));
}
