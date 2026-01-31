//! 依存関係分析

use std::path::Path;

use super::types::{DependencyAnalysis, DetectedProjectType, EnvVarUsage, SecretUsage};

/// プロジェクトの依存関係を分析する
pub fn analyze_dependencies(
    path: &Path,
    project_type: &DetectedProjectType,
) -> DependencyAnalysis {
    let external_dependencies = parse_dependencies(path, project_type);
    let env_files = detect_env_files(path);
    let env_var_usages = scan_env_var_usages(path, project_type);
    let hardcoded_secrets = scan_hardcoded_secrets(path);

    DependencyAnalysis {
        env_var_usages,
        hardcoded_secrets,
        external_dependencies,
        env_files,
    }
}

fn parse_dependencies(path: &Path, project_type: &DetectedProjectType) -> Vec<String> {
    match project_type {
        DetectedProjectType::BackendRust => parse_cargo_dependencies(path),
        DetectedProjectType::BackendGo => parse_go_dependencies(path),
        DetectedProjectType::BackendCsharp => parse_csharp_dependencies(path),
        DetectedProjectType::BackendPython => parse_python_dependencies(path),
        DetectedProjectType::FrontendReact => parse_npm_dependencies(path),
        DetectedProjectType::FrontendFlutter => parse_pubspec_dependencies(path),
        DetectedProjectType::Unknown => Vec::new(),
    }
}

fn parse_cargo_dependencies(path: &Path) -> Vec<String> {
    let cargo_path = path.join("Cargo.toml");
    let content = match std::fs::read_to_string(&cargo_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut deps = Vec::new();
    let mut in_deps_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[dependencies]"
            || trimmed == "[dev-dependencies]"
            || trimmed == "[build-dependencies]"
        {
            in_deps_section = trimmed == "[dependencies]";
            continue;
        }

        if trimmed.starts_with('[') {
            in_deps_section = false;
            continue;
        }

        if in_deps_section {
            if let Some(name) = extract_toml_key(trimmed) {
                deps.push(name);
            }
        }

        // Also handle dependencies.xxx inline tables
        if trimmed.starts_with("dependencies.") {
            if let Some(rest) = trimmed.strip_prefix("dependencies.") {
                if let Some(name) = rest.split('.').next() {
                    deps.push(name.trim().to_string());
                }
            }
        }
    }

    deps
}

fn parse_go_dependencies(path: &Path) -> Vec<String> {
    let go_mod = path.join("go.mod");
    let content = match std::fs::read_to_string(&go_mod) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut deps = Vec::new();
    let mut in_require = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "require (" {
            in_require = true;
            continue;
        }

        if trimmed == ")" {
            in_require = false;
            continue;
        }

        if in_require && !trimmed.is_empty() && !trimmed.starts_with("//") {
            if let Some(module) = trimmed.split_whitespace().next() {
                deps.push(module.to_string());
            }
        }

        // single-line require
        if trimmed.starts_with("require ") && !trimmed.contains('(') {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                deps.push(parts[1].to_string());
            }
        }
    }

    deps
}

fn parse_csharp_dependencies(path: &Path) -> Vec<String> {
    let mut deps = Vec::new();

    // Scan for .csproj files in path and src/
    let dirs_to_scan = [path.to_path_buf(), path.join("src")];

    for dir in &dirs_to_scan {
        scan_csproj_files(dir, &mut deps);
    }

    deps
}

fn scan_csproj_files(dir: &Path, deps: &mut Vec<String>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            scan_csproj_files(&entry_path, deps);
        } else if entry_path.is_file() {
            if let Some(name) = entry_path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".csproj") {
                    if let Ok(content) = std::fs::read_to_string(&entry_path) {
                        extract_csproj_packages(&content, deps);
                    }
                }
            }
        }
    }
}

fn extract_csproj_packages(content: &str, deps: &mut Vec<String>) {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("PackageReference") && trimmed.contains("Include=") {
            if let Some(start) = trimmed.find("Include=\"") {
                let rest = &trimmed[start + 9..];
                if let Some(end) = rest.find('"') {
                    deps.push(rest[..end].to_string());
                }
            }
        }
    }
}

fn parse_python_dependencies(path: &Path) -> Vec<String> {
    // Try pyproject.toml first
    if let Ok(content) = std::fs::read_to_string(path.join("pyproject.toml")) {
        let deps = extract_python_deps_from_pyproject(&content);
        if !deps.is_empty() {
            return deps;
        }
    }

    // Fallback to requirements.txt
    if let Ok(content) = std::fs::read_to_string(path.join("requirements.txt")) {
        return extract_python_deps_from_requirements(&content);
    }

    Vec::new()
}

fn extract_python_deps_from_pyproject(content: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let mut in_deps = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "dependencies = [" || trimmed.starts_with("dependencies = [") {
            in_deps = true;
            // handle inline deps on same line
            if let Some(bracket_content) = trimmed.strip_prefix("dependencies = [") {
                extract_inline_python_deps(bracket_content, &mut deps);
            }
            continue;
        }

        if in_deps {
            if trimmed == "]" || trimmed.starts_with(']') {
                in_deps = false;
                continue;
            }
            let dep = trimmed
                .trim_matches(|c: char| c == '"' || c == '\'' || c == ',')
                .trim();
            if !dep.is_empty() {
                // Extract just the package name (before any version specifier)
                let name = dep
                    .split(['>', '<', '=', '~', '!'])
                    .next()
                    .unwrap_or(dep)
                    .trim();
                if !name.is_empty() {
                    deps.push(name.to_string());
                }
            }
        }
    }

    deps
}

fn extract_inline_python_deps(content: &str, deps: &mut Vec<String>) {
    let stripped = content.trim_end_matches(']');
    for part in stripped.split(',') {
        let dep = part
            .trim()
            .trim_matches(|c: char| c == '"' || c == '\'')
            .trim();
        if !dep.is_empty() {
            let name = dep
                .split(['>', '<', '=', '~', '!'])
                .next()
                .unwrap_or(dep)
                .trim();
            if !name.is_empty() {
                deps.push(name.to_string());
            }
        }
    }
}

fn extract_python_deps_from_requirements(content: &str) -> Vec<String> {
    let mut deps = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('-') {
            continue;
        }
        let name = trimmed
            .split(['>', '<', '=', '~', '!', '['])
            .next()
            .unwrap_or(trimmed)
            .trim();
        if !name.is_empty() {
            deps.push(name.to_string());
        }
    }
    deps
}

fn parse_npm_dependencies(path: &Path) -> Vec<String> {
    let package_json = path.join("package.json");
    let content = match std::fs::read_to_string(&package_json) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let value: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut deps = Vec::new();

    if let Some(obj) = value.get("dependencies").and_then(|v| v.as_object()) {
        for key in obj.keys() {
            deps.push(key.clone());
        }
    }

    deps
}

fn parse_pubspec_dependencies(path: &Path) -> Vec<String> {
    let pubspec = path.join("pubspec.yaml");
    let content = match std::fs::read_to_string(&pubspec) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut deps = Vec::new();
    let mut in_deps = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "dependencies:" {
            in_deps = true;
            continue;
        }

        // Another top-level key ends the dependencies section
        if !line.starts_with(' ') && !line.starts_with('\t') && trimmed.ends_with(':') && in_deps {
            in_deps = false;
            continue;
        }

        if in_deps && !trimmed.is_empty() && !trimmed.starts_with('#') {
            if let Some(name) = trimmed.split(':').next() {
                let name = name.trim();
                if !name.is_empty() && name != "flutter" && name != "sdk" {
                    deps.push(name.to_string());
                }
            }
        }
    }

    deps
}

fn detect_env_files(path: &Path) -> Vec<String> {
    let patterns = [
        ".env",
        ".env.local",
        ".env.development",
        ".env.production",
        ".env.staging",
        ".env.test",
        ".env.example",
    ];

    patterns
        .iter()
        .filter(|f| path.join(f).exists())
        .map(|f| (*f).to_string())
        .collect()
}

fn scan_env_var_usages(path: &Path, project_type: &DetectedProjectType) -> Vec<EnvVarUsage> {
    let mut usages = Vec::new();

    let env_files = [
        ".env",
        ".env.local",
        ".env.development",
        ".env.production",
        ".env.staging",
        ".env.test",
    ];

    for env_file in &env_files {
        let file_path = path.join(env_file);
        if !file_path.exists() {
            continue;
        }

        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((key, _)) = trimmed.split_once('=') {
                usages.push(EnvVarUsage {
                    file: (*env_file).to_string(),
                    line: line_num + 1,
                    pattern: format!("{}=...", key),
                    var_name: Some(key.trim().to_string()),
                });
            }
        }
    }

    // Also scan source code for env var patterns
    let src_dir = match project_type {
        DetectedProjectType::BackendRust => "src",
        DetectedProjectType::BackendGo => "internal",
        DetectedProjectType::BackendCsharp => "src",
        DetectedProjectType::BackendPython => "src",
        DetectedProjectType::FrontendReact => "src",
        DetectedProjectType::FrontendFlutter => "lib",
        DetectedProjectType::Unknown => return usages,
    };

    let src_path = path.join(src_dir);
    if src_path.exists() {
        scan_source_for_env_vars(&src_path, path, project_type, &mut usages);
    }

    usages
}

fn scan_source_for_env_vars(
    dir: &Path,
    base_path: &Path,
    project_type: &DetectedProjectType,
    usages: &mut Vec<EnvVarUsage>,
) {
    let extensions = match project_type {
        DetectedProjectType::BackendRust => &["rs"][..],
        DetectedProjectType::BackendGo => &["go"][..],
        DetectedProjectType::BackendCsharp => &["cs"][..],
        DetectedProjectType::BackendPython => &["py"][..],
        DetectedProjectType::FrontendReact => &["ts", "tsx", "js", "jsx"][..],
        DetectedProjectType::FrontendFlutter => &["dart"][..],
        DetectedProjectType::Unknown => return,
    };

    let patterns: Vec<&str> = match project_type {
        DetectedProjectType::BackendRust => vec!["std::env::var", "env::var(", "dotenvy", "dotenv"],
        DetectedProjectType::BackendGo => vec!["os.Getenv", "os.LookupEnv"],
        DetectedProjectType::BackendCsharp => vec!["Environment.GetEnvironmentVariable"],
        DetectedProjectType::BackendPython => vec!["os.environ", "os.getenv"],
        DetectedProjectType::FrontendReact => vec!["process.env", "import.meta.env"],
        DetectedProjectType::FrontendFlutter => vec!["Platform.environment", "fromEnvironment"],
        DetectedProjectType::Unknown => return,
    };

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            scan_source_for_env_vars(&entry_path, base_path, project_type, usages);
        } else if entry_path.is_file() {
            let ext = entry_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            if !extensions.contains(&ext) {
                continue;
            }

            let relative_path = entry_path
                .strip_prefix(base_path)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| entry_path.to_string_lossy().to_string());

            let content = match std::fs::read_to_string(&entry_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for (line_num, line) in content.lines().enumerate() {
                for pattern in &patterns {
                    if line.contains(pattern) {
                        usages.push(EnvVarUsage {
                            file: relative_path.clone(),
                            line: line_num + 1,
                            pattern: (*pattern).to_string(),
                            var_name: None,
                        });
                        break;
                    }
                }
            }
        }
    }
}

fn scan_hardcoded_secrets(path: &Path) -> Vec<SecretUsage> {
    let mut secrets = Vec::new();

    let config_dir = path.join("config");
    if !config_dir.exists() {
        return secrets;
    }

    let secret_patterns: &[&str] = &[
        "password", "passwd", "secret", "token", "api_key", "apikey",
        "private_key", "credential", "access_key", "encryption_key",
        "signing_key", "client_secret",
    ];

    let entries = match std::fs::read_dir(&config_dir) {
        Ok(e) => e,
        Err(_) => return secrets,
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if !entry_path.is_file() {
            continue;
        }
        let ext = entry_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if ext != "yaml" && ext != "yml" {
            continue;
        }

        let relative = entry_path
            .strip_prefix(path)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| entry_path.to_string_lossy().to_string());

        let content = match std::fs::read_to_string(&entry_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }

            if let Some(colon_pos) = trimmed.find(':') {
                let key = trimmed[..colon_pos].trim().to_lowercase();
                let value = trimmed[colon_pos + 1..].trim();

                if key.ends_with("_file") || key.ends_with("_path") || key.ends_with("_ref") {
                    continue;
                }

                if let Some(kind) = secret_patterns.iter().find(|p| key.contains(**p)) {
                    if !value.is_empty()
                        && value != "null"
                        && value != "~"
                        && !value.starts_with("${")
                        && !value.starts_with("{{")
                    {
                        secrets.push(SecretUsage {
                            file: relative.clone(),
                            line: line_num + 1,
                            kind: (*kind).to_string(),
                        });
                    }
                }
            }
        }
    }

    secrets
}

fn extract_toml_key(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('[') {
        return None;
    }
    let eq_pos = trimmed.find('=')?;
    let key = trimmed[..eq_pos].trim();
    if key.is_empty() {
        return None;
    }
    Some(key.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parse_cargo_dependencies() {
        let tmp = tempfile::tempdir().expect("tempdir failed");
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n\n[dependencies]\naxum = \"0.8\"\ntokio = { version = \"1\" }\nserde = \"1.0\"\n",
        )
        .expect("write failed");

        let deps = parse_cargo_dependencies(tmp.path());
        assert!(deps.contains(&"axum".to_string()));
        assert!(deps.contains(&"tokio".to_string()));
        assert!(deps.contains(&"serde".to_string()));
    }

    #[test]
    fn test_parse_go_dependencies() {
        let tmp = tempfile::tempdir().expect("tempdir failed");
        fs::write(
            tmp.path().join("go.mod"),
            "module example.com/test\n\ngo 1.21\n\nrequire (\n\tgithub.com/gin-gonic/gin v1.9.0\n\tgoogle.golang.org/grpc v1.60.0\n)\n",
        )
        .expect("write failed");

        let deps = parse_go_dependencies(tmp.path());
        assert!(deps.contains(&"github.com/gin-gonic/gin".to_string()));
        assert!(deps.contains(&"google.golang.org/grpc".to_string()));
    }

    #[test]
    fn test_parse_npm_dependencies() {
        let tmp = tempfile::tempdir().expect("tempdir failed");
        fs::write(
            tmp.path().join("package.json"),
            "{\"dependencies\": {\"react\": \"^18.0.0\", \"axios\": \"^1.0.0\"}}",
        )
        .expect("write failed");

        let deps = parse_npm_dependencies(tmp.path());
        assert!(deps.contains(&"react".to_string()));
        assert!(deps.contains(&"axios".to_string()));
    }

    #[test]
    fn test_detect_env_files() {
        let tmp = tempfile::tempdir().expect("tempdir failed");
        fs::write(tmp.path().join(".env"), "PORT=3000\n").expect("write failed");
        fs::write(tmp.path().join(".env.local"), "DB=test\n").expect("write failed");

        let files = detect_env_files(tmp.path());
        assert!(files.contains(&".env".to_string()));
        assert!(files.contains(&".env.local".to_string()));
    }

    #[test]
    fn test_parse_requirements_txt() {
        let content = "flask>=2.0\nrequests==2.28.0\n# comment\nSQLAlchemy~=2.0\n";
        let deps = extract_python_deps_from_requirements(content);
        assert!(deps.contains(&"flask".to_string()));
        assert!(deps.contains(&"requests".to_string()));
        assert!(deps.contains(&"SQLAlchemy".to_string()));
    }
}
