use anyhow::Result;
use dialoguer::Input;

use crate::prompt;

/// CLI-004 監査対応: `--file` フラグによる非インタラクティブバリデーションを実装する。
/// `--file` が指定されている場合は対話プロンプトをスキップしてバリデーションを実行する。
/// `--type` が未指定の場合はファイル拡張子から推定する。
///
/// # Errors
///
/// - `--file` 引数のパスが解決できない場合にエラーを返す
/// - バリデーション型の推定に失敗した場合にエラーを返す
/// - バリデーション処理自体が失敗した場合にエラーを返す
// MED-006 監査対応: CLI 引数パーサーから所有権付きで渡されるため needless_pass_by_value を抑制する
#[allow(clippy::needless_pass_by_value)]
pub fn run_with_args(
    file: Option<std::path::PathBuf>,
    validate_type: Option<String>,
) -> Result<()> {
    let Some(file_path) = file else {
        // --file なし → 従来のインタラクティブモード
        return run();
    };

    // ファイルが存在するか確認する
    let canonical = file_path
        .canonicalize()
        .map_err(|e| anyhow::anyhow!("パスの解決に失敗しました: {}: {e}", file_path.display()))?;
    let canonical_str = canonical.to_str().ok_or_else(|| {
        anyhow::anyhow!("パスの文字列変換に失敗しました: {}", canonical.display())
    })?;

    // --type が未指定の場合はファイル名から推定する
    let vtype = validate_type.as_deref().unwrap_or_else(|| {
        let name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.contains("navigation") {
            "navigation"
        } else {
            "config-schema"
        }
    });

    let errors = match vtype {
        "config-schema" => {
            k1s0_core::commands::validate::config_schema::validate_config_schema(canonical_str)
                .map_err(|e| anyhow::anyhow!("{e}"))?
        }
        "navigation" => {
            k1s0_core::commands::validate::navigation::validate_navigation(canonical_str)
                .map_err(|e| anyhow::anyhow!("{e}"))?
        }
        other => anyhow::bail!(
            "不明なバリデーション種別: '{other}'。'config-schema' または 'navigation' を指定してください。"
        ),
    };

    if errors == 0 {
        println!("\nバリデーション完了: エラーなし");
    } else {
        println!("\nバリデーション完了: {errors} 件のエラー");
    }
    Ok(())
}

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
            // CLI-MED-002 監査対応: canonicalize でパストラバーサル攻撃を防止する。
            let canonical = std::path::Path::new(&path)
                .canonicalize()
                // uninlined_format_args 対応: 変数を直接フォーマット文字列に埋め込む
                .map_err(|e| anyhow::anyhow!("パスの解決に失敗しました: {path}: {e}"))?;
            let canonical_str = canonical
                .to_str()
                // unnecessary_debug_formatting 対応: PathBuf は Display が実装されているため {} を使用する
                .ok_or_else(|| {
                    anyhow::anyhow!("パスの文字列変換に失敗しました: {}", canonical.display())
                })?;
            let errors =
                k1s0_core::commands::validate::config_schema::validate_config_schema(canonical_str)
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
            // CLI-MED-002 監査対応: canonicalize でパストラバーサル攻撃を防止する。
            let canonical = std::path::Path::new(&path)
                .canonicalize()
                // uninlined_format_args 対応: 変数を直接フォーマット文字列に埋め込む
                .map_err(|e| anyhow::anyhow!("パスの解決に失敗しました: {path}: {e}"))?;
            let canonical_str = canonical
                .to_str()
                // unnecessary_debug_formatting 対応: PathBuf は Display が実装されているため {} を使用する
                .ok_or_else(|| {
                    anyhow::anyhow!("パスの文字列変換に失敗しました: {}", canonical.display())
                })?;
            let errors =
                k1s0_core::commands::validate::navigation::validate_navigation(canonical_str)
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
