use std::path::Path;

use crate::manifest::Manifest;

use super::{LintResult, Linter, RuleId, Severity, Violation};

/// 言語ごとのパニックパターン
struct PanicPatterns {
    file_extensions: Vec<&'static str>,
    patterns: Vec<&'static str>,
    comment_prefixes: Vec<&'static str>,
    test_file_patterns: Vec<&'static str>,
    entry_points: Vec<&'static str>,
}

impl PanicPatterns {
    fn rust() -> Self {
        Self {
            file_extensions: vec!["rs"],
            patterns: vec![
                ".unwrap()",
                ".expect(",
                "panic!(",
                "todo!(",
                "unimplemented!(",
                "unreachable!(",
            ],
            comment_prefixes: vec!["//", "///"],
            test_file_patterns: vec!["_test.rs"],
            entry_points: vec!["src/main.rs"],
        }
    }

    fn go() -> Self {
        Self {
            file_extensions: vec!["go"],
            patterns: vec!["panic(", "log.Fatal("],
            comment_prefixes: vec!["//"],
            test_file_patterns: vec!["_test.go"],
            entry_points: vec!["cmd/main.go"],
        }
    }

    fn typescript() -> Self {
        Self {
            file_extensions: vec!["ts", "tsx", "js", "jsx"],
            patterns: vec!["process.exit("],
            comment_prefixes: vec!["//"],
            test_file_patterns: vec![".test.ts", ".test.tsx", ".spec.ts", ".spec.tsx", ".test.js", ".spec.js"],
            entry_points: vec![],
        }
    }

    fn csharp() -> Self {
        Self {
            file_extensions: vec!["cs"],
            patterns: vec!["Environment.Exit(", "Environment.FailFast("],
            comment_prefixes: vec!["//"],
            test_file_patterns: vec![],
            entry_points: vec![],
        }
    }

    fn python() -> Self {
        Self {
            file_extensions: vec!["py"],
            patterns: vec!["sys.exit(", "os._exit("],
            comment_prefixes: vec!["#"],
            test_file_patterns: vec!["test_"],
            entry_points: vec![],
        }
    }

    fn dart() -> Self {
        Self {
            file_extensions: vec!["dart"],
            patterns: vec!["exit("],
            comment_prefixes: vec!["//"],
            test_file_patterns: vec!["_test.dart"],
            entry_points: vec![],
        }
    }

    fn kotlin() -> Self {
        Self {
            file_extensions: vec!["kt", "kts"],
            patterns: vec!["!!", "error(", "TODO(", "throw RuntimeException"],
            comment_prefixes: vec!["//"],
            test_file_patterns: vec!["Test.kt", "Spec.kt"],
            entry_points: vec!["Main.kt"],
        }
    }
}

fn is_test_file(file_path: &Path, patterns: &PanicPatterns) -> bool {
    let name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    for pat in &patterns.test_file_patterns {
        if pat.starts_with('.') || pat.starts_with('_') {
            if name.ends_with(pat) {
                return true;
            }
        } else {
            // prefix pattern like "test_"
            if name.starts_with(pat) {
                return true;
            }
        }
    }
    false
}

fn is_entry_point(relative_path: &str, patterns: &PanicPatterns) -> bool {
    let normalized = relative_path.replace('\\', "/");
    patterns.entry_points.iter().any(|ep| normalized == *ep)
}

impl Linter {
    /// 本番コードでのパニック検出（K029）
    pub(super) fn check_panic_in_production_code(&self, path: &Path, result: &mut LintResult) {
        let manifest_path = path.join(".k1s0/manifest.json");
        let manifest = match Manifest::load(&manifest_path) {
            Ok(m) => m,
            Err(_) => return,
        };

        let (src_dir, patterns) = match manifest.service.language.as_str() {
            "rust" => ("src", PanicPatterns::rust()),
            "go" => ("internal", PanicPatterns::go()),
            "typescript" => ("src", PanicPatterns::typescript()),
            "csharp" => ("src", PanicPatterns::csharp()),
            "python" => ("src", PanicPatterns::python()),
            "dart" => ("lib", PanicPatterns::dart()),
            "kotlin" => {
                let src_dir = if manifest.service.service_type == "frontend" {
                    "app/src"
                } else {
                    "src"
                };
                (src_dir, PanicPatterns::kotlin())
            }
            _ => return,
        };

        let src_path = path.join(src_dir);
        if !src_path.exists() {
            return;
        }

        self.scan_directory_for_panic(&src_path, path, &patterns, result);
    }

    fn scan_directory_for_panic(
        &self,
        dir: &Path,
        base_path: &Path,
        patterns: &PanicPatterns,
        result: &mut LintResult,
    ) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                self.scan_directory_for_panic(&entry_path, base_path, patterns, result);
            } else if entry_path.is_file() {
                let ext = entry_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                if patterns.file_extensions.contains(&ext) {
                    self.check_file_for_panic(&entry_path, base_path, patterns, result);
                }
            }
        }
    }

    fn check_file_for_panic(
        &self,
        file_path: &Path,
        base_path: &Path,
        patterns: &PanicPatterns,
        result: &mut LintResult,
    ) {
        if is_test_file(file_path, patterns) {
            return;
        }

        let relative_path = file_path
            .strip_prefix(base_path)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| file_path.to_string_lossy().to_string());

        if is_entry_point(&relative_path, patterns) {
            return;
        }

        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return,
        };

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

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
                            RuleId::PanicInProductionCode,
                            Severity::Error,
                            format!(
                                "本番コードでパニックを起こす可能性があります: '{}'",
                                pattern.trim()
                            ),
                        )
                        .with_path(&relative_path)
                        .with_line(line_num + 1)
                        .with_hint(
                            "Result/Option を適切にハンドリングしてください。unwrap()/expect() の代わりに ? 演算子やパターンマッチを使用してください。",
                        ),
                    );
                    break;
                }
            }
        }
    }
}
