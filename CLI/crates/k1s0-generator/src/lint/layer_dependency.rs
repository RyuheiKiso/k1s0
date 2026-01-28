//! 層間依存関係のLintルール（K040-K047）
//!
//! 3層構造（framework -> domain -> feature）における
//! 層間の依存関係を検証する。

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::manifest::{LayerType, Manifest};

use super::types::{RuleId, Severity, Violation};

/// 層間依存関係のLintルール
pub struct LayerDependencyRules {
    /// k1s0 リポジトリのルートパス
    root_path: PathBuf,
    /// キャッシュされた manifest 情報
    manifest_cache: HashMap<String, CachedManifest>,
}

/// キャッシュされた manifest 情報
#[derive(Debug, Clone)]
struct CachedManifest {
    /// パス
    path: PathBuf,
    /// 層
    layer: LayerType,
    /// バージョン（domain のみ）
    version: Option<String>,
    /// 非推奨情報
    deprecated: bool,
    /// 最小 framework バージョン
    min_framework_version: Option<String>,
    /// 依存する domain（名前 -> バージョン制約）
    domain_dependencies: HashMap<String, String>,
    /// 破壊的変更
    breaking_changes: HashMap<String, String>,
}

impl LayerDependencyRules {
    /// 新しいインスタンスを作成
    pub fn new(root_path: impl Into<PathBuf>) -> Self {
        Self {
            root_path: root_path.into(),
            manifest_cache: HashMap::new(),
        }
    }

    /// 全ての層間依存関係をチェック
    pub fn check(&mut self, manifest_path: &Path) -> Vec<Violation> {
        let mut violations = Vec::new();

        // manifest を読み込み
        let manifest = match Manifest::load(manifest_path) {
            Ok(m) => m,
            Err(_) => return violations, // K001 で報告される
        };

        // キャッシュを構築
        self.build_cache();

        // K040: 層間依存の基本チェック
        violations.extend(self.check_layer_dependency(&manifest, manifest_path));

        // K041: domain が存在するかチェック
        violations.extend(self.check_domain_exists(&manifest, manifest_path));

        // K042: domain バージョン制約チェック
        violations.extend(self.check_domain_version(&manifest, manifest_path));

        // K043: 循環依存チェック（将来の拡張用）
        // violations.extend(self.check_circular_dependency(&manifest, manifest_path));

        // K044: 非推奨 domain の使用チェック
        violations.extend(self.check_deprecated_domain(&manifest, manifest_path));

        // K045: min_framework_version チェック
        violations.extend(self.check_min_framework_version(&manifest, manifest_path));

        // K046: breaking_changes の影響チェック
        violations.extend(self.check_breaking_changes(&manifest, manifest_path));

        // K047: domain 層の version 必須チェック
        violations.extend(self.check_domain_version_required(&manifest, manifest_path));

        violations
    }

    /// キャッシュを構築
    fn build_cache(&mut self) {
        if !self.manifest_cache.is_empty() {
            return;
        }

        // domain ディレクトリを走査
        let domain_bases = [
            ("domain/backend/rust", "rust"),
            ("domain/backend/go", "go"),
            ("domain/frontend/react", "react"),
            ("domain/frontend/flutter", "flutter"),
        ];

        for (base, _lang) in &domain_bases {
            let base_path = self.root_path.join(base);
            if !base_path.exists() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(&base_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }

                    let manifest_path = path.join(".k1s0/manifest.json");
                    if let Ok(manifest) = Manifest::load(&manifest_path) {
                        let domain_name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();

                        let domain_deps = manifest
                            .dependencies
                            .as_ref()
                            .and_then(|d| d.domain.clone())
                            .unwrap_or_default();

                        self.manifest_cache.insert(
                            domain_name,
                            CachedManifest {
                                path: path.clone(),
                                layer: manifest.layer,
                                version: manifest.version.clone(),
                                deprecated: manifest.is_deprecated(),
                                min_framework_version: manifest.min_framework_version.clone(),
                                domain_dependencies: domain_deps,
                                breaking_changes: manifest.breaking_changes.clone().unwrap_or_default(),
                            },
                        );
                    }
                }
            }
        }
    }

    /// K040: 層間依存の基本チェック
    fn check_layer_dependency(&self, manifest: &Manifest, path: &Path) -> Vec<Violation> {
        let mut violations = Vec::new();

        match manifest.layer {
            LayerType::Feature => {
                // feature は domain または framework にのみ依存可能
                // 他の feature への依存は違反
                // (現状、feature 間依存の検出は複雑なので将来実装)
            }
            LayerType::Domain => {
                // domain は framework にのみ依存可能
                // 他の domain への依存は循環の可能性があるが許可する（K043 でチェック）
                // feature への依存は違反
                if manifest.domain.is_some() {
                    violations.push(
                        Violation::new(
                            RuleId::LayerDependencyViolation,
                            Severity::Error,
                            "domain 層は他の domain に所属できません",
                        )
                        .with_path(path.display().to_string())
                        .with_hint("domain 層の manifest.json から domain フィールドを削除してください"),
                    );
                }
            }
            LayerType::Framework => {
                // framework は何にも依存しない（他の framework crate は可）
                if manifest.domain.is_some() {
                    violations.push(
                        Violation::new(
                            RuleId::LayerDependencyViolation,
                            Severity::Error,
                            "framework 層は domain に依存できません",
                        )
                        .with_path(path.display().to_string())
                        .with_hint("framework 層は最下層です"),
                    );
                }
            }
        }

        violations
    }

    /// K041: domain の存在チェック
    fn check_domain_exists(&self, manifest: &Manifest, path: &Path) -> Vec<Violation> {
        let mut violations = Vec::new();

        if manifest.layer != LayerType::Feature {
            return violations;
        }

        if let Some(ref domain_name) = manifest.domain {
            if !self.manifest_cache.contains_key(domain_name) {
                violations.push(
                    Violation::new(
                        RuleId::DomainNotFound,
                        Severity::Error,
                        format!("domain '{}' が見つかりません", domain_name),
                    )
                    .with_path(path.display().to_string())
                    .with_hint(format!(
                        "'k1s0 new-domain --name {}' で domain を作成してください",
                        domain_name
                    )),
                );
            }
        }

        // dependencies.domain のチェック
        if let Some(ref deps) = manifest.dependencies {
            if let Some(ref domain_deps) = deps.domain {
                for domain_name in domain_deps.keys() {
                    if !self.manifest_cache.contains_key(domain_name) {
                        violations.push(
                            Violation::new(
                                RuleId::DomainNotFound,
                                Severity::Error,
                                format!("dependencies.domain で指定された domain '{}' が見つかりません", domain_name),
                            )
                            .with_path(path.display().to_string()),
                        );
                    }
                }
            }
        }

        violations
    }

    /// K042: domain バージョン制約チェック
    fn check_domain_version(&self, manifest: &Manifest, path: &Path) -> Vec<Violation> {
        let mut violations = Vec::new();

        if manifest.layer != LayerType::Feature {
            return violations;
        }

        // manifest.domain と domain_version のチェック
        if let Some(ref domain_name) = manifest.domain {
            if let Some(cached) = self.manifest_cache.get(domain_name) {
                if let (Some(constraint), Some(actual_version)) =
                    (&manifest.domain_version, &cached.version)
                {
                    if !version_matches(constraint, actual_version) {
                        violations.push(
                            Violation::new(
                                RuleId::DomainVersionMismatch,
                                Severity::Error,
                                format!(
                                    "domain '{}' のバージョン {} が制約 {} を満たしません",
                                    domain_name, actual_version, constraint
                                ),
                            )
                            .with_path(path.display().to_string())
                            .with_hint("domain_version を更新するか、domain のバージョンを更新してください"),
                        );
                    }
                }
            }
        }

        // dependencies.domain のバージョンチェック
        if let Some(ref deps) = manifest.dependencies {
            if let Some(ref domain_deps) = deps.domain {
                for (domain_name, constraint) in domain_deps {
                    if let Some(cached) = self.manifest_cache.get(domain_name) {
                        if let Some(ref actual_version) = cached.version {
                            if !version_matches(constraint, actual_version) {
                                violations.push(
                                    Violation::new(
                                        RuleId::DomainVersionMismatch,
                                        Severity::Error,
                                        format!(
                                            "domain '{}' のバージョン {} が制約 {} を満たしません",
                                            domain_name, actual_version, constraint
                                        ),
                                    )
                                    .with_path(path.display().to_string()),
                                );
                            }
                        }
                    }
                }
            }
        }

        violations
    }

    /// K044: 非推奨 domain の使用チェック
    fn check_deprecated_domain(&self, manifest: &Manifest, path: &Path) -> Vec<Violation> {
        let mut violations = Vec::new();

        if manifest.layer != LayerType::Feature {
            return violations;
        }

        // manifest.domain のチェック
        if let Some(ref domain_name) = manifest.domain {
            if let Some(cached) = self.manifest_cache.get(domain_name) {
                if cached.deprecated {
                    violations.push(
                        Violation::new(
                            RuleId::DeprecatedDomainUsage,
                            Severity::Warning,
                            format!("domain '{}' は非推奨です", domain_name),
                        )
                        .with_path(path.display().to_string())
                        .with_hint("別の domain への移行を検討してください"),
                    );
                }
            }
        }

        // dependencies.domain のチェック
        if let Some(ref deps) = manifest.dependencies {
            if let Some(ref domain_deps) = deps.domain {
                for domain_name in domain_deps.keys() {
                    if let Some(cached) = self.manifest_cache.get(domain_name) {
                        if cached.deprecated {
                            violations.push(
                                Violation::new(
                                    RuleId::DeprecatedDomainUsage,
                                    Severity::Warning,
                                    format!("domain '{}' は非推奨です", domain_name),
                                )
                                .with_path(path.display().to_string()),
                            );
                        }
                    }
                }
            }
        }

        violations
    }

    /// K045: min_framework_version チェック
    fn check_min_framework_version(&self, manifest: &Manifest, path: &Path) -> Vec<Violation> {
        let mut violations = Vec::new();

        // domain 層の min_framework_version と k1s0_version を比較
        if manifest.layer == LayerType::Domain {
            if let (Some(min_version), k1s0_version) =
                (&manifest.min_framework_version, &manifest.k1s0_version)
            {
                if !version_satisfies(k1s0_version, min_version) {
                    violations.push(
                        Violation::new(
                            RuleId::MinFrameworkVersionViolation,
                            Severity::Warning,
                            format!(
                                "k1s0 バージョン {} が min_framework_version {} を満たしていません",
                                k1s0_version, min_version
                            ),
                        )
                        .with_path(path.display().to_string())
                        .with_hint("k1s0 CLI をアップグレードしてください"),
                    );
                }
            }
        }

        // feature 層が依存する domain の min_framework_version もチェック
        if manifest.layer == LayerType::Feature {
            if let Some(ref domain_name) = manifest.domain {
                if let Some(cached) = self.manifest_cache.get(domain_name) {
                    if let Some(ref min_version) = cached.min_framework_version {
                        if !version_satisfies(&manifest.k1s0_version, min_version) {
                            violations.push(
                                Violation::new(
                                    RuleId::MinFrameworkVersionViolation,
                                    Severity::Warning,
                                    format!(
                                        "domain '{}' の min_framework_version {} を満たしていません",
                                        domain_name, min_version
                                    ),
                                )
                                .with_path(path.display().to_string()),
                            );
                        }
                    }
                }
            }
        }

        violations
    }

    /// K046: breaking_changes の影響チェック
    fn check_breaking_changes(&self, manifest: &Manifest, path: &Path) -> Vec<Violation> {
        let mut violations = Vec::new();

        if manifest.layer != LayerType::Feature {
            return violations;
        }

        // feature が依存する domain の breaking_changes をチェック
        if let Some(ref domain_name) = manifest.domain {
            if let Some(cached) = self.manifest_cache.get(domain_name) {
                if !cached.breaking_changes.is_empty() {
                    // domain_version 制約を取得
                    let constraint = manifest.domain_version.as_deref().unwrap_or("*");

                    // breaking_changes の中で、制約に影響するバージョンを探す
                    for (version, description) in &cached.breaking_changes {
                        // 将来のバージョンの breaking_changes は警告
                        if let Some(ref current_version) = cached.version {
                            if version_greater(version, current_version) {
                                // まだ適用されていない breaking_change なので無視
                                continue;
                            }
                        }

                        // 制約範囲内の breaking_change なら警告
                        if version_matches(constraint, version) {
                            violations.push(
                                Violation::new(
                                    RuleId::BreakingChangeImpact,
                                    Severity::Warning,
                                    format!(
                                        "domain '{}' v{} に破壊的変更があります: {}",
                                        domain_name, version, description
                                    ),
                                )
                                .with_path(path.display().to_string())
                                .with_hint("CHANGELOG を確認し、必要に応じてコードを更新してください"),
                            );
                        }
                    }
                }
            }
        }

        violations
    }

    /// K047: domain 層の version 必須チェック
    fn check_domain_version_required(&self, manifest: &Manifest, path: &Path) -> Vec<Violation> {
        let mut violations = Vec::new();

        if manifest.layer == LayerType::Domain && manifest.version.is_none() {
            violations.push(
                Violation::new(
                    RuleId::DomainVersionMissing,
                    Severity::Error,
                    "domain 層には version フィールドが必須です",
                )
                .with_path(path.display().to_string())
                .with_hint("manifest.json に \"version\": \"0.1.0\" を追加してください"),
            );
        }

        violations
    }
}

/// バージョン制約をチェック（^x.y.z 形式に対応）
fn version_matches(constraint: &str, version: &str) -> bool {
    // semver クレートを使用
    let version = match semver::Version::parse(version) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let req = match semver::VersionReq::parse(constraint) {
        Ok(r) => r,
        Err(_) => return false,
    };

    req.matches(&version)
}

/// バージョンが要求を満たすかチェック
fn version_satisfies(actual: &str, required: &str) -> bool {
    let actual = match semver::Version::parse(actual) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let required = match semver::Version::parse(required) {
        Ok(v) => v,
        Err(_) => return false,
    };

    actual >= required
}

/// バージョン比較（v1 > v2）
fn version_greater(v1: &str, v2: &str) -> bool {
    let v1 = match semver::Version::parse(v1) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let v2 = match semver::Version::parse(v2) {
        Ok(v) => v,
        Err(_) => return false,
    };

    v1 > v2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_matches() {
        assert!(version_matches("^1.0.0", "1.0.0"));
        assert!(version_matches("^1.0.0", "1.5.0"));
        assert!(version_matches("^1.0.0", "1.0.5"));
        assert!(!version_matches("^1.0.0", "2.0.0"));
        assert!(!version_matches("^1.0.0", "0.9.0"));

        assert!(version_matches("^0.1.0", "0.1.0"));
        assert!(version_matches("^0.1.0", "0.1.5"));
        assert!(!version_matches("^0.1.0", "0.2.0"));
    }

    #[test]
    fn test_version_satisfies() {
        assert!(version_satisfies("0.1.0", "0.1.0"));
        assert!(version_satisfies("0.2.0", "0.1.0"));
        assert!(!version_satisfies("0.1.0", "0.2.0"));
    }

    #[test]
    fn test_version_greater() {
        assert!(version_greater("1.0.0", "0.9.0"));
        assert!(version_greater("1.1.0", "1.0.0"));
        assert!(!version_greater("1.0.0", "1.0.0"));
        assert!(!version_greater("0.9.0", "1.0.0"));
    }
}
