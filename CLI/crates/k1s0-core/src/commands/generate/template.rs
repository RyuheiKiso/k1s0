// テンプレート解決・レンダリング・BFF生成・正規化を担当するモジュール。
// テンプレートディレクトリの探索、scaffold の生成、BFF 層の追加生成、
// DB/ライブラリ固有のレイアウト正規化を行う。

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use super::context::build_template_context;
use super::paths::build_output_path;
use super::scaffold::{generate_client, generate_database, generate_library, generate_server};
use super::types::{ApiStyle, GenerateConfig, Kind, LangFw, Language, Tier};
use crate::config::CliConfig;
use crate::template::context::TemplateContextBuilder;
use crate::template::TemplateEngine;

// ============================================================================
// テンプレートディレクトリ解決
// ============================================================================

/// テンプレートディレクトリのパスを解決する。
///
/// ベースディレクトリから祖先を辿り、既知のテンプレートディレクトリ候補を探索する。
/// 見つからない場合は `CARGO_MANIFEST_DIR` 基準のフォールバックを試みる。
pub(crate) fn resolve_template_dir(base_dir: &Path) -> PathBuf {
    // ベースディレクトリの祖先を順に探索
    // RUST-005 監査対応: canonicalize() で symlink を解決し、祖先ディレクトリの配下であることを検証する。
    // symlink 経由で意図しないテンプレートディレクトリが使用されるパストラバーサルリスクを防止する。
    for ancestor in base_dir.ancestors() {
        for candidate in template_dir_candidates(ancestor) {
            if is_template_dir(&candidate) {
                if let (Ok(canonical_candidate), Ok(canonical_ancestor)) =
                    (candidate.canonicalize(), ancestor.canonicalize())
                {
                    if canonical_candidate.starts_with(&canonical_ancestor) {
                        return candidate;
                    }
                }
            }
        }
    }

    // CARGO_MANIFEST_DIR からの探索（ビルド時パス）
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    for candidate in template_dir_candidates(manifest_path) {
        if is_template_dir(&candidate) {
            return candidate;
        }
    }

    // 兄弟クレートのテンプレートディレクトリ
    let sibling_cli_templates = manifest_path.join("..").join("k1s0-cli").join("templates");
    if is_template_dir(&sibling_cli_templates) {
        return sibling_cli_templates;
    }

    // テンプレートディレクトリが見つからなかった場合に警告を出力する
    // ユーザーがテンプレートなしでのインライン生成であることを認識できるようにする
    eprintln!(
        "警告: テンプレートディレクトリが見つかりません。インライン生成にフォールバックします。\
        \nWarning: Template directory not found, falling back to inline generation."
    );

    // 最終フォールバック
    base_dir
        .join("CLI")
        .join("crates")
        .join("k1s0-cli")
        .join("templates")
}

/// テンプレートディレクトリの候補パスを生成する。
fn template_dir_candidates(root: &Path) -> [PathBuf; 4] {
    [
        root.join("CLI")
            .join("crates")
            .join("k1s0-cli")
            .join("templates"),
        root.join("CLI").join("templates"),
        root.join("crates").join("k1s0-cli").join("templates"),
        root.join("templates"),
    ]
}

/// 候補ディレクトリが有効なテンプレートディレクトリかどうかを判定する。
///
/// server, client, library, database, bff の5種類のサブディレクトリが揃っている必要がある。
fn is_template_dir(candidate: &Path) -> bool {
    candidate.join("server").is_dir()
        && candidate.join("client").is_dir()
        && candidate.join("library").is_dir()
        && candidate.join("database").is_dir()
        && candidate.join("bff").is_dir()
}

// ============================================================================
// scaffold レンダリング
// ============================================================================

/// プロジェクト本体の scaffold を生成する。
///
/// Helm/CI/CD や後処理は行わず、対象モジュール配下のみを作成する。
///
/// # Errors
///
/// ディレクトリ作成またはファイル生成に失敗した場合にエラーを返す。
pub(crate) fn render_scaffold_preview(
    config: &GenerateConfig,
    base_dir: &Path,
    cli_config: &CliConfig,
    template_dir: &Path,
) -> Result<PathBuf> {
    let output_path = build_output_path(config, base_dir);
    fs::create_dir_all(&output_path)?;

    // テンプレートディレクトリが存在しない場合はインライン生成にフォールバックする
    if !template_dir.exists() {
        eprintln!(
            "警告: テンプレートディレクトリ '{}' が存在しません。インライン生成を使用します。",
            template_dir.display()
        );
    }

    // テンプレートエンジンでの生成を試みる
    let template_generated = if template_dir.exists() {
        try_generate_from_templates(config, &output_path, template_dir, cli_config)
    } else {
        false
    };

    // テンプレートが見つからない場合はインライン生成にフォールバック
    if !template_generated {
        generate_inline_scaffold(config, &output_path)?;
    }

    // BFF 層の追加生成（条件を満たす場合のみ）
    render_bff_if_needed(config, template_dir, &output_path)?;
    // DB/ライブラリ固有のレイアウト正規化
    normalize_generated_scaffold(config, &output_path)?;

    Ok(output_path)
}

/// テンプレートエンジンを使って生成を試みる。成功した場合 true を返す。
fn try_generate_from_templates(
    config: &GenerateConfig,
    output_path: &Path,
    template_dir: &Path,
    cli_config: &CliConfig,
) -> bool {
    let Some(ctx) = build_template_context(config, cli_config) else {
        return false;
    };

    // kind + language/framework に対応するテンプレートサブディレクトリの存在確認
    // client の場合はフレームワーク名 (react/flutter) でディレクトリを引く
    let sub_dir = if ctx.kind == "client" && !ctx.framework.is_empty() {
        &ctx.framework
    } else {
        &ctx.language
    };
    let kind_lang_dir = template_dir.join(&ctx.kind).join(sub_dir);
    if !kind_lang_dir.exists() {
        return false;
    }

    let Ok(mut engine) = TemplateEngine::new(template_dir) else {
        return false;
    };

    match engine.render_to_dir(&ctx, output_path) {
        Ok(files) => !files.is_empty(),
        // テンプレートレンダリングエラーを明示的にログ出力する（M-16）
        Err(e) => {
            eprintln!("テンプレート条件評価エラー: {e}");
            false
        }
    }
}

/// インライン scaffold を生成する（テンプレートエンジンなし）。
fn generate_inline_scaffold(config: &GenerateConfig, output_path: &Path) -> Result<()> {
    match config.kind {
        Kind::Server => generate_server(config, output_path)?,
        Kind::Client => generate_client(config, output_path)?,
        Kind::Library => generate_library(config, output_path)?,
        Kind::Database => generate_database(config, output_path)?,
    }

    Ok(())
}

// ============================================================================
// BFF 生成
// ============================================================================

/// BFF（Backend For Frontend）のテンプレートを生成する。
///
/// 以下の条件をすべて満たす場合のみ生成する:
/// - kind が Server
/// - tier が Service
/// - `api_styles` に GraphQL を含む
/// - `bff_language` が指定されている
fn render_bff_if_needed(
    config: &GenerateConfig,
    template_dir: &Path,
    output_path: &Path,
) -> Result<()> {
    // BFF 生成条件の判定
    if config.kind != Kind::Server
        || config.tier != Tier::Service
        || !config.detail.api_styles.contains(&ApiStyle::GraphQL)
    {
        return Ok(());
    }

    let Some(bff_lang) = config.detail.bff_language else {
        return Ok(());
    };

    // BFF テンプレートディレクトリの存在確認
    let bff_tpl_dir = template_dir.join("bff").join(bff_lang.dir_name());
    if !bff_tpl_dir.exists() {
        return Ok(());
    }

    // BFF 用のテンプレートコンテキストを構築してレンダリング
    let bff_path = output_path.join("bff");
    fs::create_dir_all(&bff_path)?;

    // H-13 監査対応: deprecated な build() の代わりに try_build() を使用して panic を防ぐ。
    // バリデーションエラーが発生した場合は BFF 生成をスキップして正常復帰する。
    let bff_ctx = match TemplateContextBuilder::new(
        config.detail.name.as_deref().unwrap_or("service"),
        config.tier.as_str(),
        bff_lang.dir_name(),
        "bff",
    )
    .api_style("graphql")
    .try_build()
    {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("BFF テンプレートコンテキストのビルドに失敗しました: {e}");
            return Ok(());
        }
    };

    match TemplateEngine::new(template_dir) {
        Ok(mut engine) => {
            let _ = engine.render_to_dir(&bff_ctx, &bff_path);
        }
        Err(error) => {
            eprintln!("BFF テンプレートエンジンの初期化に失敗しました: {error}");
        }
    }

    Ok(())
}

// ============================================================================
// レイアウト正規化
// ============================================================================

/// 生成された scaffold のレイアウトを正規化する。
///
/// DB とライブラリに対して、規約に準拠したディレクトリ構造に整える。
fn normalize_generated_scaffold(config: &GenerateConfig, output_path: &Path) -> Result<()> {
    match config.kind {
        Kind::Database => normalize_database_layout(config, output_path),
        Kind::Library => normalize_library_layout(config, output_path),
        _ => Ok(()),
    }
}

/// データベース scaffold のレイアウトを正規化する。
///
/// - migrations/, seeds/, schema/ ディレクトリを作成
/// - ルート直下のマイグレーションファイルを migrations/ に移動
/// - database.yaml メタデータファイルを生成
fn normalize_database_layout(config: &GenerateConfig, output_path: &Path) -> Result<()> {
    let LangFw::Database { name, rdbms } = &config.lang_fw else {
        return Ok(());
    };

    let migrations_dir = output_path.join("migrations");
    fs::create_dir_all(&migrations_dir)?;
    fs::create_dir_all(output_path.join("seeds"))?;
    fs::create_dir_all(output_path.join("schema"))?;

    // ルート直下のマイグレーションファイルを正しい場所に移動
    for file_name in ["001_init.up.sql", "001_init.down.sql"] {
        let root_path = output_path.join(file_name);
        let migration_path = migrations_dir.join(file_name);
        if root_path.is_file() && !migration_path.exists() {
            fs::rename(root_path, migration_path)?;
        }
    }

    // database.yaml メタデータの生成
    let database_yaml = output_path.join("database.yaml");
    if !database_yaml.exists() {
        fs::write(
            database_yaml,
            format!("name: {name}\nrdbms: {}\n", rdbms.as_str()),
        )?;
    }

    Ok(())
}

/// ライブラリ scaffold のレイアウトを正規化する（Dart のみ）。
///
/// Dart ライブラリのエントリポイントファイル名を `snake_case` に統一する。
fn normalize_library_layout(config: &GenerateConfig, output_path: &Path) -> Result<()> {
    if !matches!(config.lang_fw, LangFw::Language(Language::Dart)) {
        return Ok(());
    }

    let Some(name) = config.detail.name.as_deref() else {
        return Ok(());
    };

    let module_name = to_snake_case(name);
    let lib_dir = output_path.join("lib");
    fs::create_dir_all(&lib_dir)?;

    // レガシーファイル名（そのまま）を snake_case にリネーム
    let legacy_entry = lib_dir.join(format!("{name}.dart"));
    let expected_entry = lib_dir.join(format!("{module_name}.dart"));

    if legacy_entry.is_file() && !expected_entry.exists() {
        fs::rename(&legacy_entry, &expected_entry)?;
    }

    // エントリポイントが存在しない場合は新規作成
    if !expected_entry.exists() {
        fs::write(
            &expected_entry,
            format!("library {module_name};\n\nexport 'src/{module_name}.dart';\n"),
        )?;
    }

    Ok(())
}

/// 文字列を `snake_case` に変換するヘルパー関数。
fn to_snake_case(value: &str) -> String {
    let mut snake = String::with_capacity(value.len());
    let mut previous_was_separator = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            snake.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator {
            snake.push('_');
            previous_was_separator = true;
        }
    }

    snake.trim_matches('_').to_string()
}
