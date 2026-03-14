// 後処理コマンドの決定・実行を担当するモジュール。
// 生成後に必要な依存解決やコード生成コマンドを判定し、
// リトライ付きで実行する。

use std::path::Path;

use super::retry::{run_with_retry, RetryConfig};
use super::types::{ApiStyle, Framework, GenerateConfig, Kind, LangFw, Language};

/// D-08: 後処理コマンドを実行する（best-effort、リトライ付き）。
///
/// 各コマンドの実行に失敗した場合はエラーメッセージを表示し、
/// 手動実行を促すが、全体の処理は続行する。
pub(crate) fn run_post_processing(config: &GenerateConfig, output_path: &Path) {
    let commands = determine_post_commands(config);
    let retry_config = RetryConfig::default();

    for (cmd, args) in &commands {
        let args_refs: Vec<&str> = args.clone();
        match run_with_retry(cmd, &args_refs, output_path, &retry_config) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("{e}");
                eprintln!(
                    "手動で実行してください: cd {} && {} {}",
                    output_path.display(),
                    cmd,
                    args.join(" ")
                );
            }
        }
    }
}

/// 後処理コマンドのリストを決定する。
///
/// テンプレートエンジン仕様.md の「生成後の後処理」セクションに準拠:
///   1. 言語固有の依存解決
///   2. コード生成 (buf generate / oapi-codegen / cargo xtask codegen)
///   3. SQL マイグレーション初期化 (DB ありの場合)
pub(crate) fn determine_post_commands(
    config: &GenerateConfig,
) -> Vec<(&'static str, Vec<&'static str>)> {
    let mut commands: Vec<(&str, Vec<&str>)> = Vec::new();

    match config.kind {
        Kind::Server => {
            // 1. 言語固有の依存解決
            match &config.lang_fw {
                LangFw::Language(Language::Go) => {
                    commands.push(("go", vec!["mod", "tidy"]));
                }
                LangFw::Language(Language::Rust) => {
                    commands.push(("cargo", vec!["check"]));
                }
                _ => {}
            }
            // 2. コード生成
            // gRPC の場合は buf generate
            if config.detail.api_styles.contains(&ApiStyle::Grpc) {
                commands.push(("buf", vec!["generate"]));
            }
            // REST (OpenAPI) の場合はコード生成
            if config.detail.api_styles.contains(&ApiStyle::Rest) {
                match &config.lang_fw {
                    LangFw::Language(Language::Go) => {
                        commands.push((
                            "oapi-codegen",
                            vec![
                                "-generate",
                                "types,server",
                                "-package",
                                "handler",
                                "-o",
                                "internal/handler/openapi.gen.go",
                                "api/openapi/openapi.yaml",
                            ],
                        ));
                    }
                    LangFw::Language(Language::Rust) => {
                        commands.push(("cargo", vec!["xtask", "codegen"]));
                    }
                    _ => {}
                }
            }
            // GraphQL の場合は gqlgen generate
            if config.detail.api_styles.contains(&ApiStyle::GraphQL) {
                if let LangFw::Language(Language::Go) = &config.lang_fw {
                    commands.push(("go", vec!["run", "github.com/99designs/gqlgen", "generate"]));
                }
            }
            // 3. DB ありの場合は SQL マイグレーション初期化
            if config.detail.db.is_some() {
                commands.push(("sqlx", vec!["database", "create"]));
            }
        }
        Kind::Client => match &config.lang_fw {
            LangFw::Framework(Framework::React) => {
                commands.push(("npm", vec!["install"]));
            }
            LangFw::Framework(Framework::Flutter) => {
                commands.push(("flutter", vec!["pub", "get"]));
            }
            _ => {}
        },
        Kind::Library => match &config.lang_fw {
            LangFw::Language(Language::Go) => {
                commands.push(("go", vec!["mod", "tidy"]));
            }
            LangFw::Language(Language::Rust) => {
                commands.push(("cargo", vec!["check"]));
            }
            LangFw::Language(Language::TypeScript) => {
                commands.push(("npm", vec!["install"]));
            }
            LangFw::Language(Language::Dart) => {
                commands.push(("flutter", vec!["pub", "get"]));
            }
            _ => {}
        },
        Kind::Database => {
            // データベースには後処理コマンドなし
        }
    }

    commands
}
