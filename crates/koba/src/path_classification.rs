#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathClass {
    Readme,
    Docs,
    AgentSkill {
        slug: String,
        surface: AgentSkillSurface,
    },
    RustSource {
        module: Option<KobaModule>,
    },
    ScoopManifest,
    GithubWorkflow,
    GithubTemplate,
    Config,
    Test,
    Evals,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentSkillSurface {
    Definition,
    References,
    Examples,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KobaModule {
    Changes,
    Commit,
    Output,
    Pr,
    Hooks,
    Github,
    RunChecks,
    Init,
    Doctor,
    Scan,
    Repo,
    Config,
    Cli,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChangeConcept {
    Analysis,
    AgentsDocs,
    Skill,
    CommitEngine,
    Output,
    Pr,
    Hooks,
    Github,
    RunChecks,
    Init,
    Changes,
    Scoop,
    GithubCi,
    GithubTemplate,
    Rust,
    Docs,
    Config,
    Other,
}

pub fn classify(path: &str) -> PathClass {
    let path = normalize(path);
    let file_name = path.rsplit('/').next().unwrap_or(path.as_str());

    if path == "readme.md" {
        return PathClass::Readme;
    }

    if path == "docs/agents.md" {
        return PathClass::Docs;
    }

    if let Some((slug, surface)) = agent_skill_path(&path) {
        return PathClass::AgentSkill { slug, surface };
    }

    if path.starts_with("evals/") {
        return PathClass::Evals;
    }

    if is_scoop_manifest(&path) {
        return PathClass::ScoopManifest;
    }

    if is_github_workflow(&path) {
        return PathClass::GithubWorkflow;
    }

    if path == ".github/pull_request_template.md" {
        return PathClass::GithubTemplate;
    }

    if is_rust_source(&path) {
        return PathClass::RustSource {
            module: koba_module_for_path(&path),
        };
    }

    if is_test_file(&path) {
        return PathClass::Test;
    }

    if is_docs_file(&path) {
        return PathClass::Docs;
    }

    if is_config_file(&path) {
        return PathClass::Config;
    }

    if file_name.ends_with(".json") && path.starts_with("bucket/") {
        return PathClass::ScoopManifest;
    }

    PathClass::Other
}

pub fn concept_for_path(path: &str) -> ChangeConcept {
    let normalized = normalize(path);
    if normalized == "tests/smoke-prompts.md" {
        return ChangeConcept::Skill;
    }
    if is_analysis_infrastructure(&normalized) {
        return ChangeConcept::Analysis;
    }

    match classify(path) {
        PathClass::Docs if normalized == "docs/agents.md" => ChangeConcept::AgentsDocs,
        PathClass::AgentSkill { .. } | PathClass::Evals => ChangeConcept::Skill,
        PathClass::RustSource {
            module: Some(KobaModule::Commit),
        } => ChangeConcept::CommitEngine,
        PathClass::RustSource {
            module: Some(KobaModule::Output),
        } => ChangeConcept::Output,
        PathClass::RustSource {
            module: Some(KobaModule::Pr),
        } => ChangeConcept::Pr,
        PathClass::RustSource {
            module: Some(KobaModule::Hooks),
        } => ChangeConcept::Hooks,
        PathClass::RustSource {
            module: Some(KobaModule::Github),
        } => ChangeConcept::Github,
        PathClass::RustSource {
            module: Some(KobaModule::RunChecks),
        } => ChangeConcept::RunChecks,
        PathClass::RustSource {
            module: Some(KobaModule::Init),
        } => ChangeConcept::Init,
        PathClass::RustSource {
            module: Some(KobaModule::Changes),
        } => ChangeConcept::Changes,
        PathClass::ScoopManifest => ChangeConcept::Scoop,
        PathClass::GithubWorkflow => ChangeConcept::GithubCi,
        PathClass::GithubTemplate => ChangeConcept::GithubTemplate,
        PathClass::RustSource { .. } => ChangeConcept::Rust,
        PathClass::Docs | PathClass::Readme | PathClass::Test => ChangeConcept::Docs,
        PathClass::Config => ChangeConcept::Config,
        PathClass::Other => ChangeConcept::Other,
    }
}

pub fn commit_scope_for_path(path: &str) -> Option<&'static str> {
    let normalized = normalize(path);
    if normalized == "tests/smoke-prompts.md" {
        return Some("skill");
    }
    if is_analysis_infrastructure(&normalized) {
        return Some("analysis");
    }
    if normalized == "crates/koba/tests/cli.rs" {
        return Some("cli");
    }

    match classify(path) {
        PathClass::Docs if normalized == "docs/agents.md" => Some("agents"),
        PathClass::AgentSkill { .. } | PathClass::Evals => Some("skill"),
        PathClass::ScoopManifest => Some("scoop"),
        PathClass::GithubWorkflow | PathClass::GithubTemplate => Some("github"),
        PathClass::Docs if normalized.contains("docs/product") => Some("product"),
        PathClass::RustSource {
            module: Some(module),
        } => Some(module.scope()),
        _ => None,
    }
}

pub fn scope_priority() -> [&'static str; 17] {
    [
        "analysis", "agents", "skill", "scoop", "github", "changes", "commit", "output", "pr",
        "hooks", "run", "init", "doctor", "scan", "repo", "config", "cli",
    ]
}

pub fn is_analysis_refactor_path_set<'a>(paths: impl IntoIterator<Item = &'a str>) -> bool {
    let paths = paths.into_iter().map(normalize).collect::<Vec<_>>();
    let has_infrastructure = paths.iter().any(|path| is_analysis_infrastructure(path));
    let has_consumer = paths.iter().any(|path| is_analysis_consumer(path));

    has_infrastructure
        && has_consumer
        && paths
            .iter()
            .all(|path| is_analysis_infrastructure(path) || is_analysis_consumer(path))
}

fn is_analysis_infrastructure(path: &str) -> bool {
    matches!(
        normalize(path).as_str(),
        "crates/koba/src/git_status.rs" | "crates/koba/src/path_classification.rs"
    )
}

fn is_analysis_consumer(path: &str) -> bool {
    matches!(
        normalize(path).as_str(),
        "crates/koba/src/changes.rs"
            | "crates/koba/src/doctor.rs"
            | "crates/koba/src/suggest_commit.rs"
            | "crates/koba/src/pr.rs"
            | "crates/koba/src/git.rs"
            | "crates/koba/src/init.rs"
            | "crates/koba/src/lib.rs"
            | "crates/koba/src/scan.rs"
    )
}

pub fn is_docs_file(path: &str) -> bool {
    let path = normalize(path);
    path.starts_with("docs/")
        || path.ends_with(".md")
        || path.ends_with(".mdx")
        || path.ends_with(".rst")
        || path == ".github/pull_request_template.md"
}

pub fn is_test_file(path: &str) -> bool {
    let path = normalize(path);
    let file_name = path.rsplit('/').next().unwrap_or(path.as_str());
    path.contains("/tests/")
        || path.starts_with("tests/")
        || path.contains("__tests__")
        || file_name.ends_with("_test.rs")
        || file_name.ends_with("_tests.rs")
        || file_name.starts_with("test_")
        || file_name.contains(".test.")
        || file_name.contains(".spec.")
}

pub fn is_chore_file(path: &str) -> bool {
    let path = normalize(path);
    let file_name = path.rsplit('/').next().unwrap_or(path.as_str());
    matches!(
        file_name,
        "cargo.toml"
            | "cargo.lock"
            | "package.json"
            | "package-lock.json"
            | "pnpm-lock.yaml"
            | "yarn.lock"
            | ".gitignore"
            | ".gitattributes"
            | "rust-toolchain.toml"
            | "rustfmt.toml"
    ) || (path.starts_with(".github/") && path != ".github/pull_request_template.md")
        || path.ends_with(".yml")
        || path.ends_with(".yaml")
        || path.ends_with(".toml")
        || path.ends_with(".json")
}

pub fn is_feature_signal(status: &str, path: &str) -> bool {
    let path = normalize(path);
    let file_name = path.rsplit('/').next().unwrap_or(path.as_str());
    let added = status.contains('A') || status == "??";

    file_name == "cli.rs"
        || (matches!(classify(&path), PathClass::RustSource { .. }) && path.contains("/src/"))
        || matches!(
            classify(&path),
            PathClass::AgentSkill { .. } | PathClass::Evals
        )
        || path.starts_with("evals/")
        || (added && (path.ends_with(".rs") || path.ends_with(".ts") || path.ends_with(".tsx")))
}

pub fn is_skill_repo_file(path: &str) -> bool {
    let path = normalize(path);
    is_readme(&path)
        || is_agent_skill_file(&path)
        || path.starts_with("evals/")
        || path == "tests/smoke-prompts.md"
}

pub fn is_agent_skill_file(path: &str) -> bool {
    let path = normalize(path);
    matches!(
        classify(&path),
        PathClass::AgentSkill { .. } | PathClass::Evals
    ) || path == "tests/smoke-prompts.md"
}

pub fn is_agent_skill_enhancement(paths: &[String]) -> bool {
    paths.iter().any(|path| {
        let path = normalize(path);
        path.starts_with("evals/")
            || path == "tests/smoke-prompts.md"
            || path.contains("/examples/")
            || matches!(
                classify(&path),
                PathClass::AgentSkill {
                    surface: AgentSkillSurface::Other,
                    ..
                }
            )
    })
}

pub fn is_weak_support_file(path: &str) -> bool {
    let path = normalize(path);
    is_readme(&path)
        || path == "crates/koba/src/cli.rs"
        || path == "crates/koba/src/commands.rs"
        || path == "crates/koba/src/lib.rs"
        || path == "crates/koba/tests/cli.rs"
}

pub fn is_readme(path: &str) -> bool {
    normalize(path) == "readme.md"
}

pub fn is_rust_source(path: &str) -> bool {
    normalize(path).ends_with(".rs")
}

pub fn is_cargo_file(path: &str) -> bool {
    let path = normalize(path);
    path.ends_with("cargo.toml") || path.ends_with("cargo.lock")
}

pub fn is_scoop_manifest(path: &str) -> bool {
    let path = normalize(path);
    (path.starts_with("packaging/scoop/") || path.starts_with("bucket/")) && path.ends_with(".json")
}

pub fn is_github_workflow(path: &str) -> bool {
    normalize(path).starts_with(".github/workflows/")
}

pub fn is_js_ts_source(path: &str) -> bool {
    let path = normalize(path);
    path.ends_with(".js")
        || path.ends_with(".jsx")
        || path.ends_with(".ts")
        || path.ends_with(".tsx")
        || path.ends_with(".mjs")
        || path.ends_with(".cjs")
}

pub fn is_python_source(path: &str) -> bool {
    normalize(path).ends_with(".py")
}

pub fn normalize(path: &str) -> String {
    path.replace('\\', "/").to_ascii_lowercase()
}

impl KobaModule {
    fn scope(self) -> &'static str {
        match self {
            KobaModule::Changes => "changes",
            KobaModule::Commit => "commit",
            KobaModule::Output => "output",
            KobaModule::Pr => "pr",
            KobaModule::Hooks => "hooks",
            KobaModule::Github => "github",
            KobaModule::RunChecks => "run",
            KobaModule::Init => "init",
            KobaModule::Doctor => "doctor",
            KobaModule::Scan => "scan",
            KobaModule::Repo => "repo",
            KobaModule::Config => "config",
            KobaModule::Cli => "cli",
        }
    }
}

fn agent_skill_path(path: &str) -> Option<(String, AgentSkillSurface)> {
    let mut parts = path.split('/');
    if parts.next() != Some("skills") {
        return None;
    }
    let slug = parts.next()?.to_owned();
    if slug.is_empty() {
        return None;
    }
    let surface = match parts.next()? {
        "SKILL.md" | "skill.md" => AgentSkillSurface::Definition,
        "references" => AgentSkillSurface::References,
        "examples" => AgentSkillSurface::Examples,
        _ => AgentSkillSurface::Other,
    };

    Some((slug, surface))
}

fn koba_module_for_path(path: &str) -> Option<KobaModule> {
    match path {
        "crates/koba/src/changes.rs" => Some(KobaModule::Changes),
        "crates/koba/src/suggest_commit.rs" => Some(KobaModule::Commit),
        "crates/koba/src/output.rs" => Some(KobaModule::Output),
        "crates/koba/src/pr.rs" => Some(KobaModule::Pr),
        "crates/koba/src/hooks.rs" => Some(KobaModule::Hooks),
        "crates/koba/src/github.rs" => Some(KobaModule::Github),
        "crates/koba/src/run_checks.rs" => Some(KobaModule::RunChecks),
        "crates/koba/src/init.rs" => Some(KobaModule::Init),
        "crates/koba/src/doctor.rs" => Some(KobaModule::Doctor),
        "crates/koba/src/scan.rs" => Some(KobaModule::Scan),
        "crates/koba/src/repo.rs" => Some(KobaModule::Repo),
        "crates/koba/src/config.rs" => Some(KobaModule::Config),
        "crates/koba/src/cli.rs" | "crates/koba/src/commands.rs" | "crates/koba/src/lib.rs" => {
            Some(KobaModule::Cli)
        }
        _ => None,
    }
}

fn is_config_file(path: &str) -> bool {
    let path = normalize(path);
    path.ends_with(".toml")
        || path.ends_with(".yml")
        || path.ends_with(".yaml")
        || path.ends_with(".json")
        || path == ".gitignore"
        || path == ".gitattributes"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_generic_agent_skill_docs() {
        assert_eq!(
            commit_scope_for_path("skills/hoi4-modding/SKILL.md"),
            Some("skill")
        );
        assert!(matches!(
            classify("skills/hoi4-modding/references/workflows.md"),
            PathClass::AgentSkill {
                slug,
                surface: AgentSkillSurface::References
            } if slug == "hoi4-modding"
        ));
    }

    #[test]
    fn classifies_agent_skill_examples_evals_and_smoke_prompts() {
        assert!(matches!(
            classify("skills/hoi4-modding/examples/minimal-event.txt"),
            PathClass::AgentSkill {
                surface: AgentSkillSurface::Examples,
                ..
            }
        ));
        assert_eq!(classify("evals/trigger-evals.json"), PathClass::Evals);
        assert!(is_agent_skill_file("tests/smoke-prompts.md"));
        assert_eq!(
            concept_for_path("tests/smoke-prompts.md"),
            ChangeConcept::Skill
        );
        assert_eq!(
            commit_scope_for_path("tests/smoke-prompts.md"),
            Some("skill")
        );
    }

    #[test]
    fn classifies_scoop_manifest_paths() {
        assert_eq!(
            classify("packaging/scoop/bucket/koba.json"),
            PathClass::ScoopManifest
        );
        assert_eq!(classify("bucket/koba.json"), PathClass::ScoopManifest);
        assert_eq!(
            commit_scope_for_path("packaging/scoop/bucket/koba.json"),
            Some("scoop")
        );
    }

    #[test]
    fn classifies_github_workflow_and_template_paths() {
        assert_eq!(
            classify(".github/workflows/ci.yml"),
            PathClass::GithubWorkflow
        );
        assert_eq!(
            classify(".github/pull_request_template.md"),
            PathClass::GithubTemplate
        );
    }

    #[test]
    fn classifies_koba_source_module_scopes() {
        assert_eq!(
            commit_scope_for_path("crates/koba/src/changes.rs"),
            Some("changes")
        );
        assert_eq!(
            commit_scope_for_path("crates/koba/src/suggest_commit.rs"),
            Some("commit")
        );
        assert_eq!(
            commit_scope_for_path("crates/koba/src/output.rs"),
            Some("output")
        );
        assert_eq!(commit_scope_for_path("crates/koba/src/pr.rs"), Some("pr"));
        assert_eq!(
            commit_scope_for_path("crates/koba/src/hooks.rs"),
            Some("hooks")
        );
    }

    #[test]
    fn normalizes_windows_path_separators() {
        assert_eq!(
            commit_scope_for_path("skills\\hoi4-modding\\SKILL.md"),
            Some("skill")
        );
    }

    #[test]
    fn detects_cross_cutting_analysis_refactor_path_set() {
        assert!(is_analysis_refactor_path_set([
            "crates/koba/src/git_status.rs",
            "crates/koba/src/path_classification.rs",
            "crates/koba/src/changes.rs",
            "crates/koba/src/suggest_commit.rs",
            "crates/koba/src/pr.rs",
            "crates/koba/src/doctor.rs",
            "crates/koba/src/init.rs",
            "crates/koba/src/scan.rs",
        ]));
        assert!(!is_analysis_refactor_path_set([
            "crates/koba/src/changes.rs",
            "crates/koba/src/pr.rs",
        ]));
    }
}
