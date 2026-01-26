use std::path::Path;

use crate::manifest::Manifest;

use super::{contains_adr_reference, LintResult, Linter, RuleId, Severity, Violation};

impl Linter {
    /// gRPC リトライ設定を検査（K030/K031/K032）
    pub(super) fn check_retry_usage(
        &self,
        path: &Path,
        result: &mut LintResult,
        check_k030: bool,
        check_k031: bool,
        check_k032: bool,
    ) {
        // manifest から言語を取得
        let manifest_path = path.join(".k1s0/manifest.json");
        let manifest = match Manifest::load(&manifest_path) {
            Ok(m) => m,
            Err(_) => return, // manifest がない場合はスキップ
        };

        // Rust のみ対象（gRPC client は Rust 実装）
        if manifest.service.language != "rust" {
            return;
        }

        let src_path = path.join("src");
        if !src_path.exists() {
            return;
        }

        // ソースファイルを走査
        self.scan_directory_for_retry(&src_path, path, result, check_k030, check_k031, check_k032);
    }

    /// ディレクトリを再帰的に走査してリトライ設定を検出
    fn scan_directory_for_retry(
        &self,
        dir: &Path,
        base_path: &Path,
        result: &mut LintResult,
        check_k030: bool,
        check_k031: bool,
        check_k032: bool,
    ) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                self.scan_directory_for_retry(&entry_path, base_path, result, check_k030, check_k031, check_k032);
            } else if entry_path.is_file() {
                let ext = entry_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext == "rs" {
                    self.check_file_for_retry(&entry_path, base_path, result, check_k030, check_k031, check_k032);
                }
            }
        }
    }

    /// ファイル内のリトライ設定を検査
    fn check_file_for_retry(
        &self,
        file_path: &Path,
        base_path: &Path,
        result: &mut LintResult,
        check_k030: bool,
        check_k031: bool,
        check_k032: bool,
    ) {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return,
        };

        let relative_path = file_path
            .strip_prefix(base_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| file_path.to_string_lossy().to_string());

        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            // RetryConfig::enabled( の検出
            if let Some(pos) = line.find("RetryConfig::enabled(") {
                // K030: リトライ設定の存在を検知（Warning）
                if check_k030 {
                    result.add_violation(
                        Violation::new(
                            RuleId::RetryUsageDetected,
                            Severity::Warning,
                            "gRPC リトライ設定が検出されました",
                        )
                        .with_path(&relative_path)
                        .with_line(line_num + 1)
                        .with_hint("リトライは原則禁止です。冪等性が保証されている操作のみ、ADR での承認を得た上で使用してください。"),
                    );
                }

                // K031: ADR 参照のチェック
                if check_k031 {
                    let has_adr_reference = self.check_adr_reference_in_context(&lines, line_num, pos, line);
                    if !has_adr_reference {
                        result.add_violation(
                            Violation::new(
                                RuleId::RetryWithoutAdr,
                                Severity::Error,
                                "gRPC リトライ設定に ADR 参照がありません",
                            )
                            .with_path(&relative_path)
                            .with_line(line_num + 1)
                            .with_hint("リトライを有効にするには ADR 参照が必須です。例: RetryConfig::enabled(\"ADR-001\")"),
                        );
                    }
                }

                // K032: 必須設定のチェック
                if check_k032 {
                    let has_required_config = self.check_retry_required_config(&lines, line_num);
                    if !has_required_config {
                        result.add_violation(
                            Violation::new(
                                RuleId::RetryConfigIncomplete,
                                Severity::Error,
                                "gRPC リトライ設定に必須パラメータが不足しています",
                            )
                            .with_path(&relative_path)
                            .with_line(line_num + 1)
                            .with_hint("max_attempts() の指定が必要です。"),
                        );
                    }
                }
            }
        }
    }

    /// ADR 参照が存在するかチェック
    fn check_adr_reference_in_context(
        &self,
        lines: &[&str],
        line_num: usize,
        pos: usize,
        current_line: &str,
    ) -> bool {
        // 引数内を抽出（RetryConfig::enabled("ADR-001")）
        let after_paren = &current_line[pos + "RetryConfig::enabled(".len()..];
        if contains_adr_reference(after_paren) {
            return true;
        }

        // 前の2行のコメントも確認
        let start_line = line_num.saturating_sub(2);
        for i in start_line..line_num {
            if let Some(line) = lines.get(i) {
                if contains_adr_reference(line) {
                    return true;
                }
            }
        }

        false
    }

    /// リトライ設定に必須パラメータがあるかチェック
    fn check_retry_required_config(&self, lines: &[&str], start_line: usize) -> bool {
        // RetryConfig::enabled() から .build() までを探索
        let mut has_max_attempts = false;
        let mut has_build = false;

        // 最大10行先まで確認（チェーンメソッド呼び出しを想定）
        let end_line = std::cmp::min(start_line + 10, lines.len());

        for i in start_line..end_line {
            if let Some(line) = lines.get(i) {
                if line.contains(".max_attempts(") {
                    has_max_attempts = true;
                }
                if line.contains(".build()") {
                    has_build = true;
                    break;
                }
                // 文の終わり（セミコロン）を検出したら終了
                // ただし、同じ行に .build() がある場合は上で検出済み
                if line.contains(';') {
                    break;
                }
            }
        }

        // build() が呼ばれているか、max_attempts が指定されているか
        // RetryConfigBuilder::new() はデフォルトで max_attempts = 3 を持つので、
        // build() が呼ばれていれば基本的に OK
        has_build || has_max_attempts
    }
}
