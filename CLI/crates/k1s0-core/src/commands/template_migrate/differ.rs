use similar::{ChangeTag, DiffTag, TextDiff};

use super::types::{ConflictHunk, MergeResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Side {
    Ours,
    Theirs,
}

#[derive(Clone, Debug)]
struct ChangeAtom {
    side: Side,
    start: usize,
    end: usize,
    replacement: Vec<String>,
}

impl ChangeAtom {
    fn is_insertion(&self) -> bool {
        self.start == self.end
    }
}

#[derive(Clone, Debug)]
struct ChangeCluster {
    start: usize,
    end: usize,
    atoms: Vec<ChangeAtom>,
}

impl ChangeCluster {
    fn new(atom: ChangeAtom) -> Self {
        Self {
            start: atom.start,
            end: atom.end,
            atoms: vec![atom],
        }
    }

    fn add_atom(&mut self, atom: ChangeAtom) {
        self.start = self.start.min(atom.start);
        self.end = self.end.max(atom.end);
        self.atoms.push(atom);
    }
}

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

    let mut atoms = extract_change_atoms(base, ours, Side::Ours);
    atoms.extend(extract_change_atoms(base, theirs, Side::Theirs));
    atoms.sort_by(compare_atoms);

    let clusters = build_change_clusters(atoms);
    let base_lines = split_lines_with_endings(base);
    let mut merged = String::new();
    let mut cursor = 0;
    let mut conflicts = Vec::new();

    for cluster in &clusters {
        append_base_range(&mut merged, &base_lines, cursor, cluster.start);
        match merge_cluster(cluster, &base_lines, base, ours, theirs) {
            Ok(fragment) => merged.push_str(&fragment),
            Err(conflict) => conflicts.push(conflict),
        }
        cursor = cluster.end;
    }

    append_base_range(&mut merged, &base_lines, cursor, base_lines.len());

    if !conflicts.is_empty() {
        return MergeResult::Conflict(conflicts);
    }

    if merged == ours {
        MergeResult::NoChange
    } else {
        MergeResult::Clean(merged)
    }
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

fn extract_change_atoms(base: &str, updated: &str, side: Side) -> Vec<ChangeAtom> {
    let diff = TextDiff::from_lines(base, updated);
    let new_slices = diff.new_slices();

    diff.ops()
        .iter()
        .filter_map(|op| {
            if op.tag() == DiffTag::Equal {
                return None;
            }

            let (_, old_range, new_range) = op.as_tag_tuple();
            Some(ChangeAtom {
                side,
                start: old_range.start,
                end: old_range.end,
                replacement: new_slices[new_range]
                    .iter()
                    .map(|line| (*line).to_string())
                    .collect(),
            })
        })
        .collect()
}

fn compare_atoms(left: &ChangeAtom, right: &ChangeAtom) -> std::cmp::Ordering {
    left.start
        .cmp(&right.start)
        .then_with(|| usize::from(!left.is_insertion()).cmp(&usize::from(!right.is_insertion())))
        .then_with(|| left.end.cmp(&right.end))
}

fn build_change_clusters(atoms: Vec<ChangeAtom>) -> Vec<ChangeCluster> {
    let mut clusters = Vec::new();
    let mut current: Option<ChangeCluster> = None;

    for atom in atoms {
        match current.as_mut() {
            Some(cluster)
                if cluster
                    .atoms
                    .iter()
                    .any(|existing| atoms_overlap(existing, &atom)) =>
            {
                cluster.add_atom(atom);
            }
            Some(cluster) => {
                clusters.push(cluster.clone());
                current = Some(ChangeCluster::new(atom));
            }
            None => {
                current = Some(ChangeCluster::new(atom));
            }
        }
    }

    if let Some(cluster) = current {
        clusters.push(cluster);
    }

    clusters
}

fn atoms_overlap(left: &ChangeAtom, right: &ChangeAtom) -> bool {
    match (left.is_insertion(), right.is_insertion()) {
        (true, true) => left.start == right.start,
        (true, false) => left.start >= right.start && left.start < right.end,
        (false, true) => right.start >= left.start && right.start < left.end,
        (false, false) => left.start < right.end && right.start < left.end,
    }
}

fn merge_cluster(
    cluster: &ChangeCluster,
    base_lines: &[&str],
    base: &str,
    ours: &str,
    theirs: &str,
) -> Result<String, ConflictHunk> {
    let base_fragment = render_base_fragment(base_lines, cluster.start, cluster.end);
    let ours_atoms = cluster_atoms(cluster, Side::Ours);
    let theirs_atoms = cluster_atoms(cluster, Side::Theirs);

    match (ours_atoms.is_empty(), theirs_atoms.is_empty()) {
        (false, true) => Ok(render_side_fragment(
            base_lines,
            cluster.start,
            cluster.end,
            &ours_atoms,
        )),
        (true, false) => Ok(render_side_fragment(
            base_lines,
            cluster.start,
            cluster.end,
            &theirs_atoms,
        )),
        (false, false) => {
            let ours_fragment =
                render_side_fragment(base_lines, cluster.start, cluster.end, &ours_atoms);
            let theirs_fragment =
                render_side_fragment(base_lines, cluster.start, cluster.end, &theirs_atoms);

            if ours_fragment == theirs_fragment {
                Ok(ours_fragment)
            } else {
                Err(ConflictHunk {
                    base: base.to_string(),
                    ours: ours.to_string(),
                    theirs: theirs.to_string(),
                    base_preview: Some(base_fragment),
                    ours_preview: Some(ours_fragment),
                    theirs_preview: Some(theirs_fragment),
                })
            }
        }
        (true, true) => Ok(String::new()),
    }
}

fn cluster_atoms(cluster: &ChangeCluster, side: Side) -> Vec<&ChangeAtom> {
    let mut atoms: Vec<_> = cluster
        .atoms
        .iter()
        .filter(|atom| atom.side == side)
        .collect();
    atoms.sort_by(|left, right| compare_atoms(left, right));
    atoms
}

fn render_base_fragment(base_lines: &[&str], start: usize, end: usize) -> String {
    let mut rendered = String::new();
    append_base_range(&mut rendered, base_lines, start, end);
    rendered
}

fn render_side_fragment(
    base_lines: &[&str],
    start: usize,
    end: usize,
    atoms: &[&ChangeAtom],
) -> String {
    let mut rendered = String::new();
    let mut cursor = start;

    for atom in atoms {
        append_base_range(&mut rendered, base_lines, cursor, atom.start);
        for line in &atom.replacement {
            rendered.push_str(line);
        }
        if atom.end > cursor {
            cursor = atom.end;
        }
    }

    append_base_range(&mut rendered, base_lines, cursor, end);
    rendered
}

fn append_base_range(output: &mut String, base_lines: &[&str], start: usize, end: usize) {
    for line in &base_lines[start..end] {
        output.push_str(line);
    }
}

fn split_lines_with_endings(text: &str) -> Vec<&str> {
    if text.is_empty() {
        Vec::new()
    } else {
        text.split_inclusive('\n').collect()
    }
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
    fn test_non_overlapping_line_updates_merge_cleanly() {
        let base = "a\nb\nc\n";
        let ours = "A\nb\nc\n";
        let theirs = "a\nb\nC\n";
        let result = three_way_merge(base, ours, theirs);
        match result {
            MergeResult::Clean(content) => assert_eq!(content, "A\nb\nC\n"),
            other => panic!("expected Clean, got {other:?}"),
        }
    }

    #[test]
    fn test_disjoint_insertions_merge_cleanly() {
        let base = "a\nb\nc\n";
        let ours = "a\nx\nb\nc\n";
        let theirs = "a\nb\ny\nc\n";
        let result = three_way_merge(base, ours, theirs);
        match result {
            MergeResult::Clean(content) => assert_eq!(content, "a\nx\nb\ny\nc\n"),
            other => panic!("expected Clean, got {other:?}"),
        }
    }

    #[test]
    fn test_same_anchor_insertions_conflict() {
        let base = "a\nb\n";
        let ours = "a\nx\nb\n";
        let theirs = "a\ny\nb\n";
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
