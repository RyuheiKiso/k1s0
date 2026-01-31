use std::path::Path;

use crate::manifest::Manifest;

use super::{LintResult, Linter, RuleId, Severity, Violation};

#[cfg(feature = "ast")]
use super::ast::parser::Language;
#[cfg(feature = "ast")]
use super::ast::query::QueryId;
#[cfg(feature = "ast")]
use super::ast::{AstContext, ParserPool, QueryCache};

/// 環境変数パターンの定義
#[derive(Debug, Clone)]
pub struct EnvVarPattern {
    /// 検出対象のパターン文字列
    pub pattern: &'static str,
    /// 検出時に表示するヒント
    pub hint: String,
}

/// 言語ごとの環境変数パターン
#[derive(Debug, Clone)]
pub struct EnvVarPatterns {
    /// 対象ファイルの拡張子
    pub file_extensions: Vec<&'static str>,
    /// 検出パターン
    pub patterns: Vec<EnvVarPattern>,
}

impl EnvVarPatterns {
    /// Rust の環境変数パターン
    pub fn rust() -> Self {
        Self {
            file_extensions: vec!["rs"],
            patterns: vec![
                EnvVarPattern {
                    pattern: "std::env::var",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "std::env::var_os",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "std::env::vars",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "std::env::vars_os",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "std::env::set_var",
                    hint: "環境変数の設定は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "std::env::remove_var",
                    hint: "環境変数の削除は禁止されています。".to_string(),
                },
                EnvVarPattern {
                    pattern: "env::var(",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "env::var_os(",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "env::vars(",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "env::set_var(",
                    hint: "環境変数の設定は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "env::remove_var(",
                    hint: "環境変数の削除は禁止されています。".to_string(),
                },
                EnvVarPattern {
                    pattern: "dotenv",
                    hint: "dotenv の使用は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "dotenvy",
                    hint: "dotenvy の使用は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
            ],
        }
    }

    /// Go の環境変数パターン
    pub fn go() -> Self {
        Self {
            file_extensions: vec!["go"],
            patterns: vec![
                EnvVarPattern {
                    pattern: "os.Getenv",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config パッケージを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "os.LookupEnv",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config パッケージを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "os.Setenv",
                    hint: "環境変数の設定は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "os.Unsetenv",
                    hint: "環境変数の削除は禁止されています。".to_string(),
                },
                EnvVarPattern {
                    pattern: "os.Environ",
                    hint: "環境変数の一覧取得は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "godotenv",
                    hint: "godotenv の使用は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
            ],
        }
    }

    /// TypeScript の環境変数パターン
    pub fn typescript() -> Self {
        Self {
            file_extensions: vec!["ts", "tsx", "js", "jsx"],
            patterns: vec![
                EnvVarPattern {
                    pattern: "process.env",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "import.meta.env",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "dotenv",
                    hint: "dotenv の使用は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
            ],
        }
    }

    /// C# の環境変数パターン
    pub fn csharp() -> Self {
        Self {
            file_extensions: vec!["cs"],
            patterns: vec![
                EnvVarPattern {
                    pattern: "Environment.GetEnvironmentVariable",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "Environment.GetEnvironmentVariables",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "Environment.ExpandEnvironmentVariables",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: ".AddEnvironmentVariables(",
                    hint: "AddEnvironmentVariables の使用は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
            ],
        }
    }

    /// Python の環境変数パターン
    pub fn python() -> Self {
        Self {
            file_extensions: vec!["py"],
            patterns: vec![
                EnvVarPattern {
                    pattern: "os.environ",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "os.getenv",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "os.putenv",
                    hint: "環境変数の設定は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "os.unsetenv",
                    hint: "環境変数の削除は禁止されています。".to_string(),
                },
                EnvVarPattern {
                    pattern: "load_dotenv",
                    hint: "dotenv の使用は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "from dotenv",
                    hint: "dotenv の使用は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "import dotenv",
                    hint: "dotenv の使用は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
            ],
        }
    }

    /// Kotlin の環境変数パターン
    pub fn kotlin() -> Self {
        Self {
            file_extensions: vec!["kt", "kts"],
            patterns: vec![
                EnvVarPattern {
                    pattern: "System.getenv",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "System.getProperty",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "ProcessBuilder",
                    hint: "外部プロセス実行は禁止されています。".to_string(),
                },
                EnvVarPattern {
                    pattern: "dotenv",
                    hint: "dotenv の使用は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "BuildConfig.",
                    hint: "BuildConfig の使用は禁止されています。config/{env}.yaml を使用してください。".to_string(),
                },
            ],
        }
    }

    /// Dart の環境変数パターン
    pub fn dart() -> Self {
        Self {
            file_extensions: vec!["dart"],
            patterns: vec![
                EnvVarPattern {
                    pattern: "Platform.environment",
                    hint: "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "fromEnvironment",
                    hint: "コンパイル時環境変数の使用は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
                EnvVarPattern {
                    pattern: "flutter_dotenv",
                    hint: "flutter_dotenv の使用は禁止されています。config/{env}.yaml で設定を管理してください。".to_string(),
                },
            ],
        }
    }
}

impl Linter {
    /// 環境変数参照を検査（K020）
    pub(super) fn check_env_var_usage(&self, path: &Path, result: &mut LintResult) {
        // manifest から言語を取得
        let manifest_path = path.join(".k1s0/manifest.json");
        let manifest = match Manifest::load(&manifest_path) {
            Ok(m) => m,
            Err(_) => return, // manifest がない場合はスキップ
        };

        // 言語に応じたソースディレクトリとパターンを決定
        let (src_dir, patterns) = match manifest.service.language.as_str() {
            "rust" => ("src", EnvVarPatterns::rust()),
            "go" => ("internal", EnvVarPatterns::go()),
            "typescript" => ("src", EnvVarPatterns::typescript()),
            "csharp" => ("src", EnvVarPatterns::csharp()),
            "python" => ("src", EnvVarPatterns::python()),
            "dart" => ("lib", EnvVarPatterns::dart()),
            "kotlin" => {
                let src_dir = if manifest.service.service_type == "frontend" {
                    "app/src"
                } else {
                    "src"
                };
                (src_dir, EnvVarPatterns::kotlin())
            }
            _ => return, // 不明な言語の場合はスキップ
        };

        let src_path = path.join(src_dir);
        if !src_path.exists() {
            return;
        }

        // AST モード（fast でない場合）
        #[cfg(feature = "ast")]
        if !self.is_fast_mode() {
            let mut pool = ParserPool::new();
            let mut cache = QueryCache::new();
            self.scan_directory_for_env_vars_ast(
                &src_path, path, &patterns, &mut pool, &mut cache, result,
            );
            return;
        }

        // grep フォールバック
        self.scan_directory_for_env_vars(&src_path, path, &patterns, result);
    }

    /// ディレクトリを再帰的に走査して環境変数参照を検出
    fn scan_directory_for_env_vars(
        &self,
        dir: &Path,
        base_path: &Path,
        patterns: &EnvVarPatterns,
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
                self.scan_directory_for_env_vars(&entry_path, base_path, patterns, result);
            } else if entry_path.is_file() {
                // ファイルの拡張子をチェック
                let ext = entry_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if patterns.file_extensions.contains(&ext) {
                    self.check_file_for_env_vars(&entry_path, base_path, patterns, result);
                }
            }
        }
    }

    /// ファイル内の環境変数参照を検査
    fn check_file_for_env_vars(
        &self,
        file_path: &Path,
        base_path: &Path,
        patterns: &EnvVarPatterns,
        result: &mut LintResult,
    ) {
        // allowlist チェック
        let relative_path = file_path
            .strip_prefix(base_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| file_path.to_string_lossy().to_string());

        if self.is_path_in_allowlist(&relative_path) {
            return;
        }

        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return,
        };

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &patterns.patterns {
                if line.contains(pattern.pattern) {
                    result.add_violation(
                        Violation::new(
                            RuleId::EnvVarUsage,
                            Severity::Error,
                            format!("環境変数参照 '{}' が検出されました", pattern.pattern),
                        )
                        .with_path(&relative_path)
                        .with_line(line_num + 1)
                        .with_hint(&pattern.hint),
                    );
                }
            }
        }
    }

    /// AST ベースのディレクトリ走査（K020）
    #[cfg(feature = "ast")]
    fn scan_directory_for_env_vars_ast(
        &self,
        dir: &Path,
        base_path: &Path,
        patterns: &EnvVarPatterns,
        pool: &mut ParserPool,
        cache: &mut QueryCache,
        result: &mut LintResult,
    ) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                self.scan_directory_for_env_vars_ast(
                    &entry_path, base_path, patterns, pool, cache, result,
                );
            } else if entry_path.is_file() {
                let ext = entry_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                if patterns.file_extensions.contains(&ext) {
                    self.check_file_for_env_vars_ast(
                        &entry_path, base_path, patterns, pool, cache, result,
                    );
                }
            }
        }
    }

    /// AST ベースの環境変数使用検出（K020）
    #[cfg(feature = "ast")]
    fn check_file_for_env_vars_ast(
        &self,
        file_path: &Path,
        base_path: &Path,
        patterns: &EnvVarPatterns,
        pool: &mut ParserPool,
        cache: &mut QueryCache,
        result: &mut LintResult,
    ) {
        let relative_path = file_path
            .strip_prefix(base_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| file_path.to_string_lossy().to_string());

        if self.is_path_in_allowlist(&relative_path) {
            return;
        }

        let lang = match Language::from_path(file_path) {
            Some(l) => l,
            None => return,
        };

        let content = match std::fs::read(file_path) {
            Ok(c) => c,
            Err(_) => return,
        };

        let ctx = match AstContext::parse(pool, lang, &content) {
            Some(c) => c,
            None => return,
        };

        let query_source = match lang {
            Language::Rust => super::ast::languages::rust::ENV_VAR_QUERY,
            Language::Go => super::ast::languages::go::ENV_VAR_QUERY,
            Language::TypeScript => super::ast::languages::typescript::ENV_VAR_QUERY,
            Language::Python => super::ast::languages::python::ENV_VAR_QUERY,
            Language::CSharp => super::ast::languages::csharp::ENV_VAR_QUERY,
            Language::Kotlin => super::ast::languages::kotlin::ENV_VAR_QUERY,
        };

        ctx.query_matches(
            cache,
            QueryId::EnvVarUsage,
            query_source,
            |node, capture_name| {
                if capture_name != "match" {
                    return;
                }
                if ctx.is_non_code(node) {
                    return;
                }

                let text = ctx.node_text(node);
                let line = ctx.node_line(node);

                // grep パターンとマッチするものを探してヒントを取得
                let hint = patterns
                    .patterns
                    .iter()
                    .find(|p| text.contains(p.pattern))
                    .map(|p| p.hint.as_str())
                    .unwrap_or(
                        "config/{env}.yaml で設定を管理してください。framework の config モジュールを使用してください。",
                    );

                let matched_pattern = patterns
                    .patterns
                    .iter()
                    .find(|p| text.contains(p.pattern))
                    .map(|p| p.pattern)
                    .unwrap_or(text.trim());

                result.add_violation(
                    Violation::new(
                        RuleId::EnvVarUsage,
                        Severity::Error,
                        format!("環境変数参照 '{}' が検出されました", matched_pattern),
                    )
                    .with_path(&relative_path)
                    .with_line(line)
                    .with_hint(hint),
                );
            },
        );
    }

    /// パスが allowlist に含まれるかチェック
    fn is_path_in_allowlist(&self, path: &str) -> bool {
        // パス区切り文字を統一（Windows 対応）
        let normalized_path = path.replace('\\', "/");

        for pattern in self.env_var_allowlist() {
            let normalized_pattern = pattern.replace('\\', "/");

            // 単純なワイルドカードマッチング
            if normalized_pattern.ends_with('*') {
                let prefix = &normalized_pattern[..normalized_pattern.len() - 1];
                if normalized_path.starts_with(prefix) {
                    return true;
                }
            } else if normalized_pattern == normalized_path {
                return true;
            }
        }
        false
    }
}
