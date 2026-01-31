use std::path::Path;

use crate::manifest::Manifest;

use super::{LintResult, Linter, RuleId, Severity, Violation};

/// kebab-case を snake_case に変換
fn kebab_to_snake(s: &str) -> String {
    s.replace('-', "_")
}

/// kebab-case を PascalCase に変換
fn kebab_to_pascal(s: &str) -> String {
    s.split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

/// 言語ごとの import パターンで domain 使用を検索
fn domain_is_used_in_file(content: &str, domain_name: &str, language: &str) -> bool {
    let snake = kebab_to_snake(domain_name);
    let pascal = kebab_to_pascal(domain_name);

    for line in content.lines() {
        let trimmed = line.trim();
        let used = match language {
            "rust" => {
                trimmed.contains(&format!("use {}", snake))
                    || trimmed.contains(&format!("{}::", snake))
            }
            "go" => {
                trimmed.contains(&format!("\"{}\"", domain_name))
                    || trimmed.contains(&format!("/{}", domain_name))
            }
            "typescript" => {
                trimmed.contains(&format!("from '{}'", domain_name))
                    || trimmed.contains(&format!("from \"{}\"", domain_name))
                    || trimmed.contains(&format!("/{}'", domain_name))
                    || trimmed.contains(&format!("/{}\"", domain_name))
            }
            "csharp" => trimmed.contains(&format!("using {}", pascal)) || trimmed.contains(&pascal),
            "python" => {
                trimmed.contains(&format!("import {}", snake))
                    || trimmed.contains(&format!("from {}", snake))
            }
            "dart" => trimmed.contains(&format!("package:{}/", domain_name)),
            _ => false,
        };
        if used {
            return true;
        }
    }
    false
}

impl Linter {
    /// 未使用 domain 依存の検査（K028）
    pub(super) fn check_unused_domain_dependency(&self, path: &Path, result: &mut LintResult) {
        let manifest_path = path.join(".k1s0/manifest.json");
        let manifest = match Manifest::load(&manifest_path) {
            Ok(m) => m,
            Err(_) => return,
        };

        let domain_deps = match &manifest.dependencies {
            Some(deps) => match &deps.domain {
                Some(d) => d.clone(),
                None => return,
            },
            None => return,
        };

        if domain_deps.is_empty() {
            return;
        }

        let language = &manifest.service.language;
        let src_dir = match language.as_str() {
            "rust" => "src",
            "go" => "internal",
            "typescript" => "src",
            "csharp" => "src",
            "python" => "src",
            "dart" => "lib",
            _ => return,
        };

        let src_path = path.join(src_dir);
        if !src_path.exists() {
            return;
        }

        // src/ 配下の全ソースファイルの内容を結合
        let all_content = self.collect_source_content(&src_path, language);

        for domain_name in domain_deps.keys() {
            if !domain_is_used_in_file(&all_content, domain_name, language) {
                result.add_violation(
                    Violation::new(
                        RuleId::UnusedDomainDependency,
                        Severity::Warning,
                        format!(
                            "domain '{}' が dependencies.domain に宣言されていますがコードで使用されていません",
                            domain_name
                        ),
                    )
                    .with_path(".k1s0/manifest.json")
                    .with_hint(
                        "不要な domain 依存を削除するか、コードで import/use してください。",
                    ),
                );
            }
        }
    }

    fn collect_source_content(&self, dir: &Path, language: &str) -> String {
        let extensions: &[&str] = match language {
            "rust" => &["rs"],
            "go" => &["go"],
            "typescript" => &["ts", "tsx", "js", "jsx"],
            "csharp" => &["cs"],
            "python" => &["py"],
            "dart" => &["dart"],
            _ => return String::new(),
        };

        let mut content = String::new();
        self.collect_files_recursive(dir, extensions, &mut content);
        content
    }

    fn collect_files_recursive(&self, dir: &Path, extensions: &[&str], content: &mut String) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                self.collect_files_recursive(&entry_path, extensions, content);
            } else if entry_path.is_file() {
                let ext = entry_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                if extensions.contains(&ext) {
                    if let Ok(c) = std::fs::read_to_string(&entry_path) {
                        content.push_str(&c);
                        content.push('\n');
                    }
                }
            }
        }
    }
}
