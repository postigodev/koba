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

pub fn hooks_install(
    adapter: crate::hooks::HookAdapter,
    dry_run: bool,
    apply: bool,
) -> Result<(), String> {
    crate::hooks::run_install(
        std::env::current_dir().map_err(|error| error.to_string())?,
        crate::hooks::InstallOptions {
            adapter,
            dry_run,
            apply,
        },
    )
}

pub fn github(command: crate::github::GithubCommand) -> Result<(), String> {
    crate::github::run(
        std::env::current_dir().map_err(|error| error.to_string())?,
        command,
    )
}

pub fn suggest_commit() -> Result<(), String> {
    crate::suggest_commit::run(std::env::current_dir().map_err(|error| error.to_string())?)
}

pub fn pr() -> Result<(), String> {
    placeholder("pr", "inspect or prepare pull request workflow assets")
}
