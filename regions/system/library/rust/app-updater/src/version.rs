use std::cmp::Ordering;

use crate::model::{AppVersionInfo, UpdateType};

/// 2つのバージョン文字列を比較する
///
/// セマンティックバージョニング（major.minor.patch）に基づいて比較し、
/// `Ordering::Less`・`Ordering::Equal`・`Ordering::Greater` を返す。
/// プレリリースサフィックス（例: "-beta"）は無視して数値のみで比較する。
#[must_use]
pub fn compare_versions(left: &str, right: &str) -> Ordering {
    let left_parts = normalize_version(left);
    let right_parts = normalize_version(right);
    let length = left_parts.len().max(right_parts.len());

    for index in 0..length {
        let left_value = left_parts.get(index).copied().unwrap_or(0);
        let right_value = right_parts.get(index).copied().unwrap_or(0);
        match left_value.cmp(&right_value) {
            // Equal の場合は次のインデックスに進むため continue は不要
            Ordering::Equal => {}
            other => return other,
        }
    }

    Ordering::Equal
}

/// 現在のバージョンとサーバーのバージョン情報からアップデート種別を判定する
///
/// - 現在バージョンが最低バージョンを下回る、または `mandatory` フラグが true → `Mandatory`
/// - 現在バージョンが最新バージョンを下回る → `Optional`
/// - それ以外 → `None`
#[must_use]
pub fn determine_update_type(current_version: &str, version_info: &AppVersionInfo) -> UpdateType {
    if compare_versions(current_version, &version_info.minimum_version) == Ordering::Less
        || version_info.mandatory
    {
        return UpdateType::Mandatory;
    }

    if compare_versions(current_version, &version_info.latest_version) == Ordering::Less {
        return UpdateType::Optional;
    }

    UpdateType::None
}

/// バージョン文字列を数値のリストに正規化する
///
/// 例: "1.2.3-beta" → [1, 2, 3]
fn normalize_version(version: &str) -> Vec<u32> {
    version
        .split('.')
        .map(|segment| {
            let numeric: String = segment.chars().filter(char::is_ascii_digit).collect();
            numeric.parse::<u32>().unwrap_or(0)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // バージョン文字列を数値リストに正規化することを確認する。
    #[test]
    fn test_normalize_version() {
        assert_eq!(normalize_version("1.2.3"), vec![1, 2, 3]);
        assert_eq!(normalize_version("1.2.3-beta"), vec![1, 2, 3]);
        assert_eq!(normalize_version("1.0"), vec![1, 0]);
    }
}
