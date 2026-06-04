fn main() -> std::process::ExitCode {
    match koba::run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("koba: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
