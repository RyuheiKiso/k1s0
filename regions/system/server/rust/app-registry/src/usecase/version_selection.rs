use crate::domain::entity::platform::Platform;
use crate::domain::entity::version::AppVersion;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionSelectionError {
    NotFound,
    Ambiguous,
}

pub fn normalize_arch(arch: &str) -> String {
    match arch.trim().to_lowercase().as_str() {
        "x64" | "x86_64" | "amd64" => "amd64".to_string(),
        "aarch64" | "arm64" => "arm64".to_string(),
        other => other.to_string(),
    }
}

pub fn filter_versions<'a>(
    versions: &'a [AppVersion],
    version: Option<&str>,
    platform: Option<&Platform>,
    arch: Option<&str>,
) -> Vec<&'a AppVersion> {
    let normalized_arch = arch.map(normalize_arch);

    versions
        .iter()
        .filter(|candidate| {
            version.is_none_or(|target| candidate.version == target)
                && platform.is_none_or(|target| candidate.platform == *target)
                && normalized_arch
                    .as_deref()
                    .is_none_or(|target| normalize_arch(&candidate.arch) == target)
        })
        .collect()
}

pub fn select_latest(
    versions: &[AppVersion],
    platform: Option<&Platform>,
    arch: Option<&str>,
) -> Option<AppVersion> {
    filter_versions(versions, None, platform, arch)
        .into_iter()
        .max_by(|left, right| left.published_at.cmp(&right.published_at))
        .cloned()
}

pub fn resolve_version(
    versions: &[AppVersion],
    version: &str,
    platform: Option<&Platform>,
    arch: Option<&str>,
) -> Result<AppVersion, VersionSelectionError> {
    let matched = filter_versions(versions, Some(version), platform, arch);

    match matched.as_slice() {
        [] => Err(VersionSelectionError::NotFound),
        [single] => Ok((*single).clone()),
        _ => Err(VersionSelectionError::Ambiguous),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    fn version(version: &str, platform: Platform, arch: &str, published_at: &str) -> AppVersion {
        AppVersion {
            id: uuid::Uuid::new_v4(),
            app_id: "order-client".to_string(),
            version: version.to_string(),
            platform,
            arch: arch.to_string(),
            size_bytes: Some(1),
            checksum_sha256: "checksum".to_string(),
            s3_key: "key".to_string(),
            release_notes: None,
            mandatory: false,
            published_at: DateTime::parse_from_rfc3339(published_at)
                .unwrap()
                .with_timezone(&Utc),
            created_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn normalizes_arch_aliases() {
        assert_eq!(normalize_arch("x64"), "amd64");
        assert_eq!(normalize_arch("x86_64"), "amd64");
        assert_eq!(normalize_arch("AARCH64"), "arm64");
    }

    #[test]
    fn selects_latest_for_optional_filters() {
        let versions = vec![
            version("1.0.0", Platform::Windows, "amd64", "2026-03-01T00:00:00Z"),
            version("1.1.0", Platform::Windows, "amd64", "2026-03-02T00:00:00Z"),
            version("1.2.0", Platform::Macos, "arm64", "2026-03-03T00:00:00Z"),
        ];

        let latest = select_latest(&versions, Some(&Platform::Windows), Some("x64")).unwrap();
        assert_eq!(latest.version, "1.1.0");
    }

    #[test]
    fn rejects_ambiguous_version_without_platform_or_arch() {
        let versions = vec![
            version("1.2.0", Platform::Windows, "amd64", "2026-03-01T00:00:00Z"),
            version("1.2.0", Platform::Linux, "amd64", "2026-03-02T00:00:00Z"),
        ];

        let result = resolve_version(&versions, "1.2.0", None, None);
        assert!(matches!(result, Err(VersionSelectionError::Ambiguous)));
    }
}
