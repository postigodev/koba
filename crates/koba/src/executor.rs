use std::{path::Path, process::Command};

pub fn run_shell(cwd: &Path, command: &str) -> Result<(), String> {
    let status = shell_command(command)
        .current_dir(cwd)
        .status()
        .map_err(|error| format!("failed to execute `{command}`: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "check command failed with status {}: {command}",
            status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "terminated".to_owned())
        ))
    }
}

#[cfg(windows)]
fn shell_command(command: &str) -> Command {
    let mut shell = Command::new("cmd");
    shell.args(["/C", command]);
    shell
}

#[cfg(not(windows))]
fn shell_command(command: &str) -> Command {
    let mut shell = Command::new("sh");
    shell.args(["-c", command]);
    shell
}
