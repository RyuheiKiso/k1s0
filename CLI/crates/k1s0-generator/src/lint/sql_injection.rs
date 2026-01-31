use std::path::Path;

use crate::manifest::Manifest;

use super::{LintResult, Linter, RuleId, Severity, Violation};

#[cfg(feature = "ast")]
use super::ast::parser::Language;
#[cfg(feature = "ast")]
use super::ast::query::QueryId;
#[cfg(feature = "ast")]
use super::ast::{AstContext, ParserPool, QueryCache};

/// 言語ごとの SQL インジェクションパターン
struct SqlInjectionPatterns {
    file_extensions: Vec<&'static str>,
    patterns: Vec<&'static str>,
    comment_prefixes: Vec<&'static str>,
}

impl SqlInjectionPatterns {
    fn rust() -> Self {
        Self {
            file_extensions: vec!["rs"],
            patterns: vec![
                "format!(\"SELECT ",
                "format!(\"INSERT ",
                "format!(\"UPDATE ",
                "format!(\"DELETE ",
                "format!(\"select ",
                "format!(\"insert ",
                "format!(\"update ",
                "format!(\"delete ",
            ],
            comment_prefixes: vec!["//", "///"],
        }
    }

    fn go() -> Self {
        Self {
            file_extensions: vec!["go"],
            patterns: vec![
                "fmt.Sprintf(\"SELECT ",
                "fmt.Sprintf(\"select ",
                "\"SELECT \" +",
                "\"INSERT \" +",
                "\"UPDATE \" +",
                "\"DELETE \" +",
                "\"select \" +",
                "\"insert \" +",
                "\"update \" +",
                "\"delete \" +",
            ],
            comment_prefixes: vec!["//"],
        }
    }

    fn typescript() -> Self {
        Self {
            file_extensions: vec!["ts", "tsx", "js", "jsx"],
            patterns: vec![
                "`SELECT ${",
                "`INSERT ${",
                "`UPDATE ${",
                "`DELETE ${",
                "`select ${",
                "`insert ${",
                "`update ${",
                "`delete ${",
                "\"SELECT \" +",
                "\"INSERT \" +",
                "\"UPDATE \" +",
                "\"DELETE \" +",
            ],
            comment_prefixes: vec!["//"],
        }
    }

    fn csharp() -> Self {
        Self {
            file_extensions: vec!["cs"],
            patterns: vec![
                "$\"SELECT ",
                "$\"INSERT ",
                "$\"UPDATE ",
                "$\"DELETE ",
                "$\"select ",
                "$\"insert ",
                "$\"update ",
                "$\"delete ",
                "\"SELECT \" +",
                "\"INSERT \" +",
                "\"UPDATE \" +",
                "\"DELETE \" +",
            ],
            comment_prefixes: vec!["//"],
        }
    }

    fn python() -> Self {
        Self {
            file_extensions: vec!["py"],
            patterns: vec![
                "f\"SELECT ",
                "f\"INSERT ",
                "f\"UPDATE ",
                "f\"DELETE ",
                "f\"select ",
                "f\"insert ",
                "f\"update ",
                "f\"delete ",
                "f'SELECT ",
                "f'INSERT ",
                "f'UPDATE ",
                "f'DELETE ",
                "\"SELECT \" +",
                "\"INSERT \" +",
                "\"UPDATE \" +",
                "\"DELETE \" +",
                "\"SELECT \".format(",
                "\"INSERT \".format(",
                "\"UPDATE \".format(",
                "\"DELETE \".format(",
            ],
            comment_prefixes: vec!["#"],
        }
    }

    fn dart() -> Self {
        Self {
            file_extensions: vec!["dart"],
            patterns: vec![
                "'SELECT $",
                "\"SELECT $",
                "'INSERT $",
                "\"INSERT $",
                "'UPDATE $",
                "\"UPDATE $",
                "'DELETE $",
                "\"DELETE $",
            ],
            comment_prefixes: vec!["//"],
        }
    }

    fn kotlin() -> Self {
        Self {
            file_extensions: vec!["kt", "kts"],
            patterns: vec![
                "\"SELECT $",
                "\"INSERT $",
                "\"UPDATE $",
                "\"DELETE $",
                "\"select $",
                "\"insert $",
                "\"update $",
                "\"delete $",
                "\"SELECT \" +",
                "\"INSERT \" +",
                "\"UPDATE \" +",
                "\"DELETE \" +",
            ],
            comment_prefixes: vec!["//"],
        }
    }
}

impl Linter {
    /// SQL インジェクションリスクを検査（K050）
    pub(super) fn check_sql_injection_risk(&self, path: &Path, result: &mut LintResult) {
        let manifest_path = path.join(".k1s0/manifest.json");
        let manifest = match Manifest::load(&manifest_path) {
            Ok(m) => m,
            Err(_) => return,
        };

        let (src_dir, patterns) = match manifest.service.language.as_str() {
            "rust" => ("src", SqlInjectionPatterns::rust()),
            "go" => ("internal", SqlInjectionPatterns::go()),
            "typescript" => ("src", SqlInjectionPatterns::typescript()),
            "csharp" => ("src", SqlInjectionPatterns::csharp()),
            "python" => ("src", SqlInjectionPatterns::python()),
            "dart" => ("lib", SqlInjectionPatterns::dart()),
            "kotlin" => {
                let src_dir = if manifest.service.service_type == "frontend" {
                    "app/src"
                } else {
                    "src"
                };
                (src_dir, SqlInjectionPatterns::kotlin())
            }
            _ => return,
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
            self.scan_directory_for_sql_injection_ast(
                &src_path, path, &patterns, &mut pool, &mut cache, result,
            );
            return;
        }

        // grep フォールバック
        self.scan_directory_for_sql_injection(&src_path, path, &patterns, result);
    }

    fn scan_directory_for_sql_injection(
        &self,
        dir: &Path,
        base_path: &Path,
        patterns: &SqlInjectionPatterns,
        result: &mut LintResult,
    ) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                self.scan_directory_for_sql_injection(&entry_path, base_path, patterns, result);
            } else if entry_path.is_file() {
                let ext = entry_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                if patterns.file_extensions.contains(&ext) {
                    self.check_file_for_sql_injection(&entry_path, base_path, patterns, result);
                }
            }
        }
    }

    fn check_file_for_sql_injection(
        &self,
        file_path: &Path,
        base_path: &Path,
        patterns: &SqlInjectionPatterns,
        result: &mut LintResult,
    ) {
        let relative_path = file_path
            .strip_prefix(base_path)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| file_path.to_string_lossy().to_string());

        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return,
        };

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // コメント行をスキップ
            if patterns
                .comment_prefixes
                .iter()
                .any(|prefix| trimmed.starts_with(prefix))
            {
                continue;
            }

            for pattern in &patterns.patterns {
                if line.contains(pattern) {
                    result.add_violation(
                        Violation::new(
                            RuleId::SqlInjectionRisk,
                            Severity::Error,
                            format!(
                                "SQL インジェクションのリスクがあります: '{}'",
                                pattern.trim()
                            ),
                        )
                        .with_path(&relative_path)
                        .with_line(line_num + 1)
                        .with_hint(
                            "文字列補間による SQL 構築は禁止されています。パラメータバインド（$1, ?, @param 等）を使用してください。",
                        ),
                    );
                    break; // 同じ行で複数パターンマッチしても1つだけ報告
                }
            }
        }
    }

    /// AST ベースのディレクトリ走査（K050）
    #[cfg(feature = "ast")]
    fn scan_directory_for_sql_injection_ast(
        &self,
        dir: &Path,
        base_path: &Path,
        patterns: &SqlInjectionPatterns,
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
                self.scan_directory_for_sql_injection_ast(
                    &entry_path, base_path, patterns, pool, cache, result,
                );
            } else if entry_path.is_file() {
                let ext = entry_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                if patterns.file_extensions.contains(&ext) {
                    self.check_file_for_sql_injection_ast(
                        &entry_path, base_path, pool, cache, result,
                    );
                }
            }
        }
    }

    /// AST ベースの SQL インジェクション検出（K050）
    #[cfg(feature = "ast")]
    fn check_file_for_sql_injection_ast(
        &self,
        file_path: &Path,
        base_path: &Path,
        pool: &mut ParserPool,
        cache: &mut QueryCache,
        result: &mut LintResult,
    ) {
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

        let relative_path = file_path
            .strip_prefix(base_path)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| file_path.to_string_lossy().to_string());

        let query_source = match lang {
            Language::Rust => super::ast::languages::rust::SQL_INJECTION_QUERY,
            Language::Go => super::ast::languages::go::SQL_INJECTION_QUERY,
            Language::TypeScript => super::ast::languages::typescript::SQL_INJECTION_QUERY,
            Language::Python => super::ast::languages::python::SQL_INJECTION_QUERY,
            Language::CSharp => super::ast::languages::csharp::SQL_INJECTION_QUERY,
            Language::Kotlin => super::ast::languages::kotlin::SQL_INJECTION_QUERY,
        };

        ctx.query_matches(
            cache,
            QueryId::SqlInjection,
            query_source,
            |node, capture_name| {
                // @match キャプチャのみ処理（重複防止）
                if capture_name != "match" {
                    return;
                }
                if ctx.is_non_code(node) {
                    return;
                }

                let text = ctx.node_text(node);
                // SQL キーワードを含むかチェック（大文字小文字不問）
                let upper = text.to_uppercase();
                let has_sql = upper.contains("SELECT ")
                    || upper.contains("INSERT ")
                    || upper.contains("UPDATE ")
                    || upper.contains("DELETE ")
                    || upper.contains("DROP ")
                    || upper.contains("ALTER ");

                if !has_sql {
                    return;
                }

                // 補間・変数展開を含むかチェック
                let has_interpolation = text.contains('$')
                    || text.contains('{')
                    || text.contains('+')
                    || text.contains("format!");

                if !has_interpolation {
                    return;
                }

                let line = ctx.node_line(node);
                result.add_violation(
                    Violation::new(
                        RuleId::SqlInjectionRisk,
                        Severity::Error,
                        "SQL インジェクションのリスクがあります".to_string(),
                    )
                    .with_path(&relative_path)
                    .with_line(line)
                    .with_hint(
                        "文字列補間による SQL 構築は禁止されています。パラメータバインド（$1, ?, @param 等）を使用してください。",
                    ),
                );
            },
        );
    }
}
