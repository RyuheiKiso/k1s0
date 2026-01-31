use std::path::Path;

use crate::manifest::Manifest;

use super::{LintResult, Linter, RuleId, Severity, Violation};

#[cfg(feature = "ast")]
use super::ast::parser::Language;
#[cfg(feature = "ast")]
use super::ast::query::QueryId;
#[cfg(feature = "ast")]
use super::ast::{AstContext, ParserPool, QueryCache};

/// 言語ごとのプロトコル依存パターン
struct ProtocolPatterns {
    file_extensions: Vec<&'static str>,
    patterns: Vec<&'static str>,
    comment_prefixes: Vec<&'static str>,
}

impl ProtocolPatterns {
    fn rust() -> Self {
        Self {
            file_extensions: vec!["rs"],
            patterns: vec![
                "StatusCode::",
                "tonic::Code::",
                "tonic::Status::",
                "axum::http::StatusCode",
                "hyper::StatusCode",
            ],
            comment_prefixes: vec!["//", "///"],
        }
    }

    fn go() -> Self {
        Self {
            file_extensions: vec!["go"],
            patterns: vec![
                "http.Status",
                "codes.",
                "status.New",
                "status.Error",
            ],
            comment_prefixes: vec!["//"],
        }
    }

    fn typescript() -> Self {
        Self {
            file_extensions: vec!["ts", "tsx", "js", "jsx"],
            patterns: vec![
                "HttpStatus",
                "StatusCodes",
                "grpc.status.",
            ],
            comment_prefixes: vec!["//"],
        }
    }

    fn csharp() -> Self {
        Self {
            file_extensions: vec!["cs"],
            patterns: vec![
                "HttpStatusCode.",
                "StatusCode.",
                "Grpc.Core.StatusCode",
            ],
            comment_prefixes: vec!["//"],
        }
    }

    fn python() -> Self {
        Self {
            file_extensions: vec!["py"],
            patterns: vec![
                "status.HTTP_",
                "grpc.StatusCode",
                "HTTPStatus",
            ],
            comment_prefixes: vec!["#"],
        }
    }

    fn dart() -> Self {
        Self {
            file_extensions: vec!["dart"],
            patterns: vec![
                "HttpStatus.",
                "GrpcError.",
            ],
            comment_prefixes: vec!["//"],
        }
    }

    fn kotlin() -> Self {
        Self {
            file_extensions: vec!["kt", "kts"],
            patterns: vec![
                "HttpStatusCode.",
                "io.ktor.http.HttpStatusCode",
                "io.grpc.Status",
            ],
            comment_prefixes: vec!["//"],
        }
    }
}

impl Linter {
    /// Domain 層でのプロトコル依存を検査（K026）
    pub(super) fn check_protocol_dependency_in_domain(
        &self,
        path: &Path,
        result: &mut LintResult,
    ) {
        let manifest_path = path.join(".k1s0/manifest.json");
        let manifest = match Manifest::load(&manifest_path) {
            Ok(m) => m,
            Err(_) => return,
        };

        let (domain_dir, patterns) = match manifest.service.language.as_str() {
            "rust" => ("src/domain", ProtocolPatterns::rust()),
            "go" => ("internal/domain", ProtocolPatterns::go()),
            "typescript" => ("src/domain", ProtocolPatterns::typescript()),
            "csharp" => {
                let pascal_name = to_pascal_case(&manifest.service.service_name);
                let domain_dir = format!("src/{}.Domain", pascal_name);
                let domain_path = path.join(&domain_dir);
                if !domain_path.exists() {
                    return;
                }
                let patterns = ProtocolPatterns::csharp();
                #[cfg(feature = "ast")]
                if !self.is_fast_mode() {
                    let mut pool = ParserPool::new();
                    let mut cache = QueryCache::new();
                    self.scan_domain_for_protocol_ast(
                        &domain_path, path, &patterns, &mut pool, &mut cache, result,
                    );
                    return;
                }
                self.scan_domain_for_protocol(
                    &domain_path,
                    path,
                    &patterns,
                    result,
                );
                return;
            }
            "python" => {
                let snake_name = manifest.service.service_name.replace('-', "_");
                let domain_dir = format!("src/{}/domain", snake_name);
                let domain_path = path.join(&domain_dir);
                if !domain_path.exists() {
                    return;
                }
                let patterns = ProtocolPatterns::python();
                #[cfg(feature = "ast")]
                if !self.is_fast_mode() {
                    let mut pool = ParserPool::new();
                    let mut cache = QueryCache::new();
                    self.scan_domain_for_protocol_ast(
                        &domain_path, path, &patterns, &mut pool, &mut cache, result,
                    );
                    return;
                }
                self.scan_domain_for_protocol(
                    &domain_path,
                    path,
                    &patterns,
                    result,
                );
                return;
            }
            "dart" => ("lib/src/domain", ProtocolPatterns::dart()),
            "kotlin" => {
                let domain_dir = if manifest.service.service_type == "frontend" {
                    "app/src/main/kotlin/domain"
                } else {
                    "src/domain"
                };
                (domain_dir, ProtocolPatterns::kotlin())
            }
            _ => return,
        };

        let domain_path = path.join(domain_dir);
        if !domain_path.exists() {
            return;
        }

        // AST モード（fast でない場合）
        #[cfg(feature = "ast")]
        if !self.is_fast_mode() {
            let mut pool = ParserPool::new();
            let mut cache = QueryCache::new();
            self.scan_domain_for_protocol_ast(
                &domain_path, path, &patterns, &mut pool, &mut cache, result,
            );
            return;
        }

        // grep フォールバック
        self.scan_domain_for_protocol(&domain_path, path, &patterns, result);
    }

    fn scan_domain_for_protocol(
        &self,
        dir: &Path,
        base_path: &Path,
        patterns: &ProtocolPatterns,
        result: &mut LintResult,
    ) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                self.scan_domain_for_protocol(&entry_path, base_path, patterns, result);
            } else if entry_path.is_file() {
                let ext = entry_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                if patterns.file_extensions.contains(&ext) {
                    self.check_file_for_protocol(&entry_path, base_path, patterns, result);
                }
            }
        }
    }

    fn check_file_for_protocol(
        &self,
        file_path: &Path,
        base_path: &Path,
        patterns: &ProtocolPatterns,
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
                            RuleId::ProtocolDependencyInDomain,
                            Severity::Error,
                            format!(
                                "Domain 層でプロトコル固有の型 '{}' が使用されています",
                                pattern
                            ),
                        )
                        .with_path(&relative_path)
                        .with_line(line_num + 1)
                        .with_hint(
                            "Domain 層は HTTP/gRPC などのプロトコルに依存すべきではありません。ドメイン固有のエラー型を定義してください。",
                        ),
                    );
                }
            }
        }
    }

    /// AST ベースのドメイン走査（K026）
    #[cfg(feature = "ast")]
    fn scan_domain_for_protocol_ast(
        &self,
        dir: &Path,
        base_path: &Path,
        patterns: &ProtocolPatterns,
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
                self.scan_domain_for_protocol_ast(
                    &entry_path, base_path, patterns, pool, cache, result,
                );
            } else if entry_path.is_file() {
                let ext = entry_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                if patterns.file_extensions.contains(&ext) {
                    self.check_file_for_protocol_ast(
                        &entry_path, base_path, patterns, pool, cache, result,
                    );
                }
            }
        }
    }

    /// AST ベースのプロトコル依存検出（K026）
    #[cfg(feature = "ast")]
    fn check_file_for_protocol_ast(
        &self,
        file_path: &Path,
        base_path: &Path,
        patterns: &ProtocolPatterns,
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
            Language::Rust => super::ast::languages::rust::PROTOCOL_DEPENDENCY_QUERY,
            Language::Go => super::ast::languages::go::PROTOCOL_DEPENDENCY_QUERY,
            Language::TypeScript => super::ast::languages::typescript::PROTOCOL_DEPENDENCY_QUERY,
            Language::Python => super::ast::languages::python::PROTOCOL_DEPENDENCY_QUERY,
            Language::CSharp => super::ast::languages::csharp::PROTOCOL_DEPENDENCY_QUERY,
            Language::Kotlin => super::ast::languages::kotlin::PROTOCOL_DEPENDENCY_QUERY,
        };

        ctx.query_matches(
            cache,
            QueryId::ProtocolDependency,
            query_source,
            |node, _capture_name| {
                if ctx.is_non_code(node) {
                    return;
                }

                let text = ctx.node_text(node);

                // grep パターンのいずれかにマッチするか確認
                let matched_pattern = patterns
                    .patterns
                    .iter()
                    .find(|p| text.contains(*p));

                if let Some(pattern) = matched_pattern {
                    let line = ctx.node_line(node);
                    result.add_violation(
                        Violation::new(
                            RuleId::ProtocolDependencyInDomain,
                            Severity::Error,
                            format!(
                                "Domain 層でプロトコル固有の型 '{}' が使用されています",
                                pattern
                            ),
                        )
                        .with_path(&relative_path)
                        .with_line(line)
                        .with_hint(
                            "Domain 層は HTTP/gRPC などのプロトコルに依存すべきではありません。ドメイン固有のエラー型を定義してください。",
                        ),
                    );
                }
            },
        );
    }
}

/// kebab-case を PascalCase に変換
fn to_pascal_case(s: &str) -> String {
    s.split('-')
        .map(|part| {
            let mut c = part.chars();
            match c.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().to_string() + c.as_str(),
            }
        })
        .collect()
}
