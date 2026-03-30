use anyhow::Result;
use dialoguer::Input;
use std::fs;
use std::path::PathBuf;

use crate::prompt;
use k1s0_core::commands::generate::navigation::write_generated_routes_from_file;
use k1s0_core::commands::validate::navigation::NavigationYaml;

/// ナビゲーション型ファイル生成コマンドを実行する。
///
/// navigation.yaml を読み込み、TypeScript / Dart のルート型定義ファイルを生成する。
///
/// # Errors
///
/// プロンプトの入出力・ファイル操作・YAML パースに失敗した場合にエラーを返す。
pub fn run() -> Result<()> {
    // 非インタラクティブ環境（CI/CD、非TTY）では対話的プロンプトが使用できないため早期終了する
    if crate::prompt::is_non_interactive() {
        eprintln!("このコマンドは対話的な入力が必要です。TTY環境で実行してください。");
        return Err(anyhow::anyhow!("非インタラクティブ環境では実行できません: K1S0_NON_INTERACTIVE が設定されているか TTY が割り当てられていません"));
    }

    println!("\n--- ナビゲーション型ファイル生成 ---\n");

    // Step 1: navigation.yaml のパス
    let nav_path: String = Input::with_theme(&prompt::theme())
        .with_prompt("navigation.yaml のパス")
        .default("navigation.yaml".to_string())
        .interact_text()?;

    let content = fs::read_to_string(&nav_path).map_err(|e| anyhow::anyhow!("{nav_path}: {e}"))?;
    let nav: NavigationYaml = serde_yaml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("navigation.yaml のパースエラー: {e}"))?;

    // Step 2: 生成ターゲット
    let Some(idx) = prompt::select_prompt(
        "生成ターゲットを選択してください",
        &["TypeScript", "Dart", "両方"],
    )?
    else {
        return Ok(());
    };

    let targets: Vec<&str> = match idx {
        0 => vec!["typescript"],
        1 => vec!["dart"],
        2 => vec!["typescript", "dart"],
        _ => unreachable!(),
    };

    // Step 3: 出力先ディレクトリ
    let default_output_dir = match idx {
        0 => "src/navigation/__generated__",
        1 => "lib/navigation/__generated__",
        _ => "generated/navigation",
    };
    let output_dir: String = Input::with_theme(&prompt::theme())
        .with_prompt("生成先ディレクトリ")
        .default(default_output_dir.to_string())
        .interact_text()?;

    // 確認
    println!("\n[確認] 以下の内容で実行します。よろしいですか？");
    println!(
        "  ナビゲーション: {} ({} ルート)",
        nav_path,
        nav.routes.len()
    );
    println!("  出力先:         {output_dir}");
    for target in &targets {
        match *target {
            "typescript" => println!(
                "  TypeScript      → {}",
                PathBuf::from(&output_dir).join("route-types.ts").display()
            ),
            "dart" => println!(
                "  Dart            → {}",
                PathBuf::from(&output_dir).join("route_ids.dart").display()
            ),
            _ => {}
        }
    }

    if prompt::confirm_prompt()? == prompt::ConfirmResult::Yes {
    } else {
        println!("キャンセルしました。");
        return Ok(());
    }

    // 生成
    println!("\n型定義ファイルを生成中...");
    let generated = write_generated_routes_from_file(
        std::path::Path::new(&nav_path),
        std::path::Path::new(&output_dir),
        &targets,
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    for path in &generated {
        println!("  ✅ {}", path.display());
    }

    println!("\nナビゲーション型ファイルの生成が完了しました。");
    Ok(())
}
