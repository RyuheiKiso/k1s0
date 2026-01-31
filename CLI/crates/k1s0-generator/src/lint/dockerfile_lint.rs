use std::path::Path;

use super::{LintResult, Linter, RuleId, Severity, Violation};

impl Linter {
    /// Dockerfile ベースイメージ未固定の検査（K060）
    pub(super) fn check_dockerfile_base_image(&self, path: &Path, result: &mut LintResult) {
        let dockerfile_path = path.join("Dockerfile");
        if !dockerfile_path.exists() {
            return;
        }

        let content = match std::fs::read_to_string(&dockerfile_path) {
            Ok(c) => c,
            Err(_) => return,
        };

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // コメント行をスキップ
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }

            // FROM 行のみ対象
            if !trimmed.starts_with("FROM ") {
                continue;
            }

            // "FROM image AS alias" の形式からイメージ部分を抽出
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            let image = parts[1];

            // scratch は特別なイメージなのでスキップ
            if image == "scratch" {
                continue;
            }

            // --platform=... FROM image の場合
            let image = if image.starts_with("--") {
                if parts.len() < 3 {
                    continue;
                }
                parts[2]
            } else {
                image
            };

            // sha256 ダイジェスト指定は OK
            if image.contains("@sha256:") {
                continue;
            }

            // タグなし or :latest は NG
            if let Some(tag_pos) = image.rfind(':') {
                let tag = &image[tag_pos + 1..];
                if tag == "latest" {
                    result.add_violation(
                        Violation::new(
                            RuleId::DockerfileBaseImageUnpinned,
                            Severity::Warning,
                            format!(
                                "ベースイメージ '{}' が :latest タグを使用しています",
                                image
                            ),
                        )
                        .with_path("Dockerfile")
                        .with_line(line_num + 1)
                        .with_hint(
                            "具体的なバージョンタグ（例: image:1.0.0）または sha256 ダイジェストを指定してください。",
                        ),
                    );
                }
                // 具体的なタグが指定されている場合は OK
            } else {
                // タグなし
                result.add_violation(
                    Violation::new(
                        RuleId::DockerfileBaseImageUnpinned,
                        Severity::Warning,
                        format!(
                            "ベースイメージ '{}' にタグが指定されていません",
                            image
                        ),
                    )
                    .with_path("Dockerfile")
                    .with_line(line_num + 1)
                    .with_hint(
                        "具体的なバージョンタグ（例: image:1.0.0）または sha256 ダイジェストを指定してください。",
                    ),
                );
            }
        }
    }
}
