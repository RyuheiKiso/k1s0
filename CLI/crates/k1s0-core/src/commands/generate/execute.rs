// 生成実行のエントリーポイント・オーケストレーションを担当するモジュール。
// 各サブモジュール（template, context, paths, conflict, infra_gen, post_process）を
// 組み合わせて、ひな形生成の全工程を統括する。

use anyhow::Result;
use std::path::Path;

use super::conflict::ensure_generate_targets_available;
use super::context::build_template_context;
use super::infra_gen::{generate_cicd_workflows, generate_helm_chart};
use super::paths::{build_cicd_output_path, build_helm_output_path};
use super::post_process::run_post_processing;
use super::template::{render_scaffold_preview, resolve_template_dir};
use super::types::{GenerateConfig, Kind, Tier};
use crate::commands::template_migrate::parser::{
    collect_project_files, compute_checksum, snapshot_dir, write_manifest, write_snapshot,
    CURRENT_TEMPLATE_VERSION,
};
use crate::commands::template_migrate::types::TemplateManifest;
use crate::config::CliConfig;

// ============================================================================
// 公開エントリーポイント
// ============================================================================

/// ひな形生成を実行する。
///
/// カレントディレクトリを基点として生成を行う。
///
/// # Errors
///
/// カレントディレクトリの取得に失敗した場合、またはひな形生成処理でエラーが発生した場合にエラーを返す。
pub fn execute_generate(config: &GenerateConfig) -> Result<()> {
    execute_generate_with_config(config, &std::env::current_dir()?, &CliConfig::default())
}

/// 指定されたベースディレクトリを起点にひな形生成を実行する。
/// GUI などのワークスペース明示呼び出し向け。
///
/// # Errors
///
/// ディレクトリの作成またはファイルの書き込みに失敗した場合にエラーを返す。
pub fn execute_generate_at(config: &GenerateConfig, base_dir: &Path) -> Result<()> {
    execute_generate_with_config(config, base_dir, &CliConfig::default())
}

/// `CliConfig` を指定してひな形生成を実行する。
/// テンプレートエンジン + 後処理コマンド付き。
///
/// 実行フロー:
/// 1. 競合チェック
/// 2. テンプレートディレクトリ解決
/// 3. scaffold プレビュー生成
/// 4. テンプレートマニフェスト・スナップショット書き込み
/// 5. Helm Chart 生成（server のみ）
/// 6. CI/CD ワークフロー生成（全 kind）
/// 7. 後処理コマンド実行
///
/// # Errors
///
/// ディレクトリの作成、ファイルの書き込み、またはテンプレートのレンダリングに失敗した場合にエラーを返す。
pub fn execute_generate_with_config(
    config: &GenerateConfig,
    base_dir: &Path,
    cli_config: &CliConfig,
) -> Result<()> {
    // 1. 競合チェック
    ensure_generate_targets_available(config, base_dir)?;

    // 2. テンプレートディレクトリ解決
    let tpl_dir = resolve_template_dir(base_dir);

    // 3. scaffold プレビュー生成
    let output_path = render_scaffold_preview(config, base_dir, cli_config, &tpl_dir)?;

    // 4. テンプレートマニフェスト・スナップショット書き込み
    let template_context = build_template_context(config, cli_config);
    let generated_files = collect_project_files(&output_path)?;
    let checksum = compute_checksum(&output_path, &generated_files)?;
    let manifest = TemplateManifest::from_generate_config(
        config,
        cli_config,
        CURRENT_TEMPLATE_VERSION,
        &checksum,
    );
    let snapshot_path = snapshot_dir(&output_path, &checksum);
    write_snapshot(&output_path, &generated_files, &snapshot_path)?;
    write_manifest(&output_path, &manifest)?;

    // 5. Helm Chart 生成（server のみ）
    if config.kind == Kind::Server {
        let helm_output = build_helm_output_path(config, base_dir);
        if let Some(ref ctx) = template_context {
            let helm_tpl_dir = tpl_dir.join("helm");
            if helm_tpl_dir.exists() {
                match generate_helm_chart(&helm_tpl_dir, ctx, &helm_output) {
                    Ok(files) => {
                        println!("Helm Chart を生成しました: {} ファイル", files.len());
                    }
                    Err(e) => {
                        eprintln!("Helm Chart の生成に失敗しました: {e}");
                    }
                }
            }
        }
    }

    // 6. CI/CD ワークフロー生成（全 kind）
    {
        let cicd_output = build_cicd_output_path(config, base_dir);
        if let Some(ref ctx) = template_context {
            let cicd_tpl_dir = tpl_dir.join("cicd");
            if cicd_tpl_dir.exists() {
                match generate_cicd_workflows(&cicd_tpl_dir, ctx, config, &cicd_output) {
                    Ok(files) => {
                        println!("CI/CD ワークフローを生成しました: {} ファイル", files.len());
                    }
                    Err(e) => {
                        eprintln!("CI/CD ワークフローの生成に失敗しました: {e}");
                    }
                }
            }
        }
    }

    // 7. 後処理コマンドの実行（best-effort）
    run_post_processing(config, &output_path);

    Ok(())
}

// ============================================================================
// 配置先スキャン
// ============================================================================

/// 指定された Tier のディレクトリをスキャンし、既存の配置先名を返す。
///
/// Business/Service Tier の場合は `regions/<tier>/` 配下のサブディレクトリを列挙する。
/// System Tier の場合は空ベクタを返す（配置先の概念がないため）。
pub fn scan_placements_at(tier: &Tier, base_dir: &Path) -> Vec<String> {
    let sub = match tier {
        Tier::Business => "regions/business",
        Tier::Service => "regions/service",
        Tier::System => return Vec::new(),
    };
    let path = base_dir.join(sub);
    if !path.is_dir() {
        return Vec::new();
    }
    let mut names = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    names.push(name.to_string());
                }
            }
        }
    }
    names.sort();
    names
}

/// カレントディレクトリを基点に配置先をスキャンする。
pub fn scan_placements(tier: &Tier) -> Vec<String> {
    scan_placements_at(tier, Path::new("."))
}
