use anyhow::Result;
use dialoguer::Input;

use crate::prompt;

/// バリデーションコマンドを実行する。
///
/// サブメニューで config-schema / navigation を選択し、
/// 対象ファイルのパスを入力してバリデーションを実行する。
///
/// # Errors
///
/// プロンプトの入出力に失敗した場合にエラーを返す。
pub fn run() -> Result<()> {
    println!("\n--- バリデーション ---\n");

    let Some(idx) = prompt::select_prompt(
        "バリデーション対象を選択してください",
        &["config-schema バリデーション", "navigation バリデーション"],
    )?
    else {
        return Ok(());
    };

    match idx {
        0 => {
            let path: String = Input::with_theme(&prompt::theme())
                .with_prompt("config-schema.yaml のパス")
                .default("config-schema.yaml".to_string())
                .interact_text()?;
            let errors = k1s0_core::commands::validate::config_schema::validate_config_schema(
                &path,
            )
            .map_err(|e| anyhow::anyhow!("{e}"))?;
            if errors == 0 {
                println!("\nバリデーション完了: エラーなし");
            } else {
                println!("\nバリデーション完了: {errors} 件のエラー");
            }
        }
        1 => {
            let path: String = Input::with_theme(&prompt::theme())
                .with_prompt("navigation.yaml のパス")
                .default("navigation.yaml".to_string())
                .interact_text()?;
            let errors =
                k1s0_core::commands::validate::navigation::validate_navigation(&path)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
            if errors == 0 {
                println!("\nバリデーション完了: エラーなし");
            } else {
                println!("\nバリデーション完了: {errors} 件のエラー");
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}
