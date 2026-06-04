fn placeholder(command: &str, purpose: &str) -> Result<(), String> {
    println!("koba {command}: {purpose}");
    println!("status: placeholder");
    Ok(())
}

pub fn init(apply: bool) -> Result<(), String> {
    crate::init::run(
        std::env::current_dir().map_err(|error| error.to_string())?,
        crate::init::InitOptions { apply },
    )
}

pub fn scan() -> Result<(), String> {
    crate::scan::run(std::env::current_dir().map_err(|error| error.to_string())?)
}

pub fn doctor() -> Result<(), String> {
    crate::doctor::run(std::env::current_dir().map_err(|error| error.to_string())?)
}

pub fn run(stage: crate::run_checks::Stage, dry_run: bool) -> Result<(), String> {
    crate::run_checks::run(
        std::env::current_dir().map_err(|error| error.to_string())?,
        crate::run_checks::RunOptions { stage, dry_run },
    )
}

pub fn hooks() -> Result<(), String> {
    placeholder("hooks", "inspect or plan Git hook configuration")
}

pub fn suggest_commit() -> Result<(), String> {
    placeholder("suggest-commit", "suggest a safe commit command")
}

pub fn pr() -> Result<(), String> {
    placeholder("pr", "inspect or prepare pull request workflow assets")
}
