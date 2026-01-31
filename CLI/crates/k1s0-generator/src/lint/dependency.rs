use std::path::Path;

use crate::manifest::Manifest;

use super::{LintResult, Linter, RuleId, Severity, Violation};

/// 依存方向ルールの定義
#[derive(Debug, Clone)]
pub struct DependencyRules {
    /// 対象ファイルの拡張子
    pub file_extensions: Vec<&'static str>,
    /// import パターン（{layer} はプレースホルダ）
    pub import_patterns: Vec<String>,
}

impl DependencyRules {
    /// Rust の依存方向ルール
    pub fn rust() -> Self {
        Self {
            file_extensions: vec!["rs"],
            import_patterns: vec![
                "use crate::{layer}".to_string(),
                "crate::{layer}::".to_string(),
                "super::super::{layer}".to_string(),
            ],
        }
    }

    /// Go の依存方向ルール
    pub fn go() -> Self {
        Self {
            file_extensions: vec!["go"],
            import_patterns: vec![
                "\"internal/{layer}".to_string(),
                "/internal/{layer}".to_string(),
            ],
        }
    }

    /// TypeScript の依存方向ルール
    pub fn typescript() -> Self {
        Self {
            file_extensions: vec!["ts", "tsx", "js", "jsx"],
            import_patterns: vec![
                "from '../{layer}".to_string(),
                "from \"../{layer}".to_string(),
                "from '../../{layer}".to_string(),
                "from \"../../{layer}".to_string(),
                "from '@/{layer}".to_string(),
                "from \"@/{layer}".to_string(),
            ],
        }
    }

    /// Python の依存方向ルール
    pub fn python() -> Self {
        Self {
            file_extensions: vec!["py"],
            import_patterns: vec![
                "from {layer}".to_string(),
                "import {layer}".to_string(),
                "from .{layer}".to_string(),
                "from ..{layer}".to_string(),
            ],
        }
    }

    /// Dart の依存方向ルール
    pub fn dart() -> Self {
        Self {
            file_extensions: vec!["dart"],
            import_patterns: vec![
                "import 'package:".to_string() + "{layer}",
                "import '../{layer}".to_string(),
                "import '../../{layer}".to_string(),
            ],
        }
    }

    /// Kotlin の依存方向ルール
    pub fn kotlin() -> Self {
        Self {
            file_extensions: vec!["kt", "kts"],
            import_patterns: vec![
                "import {layer}.".to_string(),
                "import *.{layer}.".to_string(),
            ],
        }
    }
}

impl Linter {
    /// Clean Architecture 依存方向を検査（K022）
    pub(super) fn check_dependency_direction(&self, path: &Path, result: &mut LintResult) {
        // manifest から言語を取得
        let manifest_path = path.join(".k1s0/manifest.json");
        let manifest = match Manifest::load(&manifest_path) {
            Ok(m) => m,
            Err(_) => return, // manifest がない場合はスキップ
        };

        // 言語に応じたソースディレクトリとパターンを決定
        let (src_dir, rules) = match manifest.service.language.as_str() {
            "rust" => ("src", DependencyRules::rust()),
            "go" => ("internal", DependencyRules::go()),
            "typescript" => ("src", DependencyRules::typescript()),
            "python" => ("src", DependencyRules::python()),
            "dart" => ("lib/src", DependencyRules::dart()),
            "kotlin" => {
                let src_dir = if manifest.service.service_type == "frontend" {
                    "app/src/main/kotlin"
                } else {
                    "src"
                };
                (src_dir, DependencyRules::kotlin())
            }
            _ => return, // 不明な言語の場合はスキップ
        };

        let src_path = path.join(src_dir);
        if !src_path.exists() {
            return;
        }

        // 各層のディレクトリを検査
        for layer in &["domain", "application"] {
            let layer_path = src_path.join(layer);
            if layer_path.exists() && layer_path.is_dir() {
                self.scan_layer_for_violations(&layer_path, path, layer, &rules, result);
            }
        }
    }

    /// 特定の層のディレクトリを走査して依存方向違反を検出
    fn scan_layer_for_violations(
        &self,
        dir: &Path,
        base_path: &Path,
        layer: &str,
        rules: &DependencyRules,
        result: &mut LintResult,
    ) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                // 再帰的に走査
                self.scan_layer_for_violations(&entry_path, base_path, layer, rules, result);
            } else if entry_path.is_file() {
                // ファイルの拡張子をチェック
                let ext = entry_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if rules.file_extensions.contains(&ext) {
                    self.check_file_for_violations(&entry_path, base_path, layer, rules, result);
                }
            }
        }
    }

    /// ファイル内の依存方向違反を検査
    fn check_file_for_violations(
        &self,
        file_path: &Path,
        base_path: &Path,
        layer: &str,
        rules: &DependencyRules,
        result: &mut LintResult,
    ) {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return,
        };

        let relative_path = file_path
            .strip_prefix(base_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| file_path.to_string_lossy().to_string());

        // 層に応じた禁止パターンを取得
        let forbidden_layers = match layer {
            "domain" => vec!["application", "infrastructure", "presentation"],
            "application" => vec!["infrastructure", "presentation"],
            _ => return,
        };

        for (line_num, line) in content.lines().enumerate() {
            // コメント行はスキップ
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("#") || trimmed.starts_with("/*") {
                continue;
            }

            // 各禁止層へのインポートをチェック
            for forbidden in &forbidden_layers {
                for pattern in &rules.import_patterns {
                    let forbidden_pattern = pattern.replace("{layer}", forbidden);
                    if line.contains(&forbidden_pattern) {
                        result.add_violation(
                            Violation::new(
                                RuleId::DependencyDirection,
                                Severity::Error,
                                format!(
                                    "{} 層から {} 層への依存が検出されました",
                                    layer, forbidden
                                ),
                            )
                            .with_path(&relative_path)
                            .with_line(line_num + 1)
                            .with_hint(format!(
                                "Clean Architecture では {} 層は {} 層に依存できません。依存関係を逆転させてください。",
                                layer, forbidden
                            )),
                        );
                    }
                }
            }
        }
    }
}
