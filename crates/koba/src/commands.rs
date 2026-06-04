fn placeholder(command: &str, purpose: &str) -> Result<(), String> {
    println!("koba {command}: {purpose}");
    println!("status: placeholder");
    Ok(())
}

pub fn init() -> Result<(), String> {
    placeholder("init", "plan a local-first workflow configuration")
}

pub fn scan() -> Result<(), String> {
    crate::scan::run(std::env::current_dir().map_err(|error| error.to_string())?)
}

pub fn doctor() -> Result<(), String> {
    crate::doctor::run(std::env::current_dir().map_err(|error| error.to_string())?)
}

pub fn run() -> Result<(), String> {
    placeholder("run", "execute a configured workflow check")
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
