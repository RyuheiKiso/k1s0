// 出力先パス構築全般を担当するモジュール。
// 生成対象（サーバー、クライアント、ライブラリ、データベース）の
// ディレクトリ配置規約に従い、適切なパスを組み立てる。

use std::path::{Path, PathBuf};

use super::types::{GenerateConfig, Kind, LangFw, Tier};

// ============================================================================
// メイン出力パス
// ============================================================================

/// 生成対象モジュールの出力先パスを構築する。
///
/// `regions/<tier>/[placement]/<kind>/<lang>/[name]` の規約に基づく。
pub fn build_output_path(config: &GenerateConfig, base_dir: &Path) -> PathBuf {
    let mut path = base_dir.join("regions");
    path.push(config.tier.as_str());

    // 配置先ドメインディレクトリ
    if let Some(ref placement) = config.placement {
        path.push(placement);
    }

    // 種別ごとのサブディレクトリ構築
    match config.kind {
        Kind::Server => {
            path.push("server");
            if let LangFw::Language(lang) = config.lang_fw {
                path.push(lang.dir_name());
            }
            // system / business の場合はサービス名をサブディレクトリに
            if config.tier != Tier::Service {
                if let Some(ref name) = config.detail.name {
                    path.push(name);
                }
            }
        }
        Kind::Client => {
            path.push("client");
            if let LangFw::Framework(fw) = config.lang_fw {
                path.push(fw.dir_name());
            }
            // business の場合はアプリ名をサブディレクトリに
            if config.tier == Tier::Business {
                if let Some(ref name) = config.detail.name {
                    path.push(name);
                }
            }
        }
        Kind::Library => {
            path.push("library");
            if let LangFw::Language(lang) = config.lang_fw {
                path.push(lang.dir_name());
            }
            if let Some(ref name) = config.detail.name {
                path.push(name);
            }
        }
        Kind::Database => {
            path.push("database");
            if let LangFw::Database { ref name, .. } = config.lang_fw {
                path.push(name);
            }
        }
    }

    path
}

/// モジュール識別子を生成する（CI/CD ワークフローファイル名などに使用）。
///
/// 出力パスからリポジトリ相対パスを取得し、スラッシュをハイフンに変換する。
pub(crate) fn generated_module_identifier(config: &GenerateConfig) -> String {
    let relative = build_output_path(config, Path::new(""))
        .to_string_lossy()
        .replace('\\', "/");
    let relative = relative.strip_prefix("regions/").unwrap_or(&relative);
    relative.replace('/', "-")
}

// ============================================================================
// CI/CD・Helm 出力パス
// ============================================================================

/// CI ワークフローファイルのパスを構築する。
pub(crate) fn build_ci_workflow_path(config: &GenerateConfig, base_dir: &Path) -> PathBuf {
    let file_name = format!("{}-ci.yaml", generated_module_identifier(config));
    build_cicd_output_path(config, base_dir).join(file_name)
}

/// Deploy ワークフローファイルのパスを構築する（server のみ）。
pub(crate) fn build_deploy_workflow_path(
    config: &GenerateConfig,
    base_dir: &Path,
) -> Option<PathBuf> {
    (config.kind == Kind::Server).then(|| {
        let file_name = format!("{}-deploy.yaml", generated_module_identifier(config));
        build_cicd_output_path(config, base_dir).join(file_name)
    })
}

/// Helm Chart の出力先パスを構築する。
pub(crate) fn build_helm_output_path(config: &GenerateConfig, base_dir: &Path) -> PathBuf {
    let mut path = base_dir.join("infra").join("helm").join("services");
    path.push(config.tier.as_str());

    // business Tier の場合はドメインディレクトリを挟む
    if config.tier == Tier::Business {
        if let Some(ref placement) = config.placement {
            path.push(placement);
        }
    }

    // サービス名
    if let Some(ref name) = config.detail.name {
        path.push(name);
    }

    path
}

/// CI/CD ワークフローの出力先ディレクトリパスを構築する。
pub(crate) fn build_cicd_output_path(_config: &GenerateConfig, base_dir: &Path) -> PathBuf {
    base_dir.join(".github").join("workflows")
}
