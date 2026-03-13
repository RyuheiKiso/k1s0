use std::cmp::Ordering;

use crate::model::{AppVersionInfo, UpdateType};

pub fn compare_versions(left: &str, right: &str) -> Ordering {
    let left_parts = normalize_version(left);
    let right_parts = normalize_version(right);
    let length = left_parts.len().max(right_parts.len());

    for index in 0..length {
        let left_value = left_parts.get(index).copied().unwrap_or(0);
        let right_value = right_parts.get(index).copied().unwrap_or(0);
        match left_value.cmp(&right_value) {
            Ordering::Equal => continue,
            other => return other,
        }
    }

    Ordering::Equal
}

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

fn normalize_version(version: &str) -> Vec<u32> {
    version
        .split('.')
        .map(|segment| {
            let numeric: String = segment.chars().filter(|c| c.is_ascii_digit()).collect();
            numeric.parse::<u32>().unwrap_or(0)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_version() {
        assert_eq!(normalize_version("1.2.3"), vec![1, 2, 3]);
        assert_eq!(normalize_version("1.2.3-beta"), vec![1, 2, 3]);
        assert_eq!(normalize_version("1.0"), vec![1, 0]);
    }
}
