use similar::{ChangeTag, TextDiff};

use super::types::{ConflictHunk, MergeResult};

/// 三方向マージを行う。
///
/// - base: 元のテンプレート（前バージョン）
/// - ours: ユーザーの現在のファイル
/// - theirs: 新テンプレート（新バージョン）
pub fn three_way_merge(base: &str, ours: &str, theirs: &str) -> MergeResult {
    // 両方同一なら変更なし
    if ours == theirs {
        return MergeResult::NoChange;
    }
    // ユーザーが変更していなければ新テンプレートをそのまま採用
    if ours == base {
        return MergeResult::Clean(theirs.to_string());
    }
    // テンプレート側が変更されていなければユーザーの変更を維持
    if theirs == base {
        return MergeResult::Clean(ours.to_string());
    }

    // 両方変更あり — 行単位でマージを試みる
    let diff_ours = TextDiff::from_lines(base, ours);
    let diff_theirs = TextDiff::from_lines(base, theirs);

    let ours_changed: std::collections::HashSet<usize> = diff_ours
        .iter_all_changes()
        .filter(|c| c.tag() != ChangeTag::Equal)
        .filter_map(|c| c.old_index())
        .collect();

    let theirs_changed: std::collections::HashSet<usize> = diff_theirs
        .iter_all_changes()
        .filter(|c| c.tag() != ChangeTag::Equal)
        .filter_map(|c| c.old_index())
        .collect();

    // 変更行が重ならなければクリーンマージ
    if ours_changed.is_disjoint(&theirs_changed) {
        // theirs の変更を ours に適用する（theirs を基準に、base→ours の変更を反映）
        // 簡易実装: theirs の差分を ours に適用
        // ここでは theirs を採用しつつ ours 固有の変更も保持する
        let base_lines: Vec<&str> = base.lines().collect();
        let ours_lines: Vec<&str> = ours.lines().collect();
        let theirs_lines: Vec<&str> = theirs.lines().collect();

        let mut merged = Vec::new();
        let max_len = base_lines
            .len()
            .max(ours_lines.len())
            .max(theirs_lines.len());

        for i in 0..max_len {
            let base_line = base_lines.get(i).copied().unwrap_or("");
            let ours_line = ours_lines.get(i).copied().unwrap_or("");
            let theirs_line = theirs_lines.get(i).copied().unwrap_or("");

            if theirs_line != base_line {
                merged.push(theirs_line);
            } else if ours_line != base_line {
                merged.push(ours_line);
            } else {
                merged.push(base_line);
            }
        }

        let result = merged.join("\n");
        // 元ファイルが改行で終わっていたら改行を追加
        if theirs.ends_with('\n') || ours.ends_with('\n') {
            return MergeResult::Clean(result + "\n");
        }
        return MergeResult::Clean(result);
    }

    // 変更行が重なる場合はコンフリクト
    MergeResult::Conflict(vec![ConflictHunk {
        base: base.to_string(),
        ours: ours.to_string(),
        theirs: theirs.to_string(),
    }])
}

/// 差分を表示用文字列としてフォーマットする。
pub fn format_diff(old: &str, new: &str) -> String {
    let diff = TextDiff::from_lines(old, new);
    let mut output = String::new();
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        output.push_str(&format!("{sign}{change}"));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_change() {
        let result = three_way_merge("hello", "hello", "hello");
        assert!(matches!(result, MergeResult::NoChange));
    }

    #[test]
    fn test_only_template_changed() {
        let result = three_way_merge("hello", "hello", "world");
        match result {
            MergeResult::Clean(s) => assert_eq!(s, "world"),
            _ => panic!("expected Clean"),
        }
    }

    #[test]
    fn test_only_user_changed() {
        let result = three_way_merge("hello", "world", "hello");
        match result {
            MergeResult::Clean(s) => assert_eq!(s, "world"),
            _ => panic!("expected Clean"),
        }
    }

    #[test]
    fn test_both_same_change() {
        let result = three_way_merge("hello", "world", "world");
        assert!(matches!(result, MergeResult::NoChange));
    }

    #[test]
    fn test_conflict() {
        let base = "line1\nline2\nline3";
        let ours = "line1\nchanged_by_user\nline3";
        let theirs = "line1\nchanged_by_template\nline3";
        let result = three_way_merge(base, ours, theirs);
        assert!(matches!(result, MergeResult::Conflict(_)));
    }

    #[test]
    fn test_format_diff() {
        let result = format_diff("hello\n", "world\n");
        assert!(result.contains("-hello"));
        assert!(result.contains("+world"));
    }
}
