//! `k1s0 playground` コマンド
//!
//! サンプルコード付きの playground 環境を Docker または ローカルプロセスで起動する。

use std::io::Write;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use chrono::Local;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

use crate::error::{CliError, Result};
use crate::output::output;

/// `k1s0 playground` の引数
#[derive(Args, Debug)]
#[command(after_long_help = r#"例:
  k1s0 playground start --type backend-rust --name my-app
  k1s0 playground start --type backend-rust --mode local
  k1s0 playground stop --name playground-20260131-120000
  k1s0 playground status
  k1s0 playground list

サンプルコード付きの playground 環境を Docker またはローカルプロセスで起動します。
"#)]
pub struct PlaygroundArgs {
    /// playground サブコマンド
    #[command(subcommand)]
    pub action: PlaygroundAction,
}

/// Playground サブコマンド
#[derive(Subcommand, Debug)]
pub enum PlaygroundAction {
    /// playground 環境を起動する
    Start(StartArgs),
    /// playground 環境を停止する
    Stop(StopArgs),
    /// playground 環境の状態を表示する
    Status(StatusArgs),
    /// 利用可能なテンプレートを一覧表示する
    List(ListArgs),
}

/// `k1s0 playground start` の引数
#[derive(Args, Debug)]
pub struct StartArgs {
    /// テンプレートタイプ（backend-rust, backend-go, backend-csharp, backend-python, frontend-react, frontend-flutter）
    #[arg(long = "type")]
    pub template_type: Option<String>,

    /// playground 名（省略時は自動生成）
    #[arg(long)]
    pub name: Option<String>,

    /// 起動モード（docker, local）省略時は自動検出
    #[arg(long)]
    pub mode: Option<String>,

    /// gRPC エンドポイントを有効にする
    #[arg(long)]
    pub with_grpc: bool,

    /// REST エンドポイントを有効にする（デフォルト: true）
    #[arg(long, default_value_t = true)]
    pub with_rest: bool,

    /// データベース（PostgreSQL）を有効にする
    #[arg(long)]
    pub with_db: bool,

    /// キャッシュ（Redis）を有効にする
    #[arg(long)]
    pub with_cache: bool,

    /// ポートオフセット（0-999）
    #[arg(long, default_value_t = 0, value_parser = clap::value_parser!(u16).range(0..1000))]
    pub port_offset: u16,

    /// 確認をスキップする
    #[arg(short, long)]
    pub yes: bool,
}

/// `k1s0 playground stop` の引数
#[derive(Args, Debug)]
pub struct StopArgs {
    /// 停止する playground 名
    #[arg(long)]
    pub name: Option<String>,

    /// ボリュームも削除する
    #[arg(long)]
    pub volumes: bool,

    /// 確認をスキップする
    #[arg(short, long)]
    pub yes: bool,
}

/// `k1s0 playground status` の引数
#[derive(Args, Debug)]
pub struct StatusArgs {
    /// JSON 形式で出力する
    #[arg(long)]
    pub json: bool,
}

/// `k1s0 playground list` の引数
#[derive(Args, Debug)]
pub struct ListArgs {}

/// Playground の起動モード
#[derive(Debug, Clone, PartialEq, Eq)]
enum PlaygroundMode {
    /// Docker Compose で起動
    Docker,
    /// ローカルプロセスで起動
    Local,
}

impl std::fmt::Display for PlaygroundMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Docker => write!(f, "docker"),
            Self::Local => write!(f, "local"),
        }
    }
}

/// Playground のメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlaygroundMetadata {
    /// playground 名
    name: String,
    /// テンプレートタイプ
    template_type: String,
    /// 起動モード
    mode: String,
    /// オプション
    options: PlaygroundOptions,
    /// ポートオフセット
    port_offset: u16,
    /// ポート設定
    ports: PortConfig,
    /// プロセス ID（ローカルモード時のみ）
    pid: Option<u32>,
    /// 作成日時
    created_at: String,
    /// ディレクトリパス
    dir: String,
}

/// Playground のオプション
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlaygroundOptions {
    /// gRPC を有効にする
    with_grpc: bool,
    /// REST を有効にする
    with_rest: bool,
    /// データベースを有効にする
    with_db: bool,
    /// キャッシュを有効にする
    with_cache: bool,
}

/// ポート設定
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortConfig {
    /// REST ポート
    rest_port: u16,
    /// gRPC ポート
    grpc_port: u16,
    /// データベースポート
    db_port: u16,
    /// Redis ポート
    redis_port: u16,
}

/// `k1s0 playground` を実行する
pub fn execute(args: PlaygroundArgs) -> Result<()> {
    match args.action {
        PlaygroundAction::Start(start_args) => execute_start(start_args),
        PlaygroundAction::Stop(stop_args) => execute_stop(stop_args),
        PlaygroundAction::Status(status_args) => execute_status(status_args),
        PlaygroundAction::List(list_args) => execute_list(list_args),
    }
}

/// playground 環境を起動する
fn execute_start(args: StartArgs) -> Result<()> {
    let out = output();

    // テンプレートタイプの検証
    let template_type = args.template_type.as_deref().ok_or_else(|| {
        CliError::usage("--type オプションは必須です")
            .with_hint("利用可能なタイプ: backend-rust, backend-go, backend-csharp, backend-python, backend-kotlin, frontend-react, frontend-flutter, frontend-android")
            .with_recovery("k1s0 playground list", "利用可能なテンプレートを一覧表示")
    })?;

    validate_template_type(template_type)?;

    // モード検出
    let mode = detect_mode(&args.mode);
    out.info(&format!("起動モード: {mode}"));

    // ツールチェイン確認
    match mode {
        PlaygroundMode::Docker => {
            if !check_docker_available() {
                return Err(CliError::validation("Docker がインストールされていません")
                    .with_hint("Docker をインストールしてください: https://docs.docker.com/get-docker/"));
            }
            if !check_docker_compose_available() {
                return Err(CliError::validation("Docker Compose v2 がインストールされていません")
                    .with_hint("Docker Desktop をインストールするか、docker compose プラグインを追加してください"));
            }
        }
        PlaygroundMode::Local => {
            check_toolchain(template_type)?;
        }
    }

    // テンプレートディレクトリの解決
    let template_base = resolve_template_dir()?;
    out.verbose(&format!("テンプレートベース: {}", template_base.display()));

    // playground ベースディレクトリの解決
    let playground_base = resolve_playground_base_dir()?;
    std::fs::create_dir_all(&playground_base)
        .map_err(|e| CliError::io(format!("playground ディレクトリの作成に失敗: {e}")))?;

    // 名前の決定
    let name = args.name.unwrap_or_else(generate_playground_name);
    let playground_dir = playground_base.join(&name);

    if playground_dir.exists() {
        return Err(CliError::conflict(format!(
            "playground '{name}' は既に存在します"
        ))
        .with_hint("別の名前を指定するか、先に停止してください")
        .with_recovery(
            format!("k1s0 playground stop --name {name}"),
            "既存の playground を停止",
        ));
    }

    // ポートの解決と確認
    let ports = resolve_ports(args.port_offset);
    check_ports(&ports, args.with_grpc, args.with_db, args.with_cache)?;

    // feature_name の各種形式を準備
    let feature_name = name.clone();
    let service_name = name.clone();
    let feature_name_snake = feature_name.replace('-', "_");
    let feature_name_pascal = feature_name
        .split('-')
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    upper + chars.as_str()
                }
            }
        })
        .collect::<String>();

    // 確認
    out.newline();
    out.header("Playground 設定:");
    out.list_item("名前", &name);
    out.list_item("テンプレート", template_type);
    out.list_item("モード", &mode.to_string());
    out.list_item("REST ポート", &ports.rest_port.to_string());
    if args.with_grpc {
        out.list_item("gRPC ポート", &ports.grpc_port.to_string());
    }
    if args.with_db {
        out.list_item("DB ポート", &ports.db_port.to_string());
    }
    if args.with_cache {
        out.list_item("Redis ポート", &ports.redis_port.to_string());
    }
    out.list_item("ディレクトリ", &playground_dir.display().to_string());
    out.newline();

    if !args.yes && !out.confirm_proceed("playground を作成しますか?") {
        return Err(CliError::cancelled("操作がキャンセルされました"));
    }

    // Tera コンテキストの構築
    let mut context = k1s0_generator::Context::new();
    context.insert("feature_name", &feature_name);
    context.insert("service_name", &service_name);
    context.insert("feature_name_snake", &feature_name_snake);
    context.insert("feature_name_pascal", &feature_name_pascal);
    context.insert("with_grpc", &args.with_grpc);
    context.insert("with_rest", &args.with_rest);
    context.insert("with_db", &args.with_db);
    context.insert("with_cache", &args.with_cache);
    context.insert("rest_port", &ports.rest_port);
    context.insert("grpc_port", &ports.grpc_port);
    context.insert("db_port", &ports.db_port);
    context.insert("redis_port", &ports.redis_port);
    context.insert("is_playground", &true);
    context.insert("mode", &mode.to_string());

    // playground ディレクトリ生成
    std::fs::create_dir_all(&playground_dir)
        .map_err(|e| CliError::io(format!("playground ディレクトリの作成に失敗: {e}")))?;

    // テンプレートのレンダリング（マルチパス）
    render_playground(
        &playground_dir,
        template_type,
        &template_base,
        &context,
        &mode,
    )?;

    // メタデータの保存
    let metadata = PlaygroundMetadata {
        name: name.clone(),
        template_type: template_type.to_string(),
        mode: mode.to_string(),
        options: PlaygroundOptions {
            with_grpc: args.with_grpc,
            with_rest: args.with_rest,
            with_db: args.with_db,
            with_cache: args.with_cache,
        },
        port_offset: args.port_offset,
        ports: ports.clone(),
        pid: None,
        created_at: Local::now().format("%Y-%m-%dT%H:%M:%S%z").to_string(),
        dir: playground_dir.display().to_string(),
    };
    save_metadata(&playground_dir, &metadata)?;

    // 起動
    match mode {
        PlaygroundMode::Docker => {
            out.info("Docker Compose で起動しています...");
            start_docker(&playground_dir)?;
        }
        PlaygroundMode::Local => {
            // データディレクトリの作成（DB 利用時）
            if args.with_db {
                let data_dir = playground_dir.join("data");
                std::fs::create_dir_all(&data_dir).map_err(|e| {
                    CliError::io(format!("データディレクトリの作成に失敗: {e}"))
                })?;
            }

            // ビルド
            out.info("プロジェクトをビルドしています...");
            run_build_command(&playground_dir, template_type)?;

            // プロセス起動
            out.info("ローカルプロセスを起動しています...");
            let pid = start_local_process(&playground_dir, template_type, &ports)?;

            // メタデータに PID を保存
            let metadata_with_pid = PlaygroundMetadata {
                pid: Some(pid),
                ..metadata
            };
            save_metadata(&playground_dir, &metadata_with_pid)?;
        }
    }

    // ヘルスチェック待機
    out.info("サービスの起動を待機しています...");
    match wait_for_health(ports.rest_port, 30) {
        Ok(()) => {
            out.success("サービスが起動しました");
        }
        Err(_) => {
            out.warning("ヘルスチェックがタイムアウトしました。サービスがまだ起動中の可能性があります。");
        }
    }

    // 成功メッセージ
    out.newline();
    out.header("Playground が起動しました:");
    if args.with_rest {
        out.list_item(
            "REST",
            &format!("http://localhost:{}", ports.rest_port),
        );
    }
    if args.with_grpc {
        out.list_item(
            "gRPC",
            &format!("http://localhost:{}", ports.grpc_port),
        );
    }
    if args.with_db {
        out.list_item(
            "PostgreSQL",
            &format!("localhost:{}", ports.db_port),
        );
    }
    if args.with_cache {
        out.list_item(
            "Redis",
            &format!("localhost:{}", ports.redis_port),
        );
    }
    out.newline();
    out.hint(&format!("停止: k1s0 playground stop --name {name}"));
    out.hint("状態: k1s0 playground status");

    Ok(())
}

/// playground 環境を停止する
fn execute_stop(args: StopArgs) -> Result<()> {
    let out = output();

    let playgrounds = list_playgrounds();
    if playgrounds.is_empty() {
        out.info("実行中の playground はありません");
        return Ok(());
    }

    // 名前が指定されていない場合、全停止確認
    let targets: Vec<&PlaygroundMetadata> = if let Some(ref name) = args.name {
        let found: Vec<&PlaygroundMetadata> =
            playgrounds.iter().filter(|p| p.name == *name).collect();
        if found.is_empty() {
            return Err(CliError::validation(format!(
                "playground '{name}' が見つかりません"
            ))
            .with_recovery("k1s0 playground status", "実行中の playground を確認"));
        }
        found
    } else {
        playgrounds.iter().collect()
    };

    if !args.yes {
        let names: Vec<&str> = targets.iter().map(|p| p.name.as_str()).collect();
        let msg = format!(
            "以下の playground を停止しますか?\n  {}",
            names.join("\n  ")
        );
        if !out.confirm_proceed(&msg) {
            return Err(CliError::cancelled("操作がキャンセルされました"));
        }
    }

    for target in &targets {
        let dir = PathBuf::from(&target.dir);
        out.info(&format!("'{}' を停止しています...", target.name));

        if target.mode == "docker" {
            stop_docker(&dir, args.volumes)?;
        } else if let Some(pid) = target.pid {
            stop_local_process(pid)?;
        }

        // メタデータファイルを削除
        let metadata_path = dir.join(".playground.json");
        let _ = std::fs::remove_file(metadata_path);

        // ボリュームオプションが指定されている場合はディレクトリも削除
        if args.volumes {
            let _ = std::fs::remove_dir_all(&dir);
            out.success(&format!("'{}' を削除しました", target.name));
        } else {
            out.success(&format!("'{}' を停止しました", target.name));
        }
    }

    Ok(())
}

/// playground 環境の状態を表示する
fn execute_status(args: StatusArgs) -> Result<()> {
    let out = output();
    let playgrounds = list_playgrounds();

    if args.json {
        let json = serde_json::to_string_pretty(&playgrounds)
            .map_err(|e| CliError::io(format!("JSON シリアライズに失敗: {e}")))?;
        println!("{json}");
        return Ok(());
    }

    if playgrounds.is_empty() {
        out.info("実行中の playground はありません");
        return Ok(());
    }

    out.header("Playground 一覧:");
    out.newline();

    for pg in &playgrounds {
        let status = if pg.mode == "docker" {
            check_docker_status(&PathBuf::from(&pg.dir))
        } else if let Some(pid) = pg.pid {
            if is_process_alive(pid) {
                "running".to_string()
            } else {
                "stopped".to_string()
            }
        } else {
            "unknown".to_string()
        };

        let dir = PathBuf::from(&pg.dir);
        let disk = calculate_disk_usage(&dir);

        out.list_item("名前", &pg.name);
        out.list_item("  テンプレート", &pg.template_type);
        out.list_item("  モード", &pg.mode);
        out.list_item("  状態", &status);
        out.list_item("  REST", &format!("http://localhost:{}", pg.ports.rest_port));
        if pg.options.with_grpc {
            out.list_item("  gRPC", &format!("http://localhost:{}", pg.ports.grpc_port));
        }
        out.list_item("  ディスク使用量", &format_bytes(disk));
        out.list_item("  作成日時", &pg.created_at);
        out.newline();
    }

    Ok(())
}

/// 利用可能なテンプレートを一覧表示する
fn execute_list(_args: ListArgs) -> Result<()> {
    let out = output();

    out.header("利用可能なテンプレート:");
    out.newline();
    out.list_item("backend-rust", "Rust バックエンドサービス (axum + tokio)");
    out.list_item("backend-go", "Go バックエンドサービス");
    out.list_item("backend-csharp", "C# バックエンドサービス (ASP.NET Core)");
    out.list_item("backend-python", "Python バックエンドサービス (FastAPI)");
    out.list_item("frontend-react", "React フロントエンド (Material-UI)");
    out.list_item("frontend-flutter", "Flutter フロントエンド (Material 3)");
    out.list_item("backend-kotlin", "Kotlin バックエンドサービス (Ktor)");
    out.list_item("frontend-android", "Android フロントエンド (Jetpack Compose)");
    out.newline();
    out.hint("使い方: k1s0 playground start --type backend-rust --name my-app");

    Ok(())
}

/// テンプレートタイプを検証する
fn validate_template_type(template_type: &str) -> Result<()> {
    let valid_types = [
        "backend-rust",
        "backend-go",
        "backend-csharp",
        "backend-python",
        "frontend-react",
        "frontend-flutter",
        "backend-kotlin",
        "frontend-android",
    ];

    if !valid_types.contains(&template_type) {
        return Err(CliError::validation(format!(
            "無効なテンプレートタイプ: {template_type}"
        ))
        .with_hint(format!(
            "利用可能なタイプ: {}",
            valid_types.join(", ")
        ))
        .with_recovery("k1s0 playground list", "利用可能なテンプレートを一覧表示"));
    }

    Ok(())
}

/// 起動モードを検出する
///
/// 明示的に指定されている場合はそれを使用し、
/// そうでない場合は Docker が利用可能かどうかで自動判定する。
fn detect_mode(explicit: &Option<String>) -> PlaygroundMode {
    if let Some(mode_str) = explicit {
        match mode_str.to_lowercase().as_str() {
            "docker" => return PlaygroundMode::Docker,
            "local" => return PlaygroundMode::Local,
            _ => {}
        }
    }

    if check_docker_available() && check_docker_compose_available() {
        PlaygroundMode::Docker
    } else {
        PlaygroundMode::Local
    }
}

/// Docker がインストールされているか確認
fn check_docker_available() -> bool {
    Command::new("docker")
        .arg("--version")
        .output()
        .is_ok_and(|out| out.status.success())
}

/// Docker Compose が利用可能か確認
fn check_docker_compose_available() -> bool {
    Command::new("docker")
        .args(["compose", "version"])
        .output()
        .is_ok_and(|out| out.status.success())
}

/// ローカルモード用のツールチェインを確認する
fn check_toolchain(template_type: &str) -> Result<()> {
    let (cmd, args, name, install_hint) = match template_type {
        "backend-rust" => ("cargo", vec!["--version"], "Rust (cargo)", "https://rustup.rs/"),
        "backend-go" => ("go", vec!["version"], "Go", "https://go.dev/dl/"),
        "backend-csharp" => (
            "dotnet",
            vec!["--version"],
            ".NET SDK",
            "https://dot.net/",
        ),
        "backend-python" => (
            "python",
            vec!["--version"],
            "Python",
            "https://www.python.org/downloads/",
        ),
        "frontend-react" => (
            "pnpm",
            vec!["--version"],
            "pnpm",
            "https://pnpm.io/installation",
        ),
        "frontend-flutter" => (
            "flutter",
            vec!["--version"],
            "Flutter",
            "https://flutter.dev/docs/get-started/install",
        ),
        "backend-kotlin" => (
            "kotlin",
            vec!["-version"],
            "Kotlin",
            "https://kotlinlang.org/docs/command-line.html",
        ),
        "frontend-android" => (
            "kotlin",
            vec!["-version"],
            "Kotlin (Android)",
            "https://developer.android.com/studio",
        ),
        _ => {
            return Err(CliError::validation(format!(
                "未対応のテンプレートタイプ: {template_type}"
            )));
        }
    };

    let check_args: Vec<&str> = args;
    match Command::new(cmd).args(&check_args).output() {
        Ok(out) if out.status.success() => Ok(()),
        _ => Err(CliError::validation(format!(
            "{name} がインストールされていません"
        ))
        .with_hint(format!("インストールしてください: {install_hint}"))),
    }
}

/// テンプレートディレクトリを解決する
///
/// カレントディレクトリから親を辿り CLI/templates/ を探す。
fn resolve_template_dir() -> Result<PathBuf> {
    // まず実行ファイルの場所から辿る
    if let Ok(exe_path) = std::env::current_exe() {
        let mut dir = exe_path.parent().map(Path::to_path_buf);
        for _ in 0..10 {
            if let Some(ref d) = dir {
                let candidate = d.join("CLI").join("templates");
                if candidate.is_dir() {
                    return Ok(candidate);
                }
                dir = d.parent().map(Path::to_path_buf);
            } else {
                break;
            }
        }
    }

    // カレントディレクトリから辿る
    if let Ok(cwd) = std::env::current_dir() {
        let mut dir = Some(cwd);
        for _ in 0..10 {
            if let Some(ref d) = dir {
                let candidate = d.join("CLI").join("templates");
                if candidate.is_dir() {
                    return Ok(candidate);
                }
                dir = d.parent().map(Path::to_path_buf);
            } else {
                break;
            }
        }
    }

    Err(CliError::config(
        "テンプレートディレクトリが見つかりません",
    )
    .with_hint("k1s0 リポジトリのルート、または CLI/templates/ が含まれるディレクトリで実行してください"))
}

/// playground ベースディレクトリを解決する
///
/// プロジェクトルートに `.k1s0/playground/` があればそこを使い、
/// なければホームディレクトリ配下の `.k1s0/playground/` を使用する。
fn resolve_playground_base_dir() -> Result<PathBuf> {
    // プロジェクトルートの .k1s0/playground/
    if let Ok(cwd) = std::env::current_dir() {
        let project_dir = cwd.join(".k1s0").join("playground");
        let k1s0_dir = cwd.join(".k1s0");
        if k1s0_dir.is_dir() {
            return Ok(project_dir);
        }
    }

    // ホームディレクトリの .k1s0/playground/
    let home = home_dir().ok_or_else(|| {
        CliError::io("ホームディレクトリが取得できません")
    })?;
    Ok(home.join(".k1s0").join("playground"))
}

/// ホームディレクトリを取得する（クロスプラットフォーム対応）
fn home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        // Windows: USERPROFILE を使用
        std::env::var_os("USERPROFILE").map(PathBuf::from)
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}

/// playground 名を自動生成する
fn generate_playground_name() -> String {
    let now = Local::now();
    now.format("playground-%Y%m%d-%H%M%S").to_string()
}

/// ポート設定を解決する
fn resolve_ports(offset: u16) -> PortConfig {
    PortConfig {
        rest_port: 8080 + offset,
        grpc_port: 50051 + offset,
        db_port: 5432 + offset,
        redis_port: 6379 + offset,
    }
}

/// ポートが利用可能か確認する
fn check_ports(
    ports: &PortConfig,
    with_grpc: bool,
    with_db: bool,
    with_cache: bool,
) -> Result<()> {
    check_single_port(ports.rest_port, "REST")?;

    if with_grpc {
        check_single_port(ports.grpc_port, "gRPC")?;
    }
    if with_db {
        check_single_port(ports.db_port, "PostgreSQL")?;
    }
    if with_cache {
        check_single_port(ports.redis_port, "Redis")?;
    }

    Ok(())
}

/// 単一ポートが利用可能か確認する
fn check_single_port(port: u16, service_name: &str) -> Result<()> {
    match TcpListener::bind(format!("127.0.0.1:{port}")) {
        Ok(_listener) => Ok(()),
        Err(_) => Err(CliError::validation(format!(
            "ポート {port} ({service_name}) は既に使用されています"
        ))
        .with_hint("--port-offset オプションで別のポートを使用してください")
        .with_recovery(
            format!("k1s0 playground start --port-offset {}", port - 8080 + 100),
            "別のポートオフセットで再試行",
        )),
    }
}

/// テンプレートをマルチパスでレンダリングする
///
/// Pass 1: ベーステンプレート（CLI/templates/{type}/feature/）
/// Pass 2: playground オーバーレイ（CLI/templates/playground/{type}/）
/// Pass 3: ローカルオーバーレイ（CLI/templates/playground/{type}-local/）※ローカルモードのみ
/// Pass 4: 共通オーバーレイ（CLI/templates/playground/common/）
fn render_playground(
    output_dir: &Path,
    template_type: &str,
    template_base: &Path,
    context: &k1s0_generator::Context,
    mode: &PlaygroundMode,
) -> Result<()> {
    let out = output();

    // Pass 1: ベーステンプレート
    let base_template_dir = template_base.join(template_type).join("feature");
    if base_template_dir.is_dir() {
        out.verbose(&format!(
            "Pass 1: ベーステンプレート ({})",
            base_template_dir.display()
        ));
        render_single_pass(&base_template_dir, output_dir, context)?;
    }

    // Pass 2: playground オーバーレイ
    let playground_overlay = template_base.join("playground").join(template_type);
    if playground_overlay.is_dir() {
        out.verbose(&format!(
            "Pass 2: playground オーバーレイ ({})",
            playground_overlay.display()
        ));
        render_single_pass(&playground_overlay, output_dir, context)?;
    }

    // Pass 3: ローカルオーバーレイ（ローカルモードのみ）
    if *mode == PlaygroundMode::Local {
        let local_overlay = template_base
            .join("playground")
            .join(format!("{template_type}-local"));
        if local_overlay.is_dir() {
            out.verbose(&format!(
                "Pass 3: ローカルオーバーレイ ({})",
                local_overlay.display()
            ));
            render_single_pass(&local_overlay, output_dir, context)?;
        }
    }

    // Pass 4: 共通オーバーレイ
    let common_overlay = template_base.join("playground").join("common");
    if common_overlay.is_dir() {
        out.verbose(&format!(
            "Pass 4: 共通オーバーレイ ({})",
            common_overlay.display()
        ));
        render_single_pass(&common_overlay, output_dir, context)?;
    }

    Ok(())
}

/// 単一パスのレンダリングを実行する
fn render_single_pass(
    template_dir: &Path,
    output_dir: &Path,
    context: &k1s0_generator::Context,
) -> Result<()> {
    let renderer = k1s0_generator::template::TemplateRenderer::new(template_dir)
        .map_err(|e| CliError::io(format!("テンプレートの読み込みに失敗: {e}")))?;

    renderer
        .render_directory(output_dir, context)
        .map_err(|e| CliError::io(format!("テンプレートのレンダリングに失敗: {e}")))?;

    Ok(())
}

/// メタデータを保存する
fn save_metadata(dir: &Path, metadata: &PlaygroundMetadata) -> Result<()> {
    let path = dir.join(".playground.json");
    let json = serde_json::to_string_pretty(metadata)
        .map_err(|e| CliError::io(format!("メタデータのシリアライズに失敗: {e}")))?;

    let mut file = std::fs::File::create(&path)
        .map_err(|e| CliError::io(format!("メタデータファイルの作成に失敗: {e}")))?;
    file.write_all(json.as_bytes())
        .map_err(|e| CliError::io(format!("メタデータの書き込みに失敗: {e}")))?;

    Ok(())
}

/// メタデータを読み込む
fn load_metadata(dir: &Path) -> Result<PlaygroundMetadata> {
    let path = dir.join(".playground.json");
    let content = std::fs::read_to_string(&path)
        .map_err(|e| CliError::io(format!("メタデータの読み取りに失敗: {e}")))?;
    let metadata: PlaygroundMetadata = serde_json::from_str(&content)
        .map_err(|e| CliError::io(format!("メタデータのパースに失敗: {e}")))?;
    Ok(metadata)
}

/// 全 playground を一覧取得する
fn list_playgrounds() -> Vec<PlaygroundMetadata> {
    let mut results = Vec::new();

    // プロジェクトルートの .k1s0/playground/
    if let Ok(cwd) = std::env::current_dir() {
        let project_dir = cwd.join(".k1s0").join("playground");
        scan_playground_dir(&project_dir, &mut results);
    }

    // ホームディレクトリの .k1s0/playground/
    if let Some(home) = home_dir() {
        let home_dir = home.join(".k1s0").join("playground");
        scan_playground_dir(&home_dir, &mut results);
    }

    // 重複排除（名前ベース）
    results.sort_by(|a, b| a.name.cmp(&b.name));
    results.dedup_by(|a, b| a.name == b.name);

    results
}

/// ディレクトリをスキャンして playground を検出する
fn scan_playground_dir(base: &Path, results: &mut Vec<PlaygroundMetadata>) {
    if !base.is_dir() {
        return;
    }

    if let Ok(entries) = std::fs::read_dir(base) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(meta) = load_metadata(&path) {
                    results.push(meta);
                }
            }
        }
    }
}

/// Docker Compose で起動する
fn start_docker(dir: &Path) -> Result<()> {
    let status = Command::new("docker")
        .args(["compose", "up", "-d", "--build"])
        .current_dir(dir)
        .status()
        .map_err(|e| CliError::io(format!("docker compose up の実行に失敗: {e}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(CliError::validation("docker compose up が失敗しました")
            .with_hint("docker compose logs で詳細を確認してください"))
    }
}

/// ローカルプロセスを起動する
fn start_local_process(dir: &Path, template_type: &str, ports: &PortConfig) -> Result<u32> {
    // ログディレクトリの作成
    let logs_dir = dir.join("logs");
    std::fs::create_dir_all(&logs_dir)
        .map_err(|e| CliError::io(format!("ログディレクトリの作成に失敗: {e}")))?;

    let log_file = std::fs::File::create(logs_dir.join("server.log"))
        .map_err(|e| CliError::io(format!("ログファイルの作成に失敗: {e}")))?;
    let log_stderr = log_file
        .try_clone()
        .map_err(|e| CliError::io(format!("ログファイルの複製に失敗: {e}")))?;

    let child = match template_type {
        "backend-rust" => Command::new("cargo")
            .args(["run", "--", "--env", "dev", "--config", "config/"])
            .current_dir(dir)
            .stdout(log_file)
            .stderr(log_stderr)
            .spawn(),
        "backend-go" => Command::new("go")
            .args(["run", "./cmd/", "--env", "dev", "--config", "config/"])
            .current_dir(dir)
            .stdout(log_file)
            .stderr(log_stderr)
            .spawn(),
        "backend-python" => {
            let snake_name = dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .replace('-', "_");
            Command::new("python")
                .args([
                    "-m",
                    "uvicorn",
                    &format!("{snake_name}.presentation.main:app"),
                    "--host",
                    "0.0.0.0",
                    "--port",
                    &ports.rest_port.to_string(),
                ])
                .current_dir(dir)
                .stdout(log_file)
                .stderr(log_stderr)
                .spawn()
        }
        "backend-csharp" => Command::new("dotnet")
            .args(["run", "--project", "src/*/"])
            .current_dir(dir)
            .stdout(log_file)
            .stderr(log_stderr)
            .spawn(),
        "frontend-react" => Command::new("npx")
            .args([
                "serve",
                "-s",
                "build",
                "-l",
                &ports.rest_port.to_string(),
            ])
            .current_dir(dir)
            .stdout(log_file)
            .stderr(log_stderr)
            .spawn(),
        "frontend-flutter" => Command::new("flutter")
            .args([
                "run",
                "-d",
                "chrome",
                "--web-port",
                &ports.rest_port.to_string(),
            ])
            .current_dir(dir)
            .stdout(log_file)
            .stderr(log_stderr)
            .spawn(),
        "backend-kotlin" => Command::new("./gradlew")
            .args(["run"])
            .current_dir(dir)
            .stdout(log_file)
            .stderr(log_stderr)
            .spawn(),
        "frontend-android" => Command::new("./gradlew")
            .args(["installDebug"])
            .current_dir(dir)
            .stdout(log_file)
            .stderr(log_stderr)
            .spawn(),
        _ => {
            return Err(CliError::validation(format!(
                "ローカル起動に未対応のテンプレートタイプ: {template_type}"
            )));
        }
    };

    let child = child.map_err(|e| {
        CliError::io(format!("プロセスの起動に失敗: {e}"))
            .with_hint("必要なツールチェインがインストールされているか確認してください")
    })?;

    Ok(child.id())
}

/// ビルドコマンドを実行する（ローカルモード用）
fn run_build_command(dir: &Path, template_type: &str) -> Result<()> {
    let status = match template_type {
        "backend-rust" => Command::new("cargo")
            .args(["build"])
            .current_dir(dir)
            .status(),
        "backend-go" => Command::new("go")
            .args(["build", "./..."])
            .current_dir(dir)
            .status(),
        "backend-python" => {
            // Python はビルド不要（依存インストールのみ）
            Command::new("python")
                .args(["-m", "pip", "install", "-e", "."])
                .current_dir(dir)
                .status()
        }
        "backend-csharp" => Command::new("dotnet")
            .args(["build"])
            .current_dir(dir)
            .status(),
        "frontend-react" => Command::new("pnpm")
            .args(["install"])
            .current_dir(dir)
            .status()
            .and_then(|s| {
                if s.success() {
                    Command::new("pnpm")
                        .args(["build"])
                        .current_dir(dir)
                        .status()
                } else {
                    Ok(s)
                }
            }),
        "frontend-flutter" => Command::new("flutter")
            .args(["build", "web"])
            .current_dir(dir)
            .status(),
        "backend-kotlin" | "frontend-android" => Command::new("./gradlew")
            .args(["build"])
            .current_dir(dir)
            .status(),
        _ => {
            return Err(CliError::validation(format!(
                "ビルドに未対応のテンプレートタイプ: {template_type}"
            )));
        }
    };

    let status =
        status.map_err(|e| CliError::io(format!("ビルドコマンドの実行に失敗: {e}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(CliError::validation("ビルドが失敗しました")
            .with_hint("ログを確認してエラーを修正してください"))
    }
}

/// Docker Compose を停止する
fn stop_docker(dir: &Path, volumes: bool) -> Result<()> {
    let mut cmd = Command::new("docker");
    cmd.args(["compose", "down"]);
    if volumes {
        cmd.arg("-v");
    }
    cmd.current_dir(dir);

    let status = cmd
        .status()
        .map_err(|e| CliError::io(format!("docker compose down の実行に失敗: {e}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(CliError::validation("docker compose down が失敗しました"))
    }
}

/// ローカルプロセスを停止する（Unix）
#[cfg(unix)]
fn stop_local_process(pid: u32) -> Result<()> {
    let _ = Command::new("kill").arg(pid.to_string()).status();
    std::thread::sleep(Duration::from_secs(3));
    let _ = Command::new("kill")
        .args(["-9", &pid.to_string()])
        .status();
    Ok(())
}

/// ローカルプロセスを停止する（Windows）
#[cfg(windows)]
fn stop_local_process(pid: u32) -> Result<()> {
    let _ = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/F"])
        .status();
    Ok(())
}

/// ヘルスエンドポイントをポーリングして起動を待機する
fn wait_for_health(port: u16, timeout_secs: u64) -> Result<()> {
    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    loop {
        if start.elapsed() > timeout {
            return Err(CliError::validation(
                "ヘルスチェックがタイムアウトしました",
            ));
        }

        // TCP 接続で確認
        if std::net::TcpStream::connect_timeout(
            &format!("127.0.0.1:{port}").parse().unwrap(),
            Duration::from_secs(1),
        )
        .is_ok()
        {
            return Ok(());
        }

        std::thread::sleep(Duration::from_millis(500));
    }
}

/// プロセスが生存しているか確認する（Unix）
#[cfg(unix)]
fn is_process_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .is_ok_and(|s| s.success())
}

/// プロセスが生存しているか確認する（Windows）
#[cfg(windows)]
fn is_process_alive(pid: u32) -> bool {
    Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/NH"])
        .output()
        .is_ok_and(|out| {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.contains(&pid.to_string())
        })
}

/// Docker Compose の状態を確認する
fn check_docker_status(dir: &Path) -> String {
    if !dir.is_dir() {
        return "not found".to_string();
    }

    Command::new("docker")
        .args(["compose", "ps", "--format", "{{.State}}"])
        .current_dir(dir)
        .output()
        .map(|out| {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let states: Vec<&str> = stdout.lines().collect();
            if states.is_empty() {
                "stopped".to_string()
            } else if states.iter().all(|s| s.contains("running")) {
                "running".to_string()
            } else {
                "partial".to_string()
            }
        })
        .unwrap_or_else(|_| "unknown".to_string())
}

/// ディレクトリのディスク使用量を再帰的に計算する
fn calculate_disk_usage(dir: &Path) -> u64 {
    if !dir.is_dir() {
        return 0;
    }

    let mut total: u64 = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(meta) = path.metadata() {
                    total += meta.len();
                }
            } else if path.is_dir() {
                total += calculate_disk_usage(&path);
            }
        }
    }
    total
}

/// バイト数を人間が読みやすい形式にフォーマットする
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct TestCli {
        #[command(subcommand)]
        command: PlaygroundAction,
    }

    #[test]
    fn test_detect_mode_explicit_docker() {
        let mode = detect_mode(&Some("docker".to_string()));
        assert_eq!(mode, PlaygroundMode::Docker);
    }

    #[test]
    fn test_detect_mode_explicit_local() {
        let mode = detect_mode(&Some("local".to_string()));
        assert_eq!(mode, PlaygroundMode::Local);
    }

    #[test]
    fn test_generate_playground_name() {
        let name = generate_playground_name();
        assert!(name.starts_with("playground-"));
        // playground-YYYYMMDD-HHMMSS の形式を検証
        assert_eq!(name.len(), "playground-YYYYMMDD-HHMMSS".len());
        // 数字とハイフンのみ（playground- の後）
        let suffix = &name["playground-".len()..];
        assert!(suffix
            .chars()
            .all(|c| c.is_ascii_digit() || c == '-'));
    }

    #[test]
    fn test_resolve_ports() {
        let ports = resolve_ports(0);
        assert_eq!(ports.rest_port, 8080);
        assert_eq!(ports.grpc_port, 50051);
        assert_eq!(ports.db_port, 5432);
        assert_eq!(ports.redis_port, 6379);
    }

    #[test]
    fn test_resolve_ports_offset() {
        let ports = resolve_ports(100);
        assert_eq!(ports.rest_port, 8180);
        assert_eq!(ports.grpc_port, 50151);
        assert_eq!(ports.db_port, 5532);
        assert_eq!(ports.redis_port, 6479);
    }

    #[test]
    fn test_check_ports_available() {
        // ポート 0 にバインドしてシステムに割り当てさせる
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        // リスナーを閉じてからチェック
        drop(listener);

        let ports = PortConfig {
            rest_port: port,
            grpc_port: port + 1,
            db_port: port + 2,
            redis_port: port + 3,
        };
        // ポートが解放された直後なので利用可能であるべき
        let result = check_ports(&ports, true, true, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_metadata_serialization() {
        let metadata = PlaygroundMetadata {
            name: "test-playground".to_string(),
            template_type: "backend-rust".to_string(),
            mode: "docker".to_string(),
            options: PlaygroundOptions {
                with_grpc: true,
                with_rest: true,
                with_db: true,
                with_cache: false,
            },
            port_offset: 0,
            ports: PortConfig {
                rest_port: 8080,
                grpc_port: 50051,
                db_port: 5432,
                redis_port: 6379,
            },
            pid: None,
            created_at: "2026-01-31T12:00:00+0900".to_string(),
            dir: "/tmp/test".to_string(),
        };

        let json = serde_json::to_string_pretty(&metadata).unwrap();
        let deserialized: PlaygroundMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "test-playground");
        assert_eq!(deserialized.template_type, "backend-rust");
        assert_eq!(deserialized.mode, "docker");
        assert!(deserialized.options.with_grpc);
        assert!(deserialized.options.with_rest);
        assert!(deserialized.options.with_db);
        assert!(!deserialized.options.with_cache);
        assert_eq!(deserialized.ports.rest_port, 8080);
        assert_eq!(deserialized.ports.grpc_port, 50051);
        assert!(deserialized.pid.is_none());
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1_048_576), "1.0 MB");
        assert_eq!(format_bytes(1_073_741_824), "1.0 GB");
    }

    #[test]
    fn test_port_config_serialization() {
        let ports = PortConfig {
            rest_port: 8080,
            grpc_port: 50051,
            db_port: 5432,
            redis_port: 6379,
        };

        let json = serde_json::to_string(&ports).unwrap();
        let deserialized: PortConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.rest_port, 8080);
        assert_eq!(deserialized.grpc_port, 50051);
        assert_eq!(deserialized.db_port, 5432);
        assert_eq!(deserialized.redis_port, 6379);
    }

    #[test]
    fn test_validate_template_type_valid() {
        assert!(validate_template_type("backend-rust").is_ok());
        assert!(validate_template_type("backend-go").is_ok());
        assert!(validate_template_type("backend-csharp").is_ok());
        assert!(validate_template_type("backend-python").is_ok());
        assert!(validate_template_type("frontend-react").is_ok());
        assert!(validate_template_type("frontend-flutter").is_ok());
    }

    #[test]
    fn test_validate_template_type_invalid() {
        assert!(validate_template_type("invalid-type").is_err());
        assert!(validate_template_type("").is_err());
    }

    #[test]
    fn test_playground_mode_display() {
        assert_eq!(PlaygroundMode::Docker.to_string(), "docker");
        assert_eq!(PlaygroundMode::Local.to_string(), "local");
    }

    #[test]
    fn test_parse_start() {
        let cli = TestCli::parse_from(["test", "start", "--type", "backend-rust"]);
        assert!(matches!(cli.command, PlaygroundAction::Start(_)));
    }

    #[test]
    fn test_parse_stop() {
        let cli = TestCli::parse_from(["test", "stop", "--name", "my-playground"]);
        match cli.command {
            PlaygroundAction::Stop(args) => {
                assert_eq!(args.name, Some("my-playground".to_string()));
            }
            _ => panic!("Expected Stop"),
        }
    }

    #[test]
    fn test_parse_status_json() {
        let cli = TestCli::parse_from(["test", "status", "--json"]);
        match cli.command {
            PlaygroundAction::Status(args) => {
                assert!(args.json);
            }
            _ => panic!("Expected Status"),
        }
    }

    #[test]
    fn test_parse_list() {
        let cli = TestCli::parse_from(["test", "list"]);
        assert!(matches!(cli.command, PlaygroundAction::List(_)));
    }
}
