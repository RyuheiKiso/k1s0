// 生成先の競合検出を担当するモジュール。
// 既存のディレクトリやファイルとの衝突を事前にチェックし、
// 上書き事故を防止する。

use anyhow::{bail, Result};
use std::path::Path;

use super::paths::{
    build_ci_workflow_path, build_deploy_workflow_path, build_helm_output_path, build_output_path,
};
use super::types::{GenerateConfig, Kind};

/// 生成先に既存アセットとの競合がないかチェックし、競合パスのリストを返す。
///
/// チェック対象:
/// - メイン出力ディレクトリ
/// - CI ワークフローファイル
/// - Helm Chart ディレクトリ（server のみ）
/// - Deploy ワークフローファイル（server のみ）
pub fn find_generate_conflicts_at(config: &GenerateConfig, base_dir: &Path) -> Vec<String> {
    let mut conflicts = Vec::new();

    // 競合チェック対象パスの収集
    let mut reserved_paths = vec![
        build_output_path(config, base_dir),
        build_ci_workflow_path(config, base_dir),
    ];

    // server の場合は Helm Chart パスも確認
    if config.kind == Kind::Server {
        reserved_paths.push(build_helm_output_path(config, base_dir));
    }

    // server の場合は Deploy ワークフローパスも確認
    if let Some(deploy_workflow_path) = build_deploy_workflow_path(config, base_dir) {
        reserved_paths.push(deploy_workflow_path);
    }

    // 存在するパスを競合として記録
    for path in reserved_paths {
        if path.exists() {
            conflicts.push(path.to_string_lossy().replace('\\', "/"));
        }
    }

    conflicts.sort();
    conflicts.dedup();
    conflicts
}

/// 生成先に競合がある場合はエラーを返す。
///
/// # Errors
///
/// 既存アセットが存在する場合にエラーを返す。
pub fn ensure_generate_targets_available(config: &GenerateConfig, base_dir: &Path) -> Result<()> {
    let conflicts = find_generate_conflicts_at(config, base_dir);
    if conflicts.is_empty() {
        return Ok(());
    }

    bail!("generated assets already exist: {}", conflicts.join(", "));
}
