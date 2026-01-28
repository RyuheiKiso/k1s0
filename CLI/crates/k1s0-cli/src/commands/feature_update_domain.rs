//! `k1s0 feature update-domain` コマンド
//!
//! feature の domain 依存バージョンを更新する。

use std::path::{Path, PathBuf};

use clap::Args;

use k1s0_generator::manifest::Manifest;

use crate::error::{CliError, Result};
use crate::output::output;

/// `k1s0 feature update-domain` の引数
#[derive(Args, Debug)]
pub struct FeatureUpdateDomainArgs {
    /// feature 名
    #[arg(short, long)]
    pub name: String,

    /// domain 名
    #[arg(long)]
    pub domain: String,

    /// 新しいバージョン制約（例: ^1.0.0）
    #[arg(long)]
    pub version: String,
}

/// `k1s0 feature update-domain` を実行する
pub fn execute(args: FeatureUpdateDomainArgs) -> Result<()> {
    let out = output();

    out.header("k1s0 feature update-domain");
    out.newline();

    // feature を探す
    let (feature_path, feature_type) = find_feature(&args.name)?;

    out.list_item("feature", &args.name);
    out.list_item("type", &feature_type);
    out.list_item("path", &feature_path.display().to_string());
    out.list_item("domain", &args.domain);
    out.list_item("new_version", &args.version);
    out.newline();

    // manifest.json を読み込む
    let manifest_path = feature_path.join(".k1s0/manifest.json");
    if !manifest_path.exists() {
        return Err(CliError::config(format!(
            "feature '{}' の manifest.json が見つかりません",
            args.name
        ))
        .with_target(manifest_path.display().to_string()));
    }

    let mut manifest = Manifest::load(&manifest_path).map_err(|e| {
        CliError::config(format!("manifest.json の読み込みに失敗: {}", e))
    })?;

    // 現在の domain 依存を確認（新形式: dependencies.domain は HashMap<String, String>）
    let current_version = manifest
        .dependencies
        .as_ref()
        .and_then(|d| d.domain.as_ref())
        .and_then(|domain_deps| domain_deps.get(&args.domain))
        .cloned()
        // 旧形式: manifest.domain_version でもチェック
        .or_else(|| {
            if manifest.domain.as_ref() == Some(&args.domain) {
                manifest.domain_version.clone()
            } else {
                None
            }
        });

    if let Some(ref cv) = current_version {
        out.list_item("current_version", cv);
    } else {
        out.info("現在 domain 依存が設定されていません。新規に追加します。");
    }

    out.newline();

    // manifest.json を更新
    update_manifest_domain(&mut manifest, &args.domain, &args.version);
    manifest.save(&manifest_path).map_err(|e| {
        CliError::io(format!("manifest.json の保存に失敗: {}", e))
    })?;
    out.file_modified(".k1s0/manifest.json");

    // ビルド設定ファイルも更新
    update_build_config(&feature_path, &feature_type, &args.domain, &args.version)?;

    out.newline();
    out.success(&format!(
        "feature '{}' の domain '{}' 依存を {} に更新しました",
        args.name, args.domain, args.version
    ));

    Ok(())
}

/// feature を探す
fn find_feature(feature_name: &str) -> Result<(PathBuf, String)> {
    let feature_bases = [
        ("feature/backend/rust", "backend-rust"),
        ("feature/backend/go", "backend-go"),
        ("feature/frontend/react", "frontend-react"),
        ("feature/frontend/flutter", "frontend-flutter"),
    ];

    for (base_path, feature_type) in &feature_bases {
        let path = PathBuf::from(format!("{}/{}", base_path, feature_name));
        if path.exists() && path.join(".k1s0/manifest.json").exists() {
            return Ok((path, feature_type.to_string()));
        }
    }

    Err(CliError::config(format!(
        "feature '{}' が見つかりません",
        feature_name
    ))
    .with_hint("feature が正しく生成されていることを確認してください"))
}

/// manifest の domain 依存を更新
fn update_manifest_domain(manifest: &mut Manifest, domain_name: &str, version: &str) {
    // 新形式: manifest.domain と manifest.domain_version を更新
    manifest.domain = Some(domain_name.to_string());
    manifest.domain_version = Some(version.to_string());

    // 新形式: dependencies.domain も更新（HashMap<String, String>）
    if manifest.dependencies.is_none() {
        manifest.dependencies = Some(k1s0_generator::manifest::Dependencies::default());
    }

    if let Some(ref mut deps) = manifest.dependencies {
        if deps.domain.is_none() {
            deps.domain = Some(std::collections::HashMap::new());
        }
        if let Some(ref mut domain_deps) = deps.domain {
            domain_deps.insert(domain_name.to_string(), version.to_string());
        }
    }
}

/// ビルド設定ファイルを更新
fn update_build_config(
    feature_path: &Path,
    feature_type: &str,
    domain_name: &str,
    version: &str,
) -> Result<()> {
    let out = output();

    match feature_type {
        "backend-rust" => {
            let cargo_toml_path = feature_path.join("Cargo.toml");
            if cargo_toml_path.exists() {
                update_cargo_toml(&cargo_toml_path, domain_name, version)?;
                out.file_modified("Cargo.toml");
            }
        }
        "backend-go" => {
            let go_mod_path = feature_path.join("go.mod");
            if go_mod_path.exists() {
                update_go_mod(&go_mod_path, domain_name)?;
                out.file_modified("go.mod");
            }
        }
        "frontend-react" => {
            let package_json_path = feature_path.join("package.json");
            if package_json_path.exists() {
                update_package_json(&package_json_path, domain_name)?;
                out.file_modified("package.json");
            }
        }
        "frontend-flutter" => {
            let pubspec_path = feature_path.join("pubspec.yaml");
            if pubspec_path.exists() {
                update_pubspec_yaml(&pubspec_path, domain_name)?;
                out.file_modified("pubspec.yaml");
            }
        }
        _ => {}
    }

    Ok(())
}

/// Cargo.toml を更新（domain 依存を追加/更新）
fn update_cargo_toml(path: &PathBuf, domain_name: &str, _version: &str) -> Result<()> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        CliError::io(format!("Cargo.toml の読み込みに失敗: {}", e))
    })?;

    let domain_snake = domain_name.replace('-', "_");
    let domain_dep_line = format!(
        "{} = {{ path = \"../../../../domain/backend/rust/{}\" }}",
        domain_snake, domain_name
    );

    // 既存の domain 依存を探す
    let domain_pattern = format!("{} = ", domain_snake);
    let new_content = if content.contains(&domain_pattern) {
        // 既存の依存を更新
        let lines: Vec<&str> = content.lines().collect();
        let updated_lines: Vec<String> = lines
            .into_iter()
            .map(|line| {
                if line.trim().starts_with(&domain_pattern) {
                    domain_dep_line.clone()
                } else {
                    line.to_string()
                }
            })
            .collect();
        updated_lines.join("\n")
    } else {
        // 新しい依存を追加（# Domain dependency セクションの後、または [dependencies] の後）
        if content.contains("# Domain dependency") {
            // 既にセクションがある場合はその後に追加
            content.replace(
                "# Domain dependency",
                &format!("# Domain dependency\n{}", domain_dep_line),
            )
        } else if content.contains("[dependencies]") {
            // [dependencies] の後に追加
            content.replace(
                "[dependencies]",
                &format!("[dependencies]\n# Domain dependency\n{}", domain_dep_line),
            )
        } else {
            // 見つからない場合は末尾に追加
            format!("{}\n\n# Domain dependency\n{}", content, domain_dep_line)
        }
    };

    std::fs::write(path, new_content).map_err(|e| {
        CliError::io(format!("Cargo.toml の書き込みに失敗: {}", e))
    })?;

    Ok(())
}

/// go.mod を更新（domain 依存を追加/更新）
fn update_go_mod(path: &PathBuf, domain_name: &str) -> Result<()> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        CliError::io(format!("go.mod の読み込みに失敗: {}", e))
    })?;

    let domain_snake = domain_name.replace('-', "_");

    // replace ディレクティブを探す/追加
    let replace_line = format!(
        "replace example.com/domain/{} => ../../../../domain/backend/go/{}",
        domain_snake, domain_name
    );

    let new_content = if content.contains(&format!("replace example.com/domain/{}", domain_snake)) {
        // 既存の replace を更新
        let lines: Vec<&str> = content.lines().collect();
        let updated_lines: Vec<String> = lines
            .into_iter()
            .map(|line| {
                if line.contains(&format!("replace example.com/domain/{}", domain_snake)) {
                    replace_line.clone()
                } else {
                    line.to_string()
                }
            })
            .collect();
        updated_lines.join("\n")
    } else {
        // 新しい replace を追加
        format!("{}\n\n{}", content.trim_end(), replace_line)
    };

    std::fs::write(path, new_content).map_err(|e| {
        CliError::io(format!("go.mod の書き込みに失敗: {}", e))
    })?;

    Ok(())
}

/// package.json を更新（domain 依存を追加/更新）
fn update_package_json(path: &PathBuf, domain_name: &str) -> Result<()> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        CliError::io(format!("package.json の読み込みに失敗: {}", e))
    })?;

    let mut json: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        CliError::config(format!("package.json のパースに失敗: {}", e))
    })?;

    // dependencies に domain パッケージを追加
    let package_name = format!("@k1s0/domain-{}", domain_name);
    if let Some(deps) = json.get_mut("dependencies").and_then(|d| d.as_object_mut()) {
        deps.insert(package_name, serde_json::json!("workspace:*"));
    }

    let new_content = serde_json::to_string_pretty(&json).map_err(|e| {
        CliError::internal(format!("package.json のシリアライズに失敗: {}", e))
    })?;

    std::fs::write(path, new_content).map_err(|e| {
        CliError::io(format!("package.json の書き込みに失敗: {}", e))
    })?;

    Ok(())
}

/// pubspec.yaml を更新（domain 依存を追加/更新）
fn update_pubspec_yaml(path: &PathBuf, domain_name: &str) -> Result<()> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        CliError::io(format!("pubspec.yaml の読み込みに失敗: {}", e))
    })?;

    let domain_snake = domain_name.replace('-', "_");
    let domain_dep = format!(
        "  {}:\n    path: ../../../../domain/frontend/flutter/{}",
        domain_snake, domain_name
    );

    // 既存の domain 依存を探す
    let new_content = if content.contains(&format!("  {}:", domain_snake)) {
        // 既存の依存を更新（複数行の更新は複雑なので、シンプルに行単位で処理）
        let lines: Vec<&str> = content.lines().collect();
        let mut updated_lines: Vec<String> = Vec::new();
        let mut skip_next = false;

        for line in lines {
            if skip_next {
                skip_next = false;
                continue;
            }
            if line.trim().starts_with(&format!("{}:", domain_snake)) {
                updated_lines.push(format!("  {}:", domain_snake));
                updated_lines.push(format!(
                    "    path: ../../../../domain/frontend/flutter/{}",
                    domain_name
                ));
                skip_next = true;
            } else {
                updated_lines.push(line.to_string());
            }
        }
        updated_lines.join("\n")
    } else {
        // dependencies: セクションに追加
        if content.contains("dependencies:") {
            // dependencies: の直後に追加
            let lines: Vec<&str> = content.lines().collect();
            let mut updated_lines: Vec<String> = Vec::new();

            for line in lines.iter() {
                updated_lines.push(line.to_string());
                if line.trim() == "dependencies:" {
                    // 次のインデントされた行の前に挿入
                    updated_lines.push(domain_dep.clone());
                }
            }
            updated_lines.join("\n")
        } else {
            format!("{}\n\ndependencies:\n{}", content, domain_dep)
        }
    };

    std::fs::write(path, new_content).map_err(|e| {
        CliError::io(format!("pubspec.yaml の書き込みに失敗: {}", e))
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use k1s0_generator::manifest::LayerType;

    #[test]
    fn test_update_manifest_domain_new() {
        let mut manifest = k1s0_generator::manifest::Manifest {
            schema_version: "1.0.0".to_string(),
            k1s0_version: "0.1.0".to_string(),
            template: k1s0_generator::manifest::TemplateInfo {
                name: "backend-rust".to_string(),
                version: "0.1.0".to_string(),
                source: "local".to_string(),
                path: "test".to_string(),
                revision: None,
                fingerprint: "abcd1234".to_string(),
            },
            service: k1s0_generator::manifest::ServiceInfo {
                service_name: "test".to_string(),
                language: "rust".to_string(),
                service_type: "backend".to_string(),
                framework: None,
            },
            layer: LayerType::Feature,
            domain: None,
            version: None,
            domain_version: None,
            min_framework_version: None,
            breaking_changes: None,
            deprecated: None,
            generated_at: "2026-01-28T00:00:00Z".to_string(),
            managed_paths: vec![],
            protected_paths: vec![],
            update_policy: std::collections::HashMap::new(),
            checksums: std::collections::HashMap::new(),
            dependencies: None,
        };

        update_manifest_domain(&mut manifest, "test-domain", "^1.0.0");

        // 新形式: manifest.domain と manifest.domain_version がセット
        assert_eq!(manifest.domain, Some("test-domain".to_string()));
        assert_eq!(manifest.domain_version, Some("^1.0.0".to_string()));

        // dependencies.domain も HashMap 形式でセット
        assert!(manifest.dependencies.is_some());
        let deps = manifest.dependencies.unwrap();
        assert!(deps.domain.is_some());
        let domain_deps = deps.domain.unwrap();
        assert_eq!(domain_deps.get("test-domain"), Some(&"^1.0.0".to_string()));
    }

    #[test]
    fn test_update_manifest_domain_existing() {
        let mut old_domain_deps = std::collections::HashMap::new();
        old_domain_deps.insert("old-domain".to_string(), "^0.1.0".to_string());

        let mut manifest = k1s0_generator::manifest::Manifest {
            schema_version: "1.0.0".to_string(),
            k1s0_version: "0.1.0".to_string(),
            template: k1s0_generator::manifest::TemplateInfo {
                name: "backend-rust".to_string(),
                version: "0.1.0".to_string(),
                source: "local".to_string(),
                path: "test".to_string(),
                revision: None,
                fingerprint: "abcd1234".to_string(),
            },
            service: k1s0_generator::manifest::ServiceInfo {
                service_name: "test".to_string(),
                language: "rust".to_string(),
                service_type: "backend".to_string(),
                framework: None,
            },
            layer: LayerType::Feature,
            domain: Some("old-domain".to_string()),
            version: None,
            domain_version: Some("^0.1.0".to_string()),
            min_framework_version: None,
            breaking_changes: None,
            deprecated: None,
            generated_at: "2026-01-28T00:00:00Z".to_string(),
            managed_paths: vec![],
            protected_paths: vec![],
            update_policy: std::collections::HashMap::new(),
            checksums: std::collections::HashMap::new(),
            dependencies: Some(k1s0_generator::manifest::Dependencies {
                framework_crates: vec![],
                framework: vec![],
                domain: Some(old_domain_deps),
            }),
        };

        update_manifest_domain(&mut manifest, "new-domain", "^2.0.0");

        // 新形式: manifest.domain と manifest.domain_version が更新
        assert_eq!(manifest.domain, Some("new-domain".to_string()));
        assert_eq!(manifest.domain_version, Some("^2.0.0".to_string()));

        // dependencies.domain も更新
        let deps = manifest.dependencies.unwrap();
        let domain_deps = deps.domain.unwrap();
        assert_eq!(domain_deps.get("new-domain"), Some(&"^2.0.0".to_string()));
    }
}
