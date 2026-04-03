use anyhow::Result;
use dialoguer::{Input, MultiSelect};
use std::path::{Path, PathBuf};

use crate::prompt;
use k1s0_core::commands::generate::config_types::{
    load_validated_schema_from_file, push_config_schema, write_generated_types_to_targets,
    GeneratedTypesTarget,
};

#[allow(clippy::missing_errors_doc, clippy::too_many_lines)]
pub fn run() -> Result<()> {
    // 非インタラクティブ環境（CI/CD、非TTY）では対話的プロンプトが使用できないため早期終了する
    if crate::prompt::is_non_interactive() {
        eprintln!("このコマンドは対話的な入力が必要です。TTY環境で実行してください。");
        return Err(anyhow::anyhow!("非インタラクティブ環境では実行できません: K1S0_NON_INTERACTIVE が設定されているか TTY が割り当てられていません"));
    }

    // 設定スキーマ型生成コマンドのメインエントリポイント
    println!("\n--- 設定スキーマ型生成 ---\n");

    // config-schema.yaml のパスをユーザーに入力させる
    let schema_path: String = Input::with_theme(&prompt::theme())
        .with_prompt("config-schema.yaml のパス")
        .default("config-schema.yaml".to_string())
        .interact_text()?;
    // HIGH-A2 監査対応: ユーザー入力パスを canonicalize してパストラバーサルを防止する。
    // ../../../etc/passwd のような traversal を防ぐため、プロジェクトディレクトリ内のパスのみを許可する。
    let schema_path_buf = Path::new(&schema_path).canonicalize().map_err(|e| {
        anyhow::anyhow!("スキーマファイルへのアクセスに失敗しました '{schema_path}': {e}")
    })?;
    let cwd = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("カレントディレクトリの取得に失敗しました: {e}"))?;
    if !schema_path_buf.starts_with(&cwd) {
        anyhow::bail!(
            "セキュリティエラー: スキーマファイルはプロジェクトディレクトリ内にある必要があります: {schema_path}"
        );
    }
    let schema = load_validated_schema_from_file(&schema_path_buf)
        .map_err(|error| anyhow::anyhow!("{schema_path}: {error}"))?;

    // 生成ターゲット（React / Flutter）をユーザーに選択させる
    let selections = MultiSelect::with_theme(&prompt::theme())
        .with_prompt("生成ターゲットを選択してください")
        .items(&["React (TypeScript)", "Flutter (Dart)"])
        .interact()?;
    if selections.is_empty() {
        return Ok(());
    }

    // 選択されたターゲットの出力ディレクトリをユーザーに入力させる
    let mut output_dirs: Vec<(String, PathBuf)> = Vec::new();
    if selections.contains(&0) {
        let output_dir: String = Input::with_theme(&prompt::theme())
            .with_prompt("React 出力ディレクトリ")
            .default("src/config/__generated__".to_string())
            .interact_text()?;
        output_dirs.push(("typescript".to_string(), PathBuf::from(output_dir)));
    }
    if selections.contains(&1) {
        let output_dir: String = Input::with_theme(&prompt::theme())
            .with_prompt("Flutter 出力ディレクトリ")
            .default("lib/config/__generated__".to_string())
            .interact_text()?;
        output_dirs.push(("dart".to_string(), PathBuf::from(output_dir)));
    }

    // config サーバーへのプッシュ有無をユーザーに確認する
    let Some(push) = prompt::yes_no_prompt("config サーバーにスキーマをプッシュしますか？")?
    else {
        return Ok(());
    };

    // プッシュする場合は config サーバーの URL を入力させる
    let server_url = if push {
        Some(
            Input::with_theme(&prompt::theme())
                .with_prompt("config サーバーの URL")
                .default("http://localhost:8080".to_string())
                .interact_text()?,
        )
    } else {
        None
    };

    // 実行前に選択内容を確認表示する
    println!("\n[確認]");
    println!("  スキーマ: {schema_path} ({})", schema.service);
    if let Some(url) = &server_url {
        println!("  プッシュ先: {url}");
    }
    for (target, output_dir) in &output_dirs {
        let file_name = match target.as_str() {
            "typescript" => "config-types.ts",
            "dart" => "config_types.dart",
            _ => continue,
        };
        println!("  {target}: {}", output_dir.join(file_name).display());
    }

    if prompt::confirm_prompt()? != prompt::ConfirmResult::Yes {
        println!("キャンセルしました。");
        return Ok(());
    }

    // config サーバーへのスキーマプッシュ処理
    if let Some(url) = &server_url {
        let token = std::env::var("K1S0_TOKEN")
            .map_err(|_| anyhow::anyhow!("プッシュが有効な場合は K1S0_TOKEN 環境変数が必要です"))?;
        println!("\nスキーマをプッシュしています...");
        push_config_schema(&schema, url, &token).map_err(|error| anyhow::anyhow!("{error}"))?;
        println!(
            "  OK スキーマを登録しました: {} ({} カテゴリ, {} フィールド)",
            schema.service,
            schema.categories.len(),
            schema
                .categories
                .iter()
                .map(|category| category.fields.len())
                .sum::<usize>()
        );
    }

    // 型定義ファイルの生成処理
    let target_specs = output_dirs
        .iter()
        .map(|(target, output_dir)| GeneratedTypesTarget {
            target: target.as_str(),
            output_dir: output_dir.as_path(),
        })
        .collect::<Vec<_>>();

    println!("\n型定義を生成しています...");
    let generated = write_generated_types_to_targets(&schema, &target_specs)
        .map_err(|error| anyhow::anyhow!("{error}"))?;
    // LOW-006 監査対応: 生成ファイル出力形式を generate events / generate navigation と統一する。
    // 他コマンド（generate_events.rs, generate_navigation.rs）に合わせて ✅ マークを使用する。
    for path in &generated {
        println!("  ✅ {}", path.display());
    }

    println!("\n設定型の生成が完了しました。");
    Ok(())
}
