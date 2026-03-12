use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const WORKFLOW_FILES: &[&str] = &[
    "ci.yaml",
    "deploy.yaml",
    "proto.yaml",
    "security.yaml",
    "kong-sync.yaml",
    "api-lint.yaml",
    "tauri-build.yaml",
    "integration-test.yaml",
    "publish-app.yaml",
];

const INFRA_DIRECTORIES: &[&str] = &["infra/docker", "infra/kong", "infra/messaging/kafka"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitConfig {
    pub project_name: String,
    pub git_init: bool,
    pub sparse_checkout: bool,
    pub tiers: Vec<Tier>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    System,
    Business,
    Service,
}

impl Tier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Tier::System => "system",
            Tier::Business => "business",
            Tier::Service => "service",
        }
    }

    pub fn display(&self) -> &'static str {
        self.as_str()
    }
}

pub fn execute_init(config: &InitConfig) -> Result<()> {
    let base = Path::new(&config.project_name);
    let scaffold_root = resolve_scaffold_root();

    fs::create_dir_all(base)?;

    for tier in &config.tiers {
        fs::create_dir_all(base.join("regions").join(tier.as_str()))?;
    }

    fs::create_dir_all(base.join("api/proto"))?;
    fs::create_dir_all(base.join("api/openapi"))?;
    fs::create_dir_all(base.join("docs"))?;
    fs::create_dir_all(base.join("infra"))?;
    fs::create_dir_all(base.join(".devcontainer"))?;
    fs::create_dir_all(base.join(".github/workflows"))?;

    fs::write(
        base.join(".devcontainer/devcontainer.json"),
        generate_devcontainer_json(&config.project_name)?,
    )?;
    copy_scaffold_file(
        &scaffold_root,
        base,
        Path::new(".devcontainer/docker-compose.extend.yaml"),
    )?;
    copy_scaffold_file(
        &scaffold_root,
        base,
        Path::new(".devcontainer/post-create.sh"),
    )?;

    for workflow in WORKFLOW_FILES {
        copy_scaffold_file(
            &scaffold_root,
            base,
            &Path::new(".github").join("workflows").join(workflow),
        )?;
    }

    fs::write(base.join("docker-compose.yaml"), generate_docker_compose()?)?;
    copy_scaffold_file(&scaffold_root, base, Path::new(".env.example"))?;
    copy_scaffold_file(
        &scaffold_root,
        base,
        Path::new("docker-compose.override.yaml.example"),
    )?;

    for directory in INFRA_DIRECTORIES {
        copy_scaffold_directory(&scaffold_root, base, Path::new(directory))?;
    }

    fs::write(
        base.join("README.md"),
        generate_readme(&config.project_name),
    )?;

    if config.git_init {
        let status = Command::new("git").arg("init").current_dir(base).status();
        match status {
            Ok(outcome) if outcome.success() => {
                if config.sparse_checkout {
                    let _ = Command::new("git")
                        .args(["sparse-checkout", "init", "--cone"])
                        .current_dir(base)
                        .status();
                    let tier_paths: Vec<String> = config
                        .tiers
                        .iter()
                        .map(|tier| format!("regions/{}", tier.as_str()))
                        .collect();
                    let _ = Command::new("git")
                        .args(["sparse-checkout", "set"])
                        .args(&tier_paths)
                        .current_dir(base)
                        .status();
                }
            }
            _ => {
                eprintln!("warning: git init failed; the workspace scaffold was still created.");
            }
        }
    }

    Ok(())
}

fn resolve_scaffold_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn project_label(project_name: &str) -> String {
    Path::new(project_name)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or(project_name)
        .to_string()
}

fn scaffold_text(relative_path: &Path) -> Result<String> {
    let full_path = resolve_scaffold_root().join(relative_path);
    fs::read_to_string(&full_path)
        .with_context(|| format!("failed to read scaffold file {}", full_path.display()))
}

fn generate_devcontainer_json(project_name: &str) -> Result<String> {
    let content = scaffold_text(Path::new(".devcontainer/devcontainer.json"))?;
    let json = strip_json_line_comments(&content);
    let mut parsed: Value =
        serde_json::from_str(&json).context("failed to parse scaffold devcontainer.json")?;
    parsed["name"] = Value::String(project_label(project_name));
    Ok(format!("{}\n", serde_json::to_string_pretty(&parsed)?))
}

fn generate_docker_compose() -> Result<String> {
    let template = scaffold_text(Path::new("docker-compose.yaml"))?;
    let filtered = strip_application_services(&template);
    if !filtered.contains("kafka-init:") {
        bail!("filtered docker-compose scaffold lost kafka-init section");
    }
    Ok(filtered)
}

fn strip_application_services(template: &str) -> String {
    let lines: Vec<&str> = template.lines().collect();
    let mut output = String::new();
    let mut skipping = false;
    let mut index = 0;

    while index < lines.len() {
        let current = lines[index].trim_start();
        let next = lines.get(index + 1).map(|line| line.trim_start());

        if !skipping
            && current.starts_with("# ============================================================")
            && next.is_some_and(|line| line.starts_with("# System"))
        {
            skipping = true;
            index += 1;
            continue;
        }

        if skipping
            && current.starts_with("# ============================================================")
            && next.is_some_and(|line| line.starts_with("# Kafka"))
        {
            skipping = false;
        }

        if !skipping {
            output.push_str(lines[index]);
            output.push('\n');
        }

        index += 1;
    }

    output
}

fn generate_readme(project_name: &str) -> String {
    let label = project_label(project_name);
    format!(
        "# {label}\n\n\
Generated by k1s0.\n\n\
## Structure\n\n\
- `regions/` - tiered modules (`system`, `business`, `service`)\n\
- `api/proto/` - protobuf definitions\n\
- `api/openapi/` - OpenAPI definitions\n\
- `infra/` - local infrastructure assets and compose support files\n\
- `.devcontainer/` - shared development container setup\n\
- `.github/workflows/` - CI/CD workflows\n\n\
## Next steps\n\n\
1. Copy `.env.example` to `.env` if you need local overrides.\n\
2. Start shared infrastructure with `docker compose --profile infra up -d`.\n\
3. Generate services and clients under `regions/` as needed.\n"
    )
}

fn strip_json_line_comments(content: &str) -> String {
    content
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn copy_scaffold_file(
    source_root: &Path,
    destination_root: &Path,
    relative_path: &Path,
) -> Result<()> {
    let source = source_root.join(relative_path);
    let destination = destination_root.join(relative_path);

    let parent = destination.parent().ok_or_else(|| {
        anyhow!(
            "destination path {} does not have a parent directory",
            destination.display()
        )
    })?;
    fs::create_dir_all(parent)?;
    fs::copy(&source, &destination).with_context(|| {
        format!(
            "failed to copy scaffold file {} -> {}",
            source.display(),
            destination.display()
        )
    })?;
    Ok(())
}

fn copy_scaffold_directory(
    source_root: &Path,
    destination_root: &Path,
    relative_path: &Path,
) -> Result<()> {
    let source = source_root.join(relative_path);
    let destination = destination_root.join(relative_path);
    copy_directory_recursive(&source, &destination)
}

fn copy_directory_recursive(source: &Path, destination: &Path) -> Result<()> {
    if !source.is_dir() {
        bail!("scaffold directory does not exist: {}", source.display());
    }

    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_path = entry.path();
        let destination_path = destination.join(entry.file_name());

        if entry_path.is_dir() {
            copy_directory_recursive(&entry_path, &destination_path)?;
        } else {
            fs::copy(&entry_path, &destination_path).with_context(|| {
                format!(
                    "failed to copy scaffold file {} -> {}",
                    entry_path.display(),
                    destination_path.display()
                )
            })?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_tier_as_str() {
        assert_eq!(Tier::System.as_str(), "system");
        assert_eq!(Tier::Business.as_str(), "business");
        assert_eq!(Tier::Service.as_str(), "service");
    }

    #[test]
    fn test_tier_display() {
        assert_eq!(Tier::System.display(), "system");
        assert_eq!(Tier::Business.display(), "business");
        assert_eq!(Tier::Service.display(), "service");
    }

    #[test]
    fn test_init_config_creation() {
        let config = InitConfig {
            project_name: "test-project".to_string(),
            git_init: true,
            sparse_checkout: true,
            tiers: vec![Tier::System, Tier::Business],
        };
        assert_eq!(config.project_name, "test-project");
        assert!(config.git_init);
        assert!(config.sparse_checkout);
        assert_eq!(config.tiers.len(), 2);
    }

    #[test]
    fn test_execute_init_creates_scaffold_files() {
        let tmp = TempDir::new().unwrap();
        let project_name = tmp.path().join("my-project").to_string_lossy().to_string();
        let config = InitConfig {
            project_name,
            git_init: false,
            sparse_checkout: false,
            tiers: vec![Tier::System, Tier::Business, Tier::Service],
        };

        execute_init(&config).unwrap();

        let base = Path::new(&config.project_name);
        assert!(base.join("regions/system").is_dir());
        assert!(base.join("regions/business").is_dir());
        assert!(base.join("regions/service").is_dir());
        assert!(base.join("api/proto").is_dir());
        assert!(base.join("api/openapi").is_dir());
        assert!(base.join("infra/docker").is_dir());
        assert!(base.join("infra/kong").is_dir());
        assert!(base.join("infra/messaging/kafka").is_dir());
        assert!(base.join(".devcontainer/devcontainer.json").is_file());
        assert!(base
            .join(".devcontainer/docker-compose.extend.yaml")
            .is_file());
        assert!(base.join(".devcontainer/post-create.sh").is_file());
        assert!(base.join(".github/workflows/ci.yaml").is_file());
        assert!(base.join(".github/workflows/deploy.yaml").is_file());
        assert!(base.join(".github/workflows/proto.yaml").is_file());
        assert!(base.join(".github/workflows/security.yaml").is_file());
        assert!(base.join(".github/workflows/kong-sync.yaml").is_file());
        assert!(base.join(".github/workflows/api-lint.yaml").is_file());
        assert!(base.join(".github/workflows/tauri-build.yaml").is_file());
        assert!(base
            .join(".github/workflows/integration-test.yaml")
            .is_file());
        assert!(base.join(".github/workflows/publish-app.yaml").is_file());
        assert!(base.join("docker-compose.yaml").is_file());
        assert!(base.join("docker-compose.override.yaml.example").is_file());
        assert!(base.join(".env.example").is_file());
        assert!(base.join("README.md").is_file());
    }

    #[test]
    fn test_execute_init_partial_tiers() {
        let tmp = TempDir::new().unwrap();
        let project_name = tmp
            .path()
            .join("partial-project")
            .to_string_lossy()
            .to_string();
        let config = InitConfig {
            project_name,
            git_init: false,
            sparse_checkout: true,
            tiers: vec![Tier::System],
        };

        execute_init(&config).unwrap();

        let base = Path::new(&config.project_name);
        assert!(base.join("regions/system").is_dir());
        assert!(!base.join("regions/business").exists());
        assert!(!base.join("regions/service").exists());
    }

    #[test]
    fn test_execute_init_with_git() {
        let tmp = TempDir::new().unwrap();
        let project_name = tmp.path().join("git-project").to_string_lossy().to_string();
        let config = InitConfig {
            project_name,
            git_init: true,
            sparse_checkout: false,
            tiers: vec![Tier::System, Tier::Business, Tier::Service],
        };

        let result = execute_init(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_devcontainer_json_uses_project_label() {
        let json = generate_devcontainer_json("C:/tmp/my-app").unwrap();
        assert!(json.contains("\"name\": \"my-app\""));
        assert!(json.contains("\"dockerComposeFile\""));
    }

    #[test]
    fn test_generate_docker_compose_removes_application_services() {
        let yaml = generate_docker_compose().unwrap();
        assert!(yaml.contains("profiles: [infra]"));
        assert!(yaml.contains("kafka-init:"));
        assert!(yaml.contains("profiles: [observability]"));
        assert!(!yaml.contains("auth-rust:"));
        assert!(!yaml.contains("domain-master-rust:"));
    }

    #[test]
    fn test_generate_readme_uses_project_label() {
        let md = generate_readme("C:/tmp/my-app");
        assert!(md.contains("# my-app"));
        assert!(md.contains("docker compose --profile infra up -d"));
    }

    #[test]
    fn test_strip_json_line_comments_removes_comment_prefixes() {
        let stripped = strip_json_line_comments("// header\n{\n  \"name\": \"x\"\n}\n");
        assert_eq!(stripped, "{\n  \"name\": \"x\"\n}");
    }
}
