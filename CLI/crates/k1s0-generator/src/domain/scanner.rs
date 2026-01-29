//! ドメイン・feature 走査

use std::collections::HashMap;
use std::path::Path;

use crate::manifest::{Deprecated, Manifest};

use super::types::{DomainInfo, DomainScanError, DeprecatedInfo, FeatureInfo};

/// ドメインベースディレクトリ定義
const DOMAIN_BASES: &[(&str, &str, &str)] = &[
    ("domain/backend/rust", "backend-rust", "rust"),
    ("domain/backend/go", "backend-go", "go"),
    ("domain/backend/csharp", "backend-csharp", "csharp"),
    ("domain/backend/python", "backend-python", "python"),
    ("domain/frontend/react", "frontend-react", "typescript"),
    ("domain/frontend/flutter", "frontend-flutter", "dart"),
];

/// feature ベースディレクトリ定義
const FEATURE_BASES: &[(&str, &str)] = &[
    ("feature/backend/rust", "backend-rust"),
    ("feature/backend/go", "backend-go"),
    ("feature/backend/csharp", "backend-csharp"),
    ("feature/backend/python", "backend-python"),
    ("feature/frontend/react", "frontend-react"),
    ("feature/frontend/flutter", "frontend-flutter"),
];

/// 全ドメインを走査する
pub fn scan_domains(root: &Path) -> Result<Vec<DomainInfo>, DomainScanError> {
    let mut domains = Vec::new();

    for &(base_rel, domain_type, language) in DOMAIN_BASES {
        let base = root.join(base_rel);
        if !base.exists() {
            continue;
        }

        let entries = std::fs::read_dir(&base)?;
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join(".k1s0/manifest.json");
            if !manifest_path.exists() {
                continue;
            }

            let manifest = Manifest::load(&manifest_path).map_err(|e| {
                DomainScanError::ManifestParse {
                    path: manifest_path.clone(),
                    message: e.to_string(),
                }
            })?;

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let version = manifest
                .version
                .clone()
                .unwrap_or_else(|| "0.0.0".to_string());

            // ドメイン間依存を取得
            let dependencies = manifest
                .dependencies
                .as_ref()
                .and_then(|d| d.domain.clone())
                .unwrap_or_default();

            let deprecated = manifest.deprecated.as_ref().and_then(|d| {
                if d.is_deprecated() {
                    let (message, alternative) = match d {
                        Deprecated::Flag(_) => ("deprecated".to_string(), None),
                        Deprecated::Info(info) => (
                            info.reason.clone().unwrap_or_else(|| "deprecated".to_string()),
                            info.migrate_to.clone(),
                        ),
                    };
                    Some(DeprecatedInfo {
                        message,
                        alternative,
                    })
                } else {
                    None
                }
            });

            let breaking_changes = manifest.breaking_changes.clone();

            domains.push(DomainInfo {
                name,
                version,
                domain_type: domain_type.to_string(),
                language: language.to_string(),
                path,
                dependencies,
                min_framework_version: manifest.min_framework_version.clone(),
                deprecated,
                breaking_changes,
            });
        }
    }

    domains.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(domains)
}

/// 全 feature を走査する
pub fn scan_features(root: &Path) -> Result<Vec<FeatureInfo>, DomainScanError> {
    let mut features = Vec::new();

    for &(base_rel, feature_type) in FEATURE_BASES {
        let base = root.join(base_rel);
        if !base.exists() {
            continue;
        }

        let entries = std::fs::read_dir(&base)?;
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join(".k1s0/manifest.json");
            if !manifest_path.exists() {
                continue;
            }

            let manifest = match Manifest::load(&manifest_path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            // ドメイン依存を収集
            let mut domain_dependencies = HashMap::new();

            // 新形式: dependencies.domain
            if let Some(deps) = &manifest.dependencies {
                if let Some(domain_deps) = &deps.domain {
                    domain_dependencies.extend(domain_deps.clone());
                }
            }

            // 旧形式: manifest.domain + domain_version
            if let Some(domain_name) = &manifest.domain {
                domain_dependencies
                    .entry(domain_name.clone())
                    .or_insert_with(|| {
                        manifest
                            .domain_version
                            .clone()
                            .unwrap_or_else(|| "^0.1.0".to_string())
                    });
            }

            features.push(FeatureInfo {
                name,
                feature_type: feature_type.to_string(),
                path,
                domain_dependencies,
            });
        }
    }

    features.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(features)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_scan_domains_nonexistent_root() {
        let result = scan_domains(&PathBuf::from("/nonexistent/path"));
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_scan_features_nonexistent_root() {
        let result = scan_features(&PathBuf::from("/nonexistent/path"));
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
