use std::process::Command;

#[test]
fn cli_can_run_scan() {
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
    assert!(stdout.contains("Koba scan"));
    assert!(stdout.contains("Repository"));
    assert!(stdout.contains("Workflow"));
    assert!(stdout.contains("GitHub"));
}

#[test]
fn cli_can_run_doctor() {
    let output = Command::new(env!("CARGO_BIN_EXE_koba"))
        .arg("doctor")
        .output()
        .expect("failed to run koba binary");

    assert!(
        output.status.success(),
        "expected success, got status {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Koba doctor"));
    assert!(stdout.contains("Repository"));
}
