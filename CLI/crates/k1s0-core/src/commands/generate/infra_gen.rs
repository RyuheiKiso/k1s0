// Helm Chart / CI/CD ワークフロー生成を担当するモジュール。
// テンプレートエンジン (Tera) を使って、インフラ関連ファイルをレンダリングする。

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use super::paths::{build_ci_workflow_path, build_deploy_workflow_path};
use super::types::{GenerateConfig, Kind};
use crate::template::TemplateEngine;

/// Helm Chart テンプレートをレンダリングする。
///
/// helm テンプレートディレクトリ配下の `.tera` ファイルを走査し、
/// `TemplateContext` を使ってレンダリングした結果を出力ディレクトリに書き込む。
///
/// # Errors
///
/// テンプレートのパースまたはレンダリングに失敗した場合にエラーを返す。
pub(crate) fn generate_helm_chart(
    helm_tpl_dir: &Path,
    ctx: &crate::template::context::TemplateContext,
    output_dir: &Path,
) -> Result<Vec<PathBuf>> {
    // 親ディレクトリが存在しない場合はエラーを返す（unwrap を避けて安全に伝播）
    let mut engine = TemplateEngine::new(
        helm_tpl_dir
            .parent()
            .ok_or_else(|| anyhow::anyhow!("helm_tpl_dir に親ディレクトリがありません"))?,
    )?;
    // helm ディレクトリ直下のテンプレートを直接レンダリング
    let tera_ctx = ctx.to_tera_context();
    let mut generated = Vec::new();

    // テンプレートディレクトリの正規パスを基準とし、パストラバーサルを防止する
    let canonical_tpl_dir = helm_tpl_dir.canonicalize()
        .map_err(|e| anyhow::anyhow!("helm_tpl_dir の canonicalize に失敗しました: {}", e))?;

    // .tera ファイルを再帰的に走査（シンボリックリンクは follow_links=false でスキップ）
    for entry in walkdir::WalkDir::new(helm_tpl_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        // ディレクトリと .tera 以外のファイルはスキップ
        if path.is_dir() || path.extension().and_then(|e| e.to_str()) != Some("tera") {
            continue;
        }

        // テンプレートファイルの正規パスを検証し、テンプレートディレクトリ外への逸脱を防止する
        let canonical_path = path.canonicalize()
            .map_err(|e| anyhow::anyhow!("テンプレートファイルの canonicalize に失敗しました: {}", e))?;
        if !canonical_path.starts_with(&canonical_tpl_dir) {
            return Err(anyhow::anyhow!(
                "テンプレートファイルがテンプレートディレクトリ外を参照しています: {}",
                path.display()
            ));
        }

        // テンプレートの相対パスを算出
        let relative = path.strip_prefix(helm_tpl_dir)?;
        let template_content = fs::read_to_string(path)?;
        let template_name = relative.to_string_lossy().replace('\\', "/");

        // Tera にテンプレートを登録してレンダリング
        engine
            .tera
            .add_raw_template(&template_name, &template_content)?;
        let rendered = engine.tera.render(&template_name, &tera_ctx)?;

        // .tera 拡張子を除去して出力パスを決定
        let output_relative = relative.to_string_lossy().replace('\\', "/");
        let output_relative = if Path::new(&output_relative)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("tera"))
        {
            output_relative[..output_relative.len() - 5].to_string()
        } else {
            output_relative.clone()
        };
        let output_path = output_dir.join(&output_relative);

        // 親ディレクトリを作成してファイルを書き込み
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&output_path, rendered)?;
        generated.push(output_path);
    }

    Ok(generated)
}

/// CI/CD ワークフローテンプレートをレンダリングする。
///
/// CI ワークフロー（全 kind）と Deploy ワークフロー（server のみ）を生成する。
///
/// # Errors
///
/// テンプレートのパースまたはレンダリングに失敗した場合にエラーを返す。
pub(crate) fn generate_cicd_workflows(
    cicd_tpl_dir: &Path,
    ctx: &crate::template::context::TemplateContext,
    config: &GenerateConfig,
    output_dir: &Path,
) -> Result<Vec<PathBuf>> {
    let tera_ctx = ctx.to_tera_context();
    let mut generated = Vec::new();
    let mut tera = tera::Tera::default();
    crate::template::filters::register_filters(&mut tera);

    // CI ワークフロー（全 kind 共通）
    let ci_template = cicd_tpl_dir.join("ci.yaml.tera");
    if ci_template.exists() {
        let template_content = fs::read_to_string(&ci_template)?;
        tera.add_raw_template("ci.yaml", &template_content)?;
        let rendered = tera.render("ci.yaml", &tera_ctx)?;

        // 出力先パスの算出（output_dir の2階層上がベースディレクトリ）
        let output_path = build_ci_workflow_path(
            config,
            output_dir
                .parent()
                .ok_or_else(|| anyhow::anyhow!("output_dir に親ディレクトリがありません"))?
                .parent()
                .ok_or_else(|| anyhow::anyhow!("output_dir の祖父ディレクトリがありません"))?,
        );
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&output_path, rendered)?;
        generated.push(output_path);
    }

    // Deploy ワークフロー（server のみ）
    if config.kind == Kind::Server {
        let deploy_template = cicd_tpl_dir.join("deploy.yaml.tera");
        if deploy_template.exists() {
            let template_content = fs::read_to_string(&deploy_template)?;
            tera.add_raw_template("deploy.yaml", &template_content)?;
            let rendered = tera.render("deploy.yaml", &tera_ctx)?;

            // Deploy ワークフローの出力先パスを2階層上のベースディレクトリから算出する
            let output_path = build_deploy_workflow_path(
                config,
                output_dir
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("output_dir に親ディレクトリがありません"))?
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("output_dir の祖父ディレクトリがありません"))?,
            )
            .expect("server deploy workflow path should exist");
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&output_path, rendered)?;
            generated.push(output_path);
        }
    }

    Ok(generated)
}
