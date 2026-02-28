use anyhow::Result;
use dialoguer::Input;
use std::fs;

use crate::prompt;
use k1s0_core::commands::generate::navigation::{generate_dart_routes, generate_typescript_routes};
use k1s0_core::commands::validate::navigation::NavigationYaml;

/// ナビゲーション型ファイル生成コマンドを実行する。
///
/// navigation.yaml を読み込み、TypeScript / Dart のルート型定義ファイルを生成する。
///
/// # Errors
///
/// プロンプトの入出力・ファイル操作・YAML パースに失敗した場合にエラーを返す。
pub fn run() -> Result<()> {
    println!("\n--- ナビゲーション型ファイル生成 ---\n");

    // Step 1: navigation.yaml のパス
    let nav_path: String = Input::with_theme(&prompt::theme())
        .with_prompt("navigation.yaml のパス")
        .default("navigation.yaml".to_string())
        .interact_text()?;

    let content = fs::read_to_string(&nav_path)
        .map_err(|e| anyhow::anyhow!("{}: {e}", nav_path))?;
    let nav: NavigationYaml = serde_yaml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("navigation.yaml のパースエラー: {e}"))?;

    // Step 2: 対象フレームワーク
    let Some(idx) = prompt::select_prompt(
        "対象フレームワークを選択してください",
        &["React (TypeScript)", "Flutter (Dart)", "両方"],
    )?
    else {
        return Ok(());
    };

    let gen_react = idx == 0 || idx == 2;
    let gen_flutter = idx == 1 || idx == 2;

    // 確認
    println!("\n[確認] 以下の内容で実行します。よろしいですか？");
    println!("  ナビゲーション: {} ({} ルート)", nav_path, nav.routes.len());
    if gen_react {
        println!("  React  → src/navigation/__generated__/route-types.ts");
    }
    if gen_flutter {
        println!("  Flutter → lib/navigation/__generated__/route_ids.dart");
    }

    match prompt::confirm_prompt()? {
        prompt::ConfirmResult::Yes => {}
        _ => {
            println!("キャンセルしました。");
            return Ok(());
        }
    }

    // 生成
    println!("\n型定義ファイルを生成中...");
    if gen_react {
        let ts = generate_typescript_routes(&nav);
        let out_path = "src/navigation/__generated__/route-types.ts";
        fs::create_dir_all("src/navigation/__generated__")?;
        fs::write(out_path, ts)?;
        println!("  ✅ {out_path}");
    }
    if gen_flutter {
        let dart = generate_dart_routes(&nav);
        let out_path = "lib/navigation/__generated__/route_ids.dart";
        fs::create_dir_all("lib/navigation/__generated__")?;
        fs::write(out_path, dart)?;
        println!("  ✅ {out_path}");
    }

    println!("\nナビゲーション型ファイルの生成が完了しました。");
    Ok(())
}
