use std::cmp::Ordering;
use std::io::Write;

use k1s0_app_updater::{
    compare_versions, determine_update_type, AppUpdater, AppVersionInfo, ChecksumVerifier,
    InMemoryAppUpdater, UpdateType,
};

// ===========================================================================
// compare_versions
// ===========================================================================

#[test]
fn compare_versions_equal() {
    assert_eq!(compare_versions("1.0.0", "1.0.0"), Ordering::Equal);
}

#[test]
fn compare_versions_greater() {
    assert_eq!(compare_versions("2.0.0", "1.0.0"), Ordering::Greater);
    assert_eq!(compare_versions("1.1.0", "1.0.0"), Ordering::Greater);
    assert_eq!(compare_versions("1.0.1", "1.0.0"), Ordering::Greater);
}

#[test]
fn compare_versions_lesser() {
    assert_eq!(compare_versions("1.0.0", "2.0.0"), Ordering::Less);
    assert_eq!(compare_versions("1.0.0", "1.1.0"), Ordering::Less);
    assert_eq!(compare_versions("1.0.0", "1.0.1"), Ordering::Less);
}

#[test]
fn compare_versions_different_lengths() {
    assert_eq!(compare_versions("1.0", "1.0.0"), Ordering::Equal);
    assert_eq!(compare_versions("1.0.0", "1.0"), Ordering::Equal);
    assert_eq!(compare_versions("1.0", "1.0.1"), Ordering::Less);
    assert_eq!(compare_versions("1.0.1", "1.0"), Ordering::Greater);
}

#[test]
fn compare_versions_pre_release() {
    // Pre-release suffixes are stripped to numeric; "1.0.0-beta" -> [1, 0, 0]
    assert_eq!(compare_versions("1.0.0-beta", "1.0.0"), Ordering::Equal);
    assert_eq!(compare_versions("1.0.0-alpha", "1.0.0-beta"), Ordering::Equal);
}

// ===========================================================================
// determine_update_type
// ===========================================================================

fn version_info(latest: &str, minimum: &str, mandatory: bool) -> AppVersionInfo {
    AppVersionInfo {
        latest_version: latest.to_string(),
        minimum_version: minimum.to_string(),
        mandatory,
        release_notes: None,
        published_at: None,
    }
}

#[test]
fn determine_update_type_mandatory_below_minimum() {
    let info = version_info("2.0.0", "1.5.0", false);
    assert_eq!(determine_update_type("1.0.0", &info), UpdateType::Mandatory);
}

#[test]
fn determine_update_type_mandatory_flag() {
    let info = version_info("2.0.0", "0.0.0", true);
    assert_eq!(determine_update_type("1.0.0", &info), UpdateType::Mandatory);
}

#[test]
fn determine_update_type_optional() {
    let info = version_info("2.0.0", "1.0.0", false);
    assert_eq!(determine_update_type("1.5.0", &info), UpdateType::Optional);
}

#[test]
fn determine_update_type_none_at_latest() {
    let info = version_info("2.0.0", "1.0.0", false);
    assert_eq!(determine_update_type("2.0.0", &info), UpdateType::None);
}

#[test]
fn determine_update_type_none_above_latest() {
    let info = version_info("2.0.0", "1.0.0", false);
    assert_eq!(determine_update_type("3.0.0", &info), UpdateType::None);
}

// ===========================================================================
// InMemoryAppUpdater
// ===========================================================================

#[tokio::test]
async fn in_memory_fetch_version_info() {
    let info = version_info("2.0.0", "1.0.0", false);
    let updater = InMemoryAppUpdater::new(info.clone(), "1.5.0".to_string());

    let fetched = updater.fetch_version_info().await.unwrap();
    assert_eq!(fetched.latest_version, "2.0.0");
    assert_eq!(fetched.minimum_version, "1.0.0");
    assert!(!fetched.mandatory);
}

#[tokio::test]
async fn in_memory_check_for_update_optional() {
    let info = version_info("2.0.0", "1.0.0", false);
    let updater = InMemoryAppUpdater::new(info, "1.5.0".to_string());

    let result = updater.check_for_update().await.unwrap();
    assert_eq!(result.update_type, UpdateType::Optional);
    assert_eq!(result.current_version, "1.5.0");
    assert_eq!(result.latest_version, "2.0.0");
    assert!(result.needs_update());
    assert!(!result.is_mandatory());
}

#[tokio::test]
async fn in_memory_check_for_update_mandatory() {
    let info = version_info("2.0.0", "1.5.0", false);
    let updater = InMemoryAppUpdater::new(info, "1.0.0".to_string());

    let result = updater.check_for_update().await.unwrap();
    assert_eq!(result.update_type, UpdateType::Mandatory);
    assert!(result.needs_update());
    assert!(result.is_mandatory());
}

#[tokio::test]
async fn in_memory_check_for_update_none() {
    let info = version_info("2.0.0", "1.0.0", false);
    let updater = InMemoryAppUpdater::new(info, "2.0.0".to_string());

    let result = updater.check_for_update().await.unwrap();
    assert_eq!(result.update_type, UpdateType::None);
    assert!(!result.needs_update());
}

#[tokio::test]
async fn in_memory_set_version_info() {
    let info = version_info("1.0.0", "1.0.0", false);
    let updater = InMemoryAppUpdater::new(info, "1.0.0".to_string());

    updater
        .set_version_info(version_info("3.0.0", "2.0.0", true))
        .await;

    let fetched = updater.fetch_version_info().await.unwrap();
    assert_eq!(fetched.latest_version, "3.0.0");
    assert!(fetched.mandatory);
}

#[tokio::test]
async fn in_memory_set_current_version() {
    let info = version_info("2.0.0", "1.0.0", false);
    let updater = InMemoryAppUpdater::new(info, "1.0.0".to_string());

    updater.set_current_version("2.0.0".to_string()).await;

    let result = updater.check_for_update().await.unwrap();
    assert_eq!(result.update_type, UpdateType::None);
}

#[tokio::test]
async fn in_memory_default() {
    let updater = InMemoryAppUpdater::default();
    let result = updater.check_for_update().await.unwrap();
    assert_eq!(result.update_type, UpdateType::None);
    assert_eq!(result.current_version, "0.0.0");
}

// ===========================================================================
// ChecksumVerifier
// ===========================================================================

#[tokio::test]
async fn checksum_calculate() {
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(b"hello world").unwrap();
    tmp.flush().unwrap();

    let checksum = ChecksumVerifier::calculate(tmp.path()).await.unwrap();
    // SHA-256 of "hello world"
    assert_eq!(
        checksum,
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );
}

#[tokio::test]
async fn checksum_verify_success() {
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(b"hello world").unwrap();
    tmp.flush().unwrap();

    let result = ChecksumVerifier::verify(
        tmp.path(),
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
    )
    .await
    .unwrap();
    assert!(result);
}

#[tokio::test]
async fn checksum_verify_failure() {
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(b"hello world").unwrap();
    tmp.flush().unwrap();

    let result = ChecksumVerifier::verify(tmp.path(), "0000000000000000")
        .await
        .unwrap();
    assert!(!result);
}

#[tokio::test]
async fn checksum_verify_or_error_success() {
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(b"hello world").unwrap();
    tmp.flush().unwrap();

    let result = ChecksumVerifier::verify_or_error(
        tmp.path(),
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn checksum_verify_or_error_mismatch() {
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(b"hello world").unwrap();
    tmp.flush().unwrap();

    let result = ChecksumVerifier::verify_or_error(tmp.path(), "badhash").await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{err}").contains("checksum"));
}

// ===========================================================================
// error variant coverage
// ===========================================================================

#[test]
fn error_display_connection() {
    let e = k1s0_app_updater::AppUpdaterError::Connection("refused".to_string());
    assert!(format!("{e}").contains("refused"));
}

#[test]
fn error_display_invalid_config() {
    let e = k1s0_app_updater::AppUpdaterError::InvalidConfig("empty url".to_string());
    assert!(format!("{e}").contains("empty url"));
}

#[test]
fn error_display_unauthorized() {
    let e = k1s0_app_updater::AppUpdaterError::Unauthorized;
    assert!(!format!("{e}").is_empty());
}

#[test]
fn error_display_app_not_found() {
    let e = k1s0_app_updater::AppUpdaterError::AppNotFound("my-app".to_string());
    assert!(format!("{e}").contains("my-app"));
}

#[test]
fn error_display_version_not_found() {
    let e = k1s0_app_updater::AppUpdaterError::VersionNotFound("1.0.0".to_string());
    assert!(format!("{e}").contains("1.0.0"));
}

#[test]
fn error_display_checksum() {
    let e = k1s0_app_updater::AppUpdaterError::Checksum("mismatch".to_string());
    assert!(format!("{e}").contains("mismatch"));
}

// ===========================================================================
// config
// ===========================================================================

#[test]
fn config_default() {
    let config = k1s0_app_updater::AppUpdaterConfig::default();
    assert!(config.server_url.is_empty());
    assert!(config.app_id.is_empty());
    assert!(config.platform.is_none());
    assert!(config.arch.is_none());
    assert!(config.check_interval.is_none());
    assert!(config.timeout.is_some());
}

// ===========================================================================
// AppRegistryAppUpdater validation
// ===========================================================================

#[test]
fn registry_updater_rejects_empty_server_url() {
    let config = k1s0_app_updater::AppUpdaterConfig {
        server_url: "".to_string(),
        app_id: "my-app".to_string(),
        ..Default::default()
    };
    let result = k1s0_app_updater::AppRegistryAppUpdater::new(config, "1.0.0".to_string());
    assert!(result.is_err());
}

#[test]
fn registry_updater_rejects_empty_app_id() {
    let config = k1s0_app_updater::AppUpdaterConfig {
        server_url: "https://example.com".to_string(),
        app_id: "".to_string(),
        ..Default::default()
    };
    let result = k1s0_app_updater::AppRegistryAppUpdater::new(config, "1.0.0".to_string());
    assert!(result.is_err());
}
