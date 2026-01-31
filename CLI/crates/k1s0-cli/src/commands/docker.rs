//! `k1s0 docker` コマンド
//!
//! Docker イメージのビルドや docker-compose の操作を支援する。

use clap::{Args, Subcommand};
use std::path::Path;
use std::process::Command;

use crate::error::{CliError, Result};
use crate::output::output;

/// `k1s0 docker` の引数
#[derive(Args, Debug)]
#[command(after_long_help = r#"例:
  k1s0 docker build
  k1s0 docker build --tag my-app:latest --no-cache
  k1s0 docker compose up -d --build
  k1s0 docker compose down -v
  k1s0 docker compose logs -f
  k1s0 docker status

Docker イメージのビルドや docker-compose の操作を支援します。
"#)]
pub struct DockerArgs {
    /// docker サブコマンド
    #[command(subcommand)]
    pub action: DockerAction,
}

/// Docker サブコマンド
#[derive(Subcommand, Debug)]
pub enum DockerAction {
    /// Docker イメージをビルドする
    Build(BuildArgs),
    /// docker compose を操作する
    Compose(ComposeArgs),
    /// コンテナの状態を表示する
    Status(StatusArgs),
}

/// `k1s0 docker build` の引数
#[derive(Args, Debug)]
pub struct BuildArgs {
    /// イメージタグ（指定しない場合は {feature_name}:{template_version}）
    #[arg(long)]
    pub tag: Option<String>,

    /// キャッシュを使用しない
    #[arg(long)]
    pub no_cache: bool,

    /// HTTP プロキシ
    #[arg(long)]
    pub http_proxy: Option<String>,

    /// HTTPS プロキシ
    #[arg(long)]
    pub https_proxy: Option<String>,
}

/// `k1s0 docker compose` の引数
#[derive(Args, Debug)]
pub struct ComposeArgs {
    /// compose アクション
    #[command(subcommand)]
    pub action: ComposeAction,
}

/// Compose サブコマンド
#[derive(Subcommand, Debug)]
pub enum ComposeAction {
    /// サービスを起動する
    Up(ComposeUpArgs),
    /// サービスを停止する
    Down(ComposeDownArgs),
    /// サービスのログを表示する
    Logs(ComposeLogsArgs),
}

/// `k1s0 docker compose up` の引数
#[derive(Args, Debug)]
pub struct ComposeUpArgs {
    /// バックグラウンドで起動
    #[arg(short, long)]
    pub detach: bool,

    /// ビルドしてから起動
    #[arg(long)]
    pub build: bool,
}

/// `k1s0 docker compose down` の引数
#[derive(Args, Debug)]
pub struct ComposeDownArgs {
    /// ボリュームも削除
    #[arg(long)]
    pub volumes: bool,
}

/// `k1s0 docker compose logs` の引数
#[derive(Args, Debug)]
pub struct ComposeLogsArgs {
    /// ログをフォローする
    #[arg(short, long)]
    pub follow: bool,

    /// 対象サービス名
    pub service: Option<String>,
}

/// `k1s0 docker status` の引数
#[derive(Args, Debug)]
pub struct StatusArgs {
    /// JSON 形式で出力
    #[arg(long)]
    pub json: bool,
}

/// `k1s0 docker` を実行する
pub fn execute(args: DockerArgs) -> Result<()> {
    match args.action {
        DockerAction::Build(build_args) => execute_build(build_args),
        DockerAction::Compose(compose_args) => execute_compose(compose_args),
        DockerAction::Status(status_args) => execute_status(status_args),
    }
}

/// Docker がインストールされているか確認
fn check_docker() -> Result<()> {
    match Command::new("docker").arg("--version").output() {
        Ok(out) if out.status.success() => Ok(()),
        _ => Err(CliError::validation(
            "Docker が検出されませんでした。\n\n\
             可観測性スタック（OTEL Collector, Jaeger, Loki, Prometheus, Grafana）の\n\
             起動には Docker Desktop が必要です。\n\n\
             サービスの開発・ビルド・テストは Docker なしで実行可能です。\n\
             可観測性が不要な場合、このエラーは無視できます。\n\n\
             Docker Desktop のインストール:\n  \
             https://docs.docker.com/desktop/install/windows-install/",
        )),
    }
}

/// docker compose が使えるか確認
fn check_docker_compose() -> Result<()> {
    match Command::new("docker")
        .args(["compose", "version"])
        .output()
    {
        Ok(out) if out.status.success() => Ok(()),
        _ => Err(
            CliError::validation("Docker Compose v2 がインストールされていません")
                .with_hint("Docker Desktop をインストールするか、docker compose プラグインを追加してください"),
        ),
    }
}

/// manifest.json からサービス名とテンプレートバージョンを読み取る
fn read_manifest() -> Result<(String, String)> {
    let manifest_path = Path::new(".k1s0/manifest.json");
    if !manifest_path.exists() {
        return Err(CliError::validation(
            ".k1s0/manifest.json が見つかりません",
        )
        .with_hint("k1s0 new-feature で生成されたディレクトリ内で実行してください"));
    }

    let content = std::fs::read_to_string(manifest_path)
        .map_err(|e| CliError::io(format!("manifest.json の読み取りに失敗: {e}")))?;

    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| CliError::io(format!("manifest.json のパースに失敗: {e}")))?;

    let service_name = json["service"]["service_name"]
        .as_str()
        .unwrap_or("app")
        .to_string();
    let template_version = json["template"]["version"]
        .as_str()
        .unwrap_or("0.1.0")
        .to_string();

    Ok((service_name, template_version))
}

/// モノレポルートを検出する
///
/// カレントディレクトリから親を辿り、`.k1s0/manifest.json` の `layer` が
/// "feature" であることを確認した上で、モノレポルート（4 階層上）を返す。
fn find_monorepo_root() -> Result<std::path::PathBuf> {
    let current = std::env::current_dir()
        .map_err(|e| CliError::io(format!("カレントディレクトリの取得に失敗: {e}")))?;

    // manifest.json から feature_relative_path のレベル数を推定
    // feature/backend/{lang}/{name} → 4 階層上がモノレポルート
    let mut root = current.clone();
    for _ in 0..4 {
        if let Some(parent) = root.parent() {
            root = parent.to_path_buf();
        } else {
            return Err(CliError::validation(
                "モノレポルートが見つかりません",
            )
            .with_hint("feature ディレクトリ内で実行してください"));
        }
    }

    Ok(root)
}

/// `k1s0 docker build` を実行
fn execute_build(args: BuildArgs) -> Result<()> {
    let out = output();
    check_docker()?;

    if !Path::new("Dockerfile").exists() {
        return Err(CliError::validation("Dockerfile が見つかりません")
            .with_hint("k1s0 new-feature で生成されたディレクトリ内で実行してください"));
    }

    let (service_name, template_version) = read_manifest()?;
    let tag = args
        .tag
        .unwrap_or_else(|| format!("{}:{}", service_name, template_version));

    // モノレポルートを検出してビルドコンテキストとする
    let monorepo_root = find_monorepo_root()?;
    let current_dir = std::env::current_dir()
        .map_err(|e| CliError::io(format!("カレントディレクトリの取得に失敗: {e}")))?;
    let dockerfile_path = current_dir.join("Dockerfile");

    out.info(&format!("Docker イメージをビルドしています: {}", tag));
    out.info(&format!("ビルドコンテキスト: {}", monorepo_root.display()));

    let mut cmd = Command::new("docker");
    cmd.args(["build", "-f"]);
    cmd.arg(&dockerfile_path);
    cmd.args(["-t", &tag]);

    if args.no_cache {
        cmd.arg("--no-cache");
    }

    if let Some(proxy) = &args.http_proxy {
        cmd.args(["--build-arg", &format!("HTTP_PROXY={proxy}")]);
    }
    if let Some(proxy) = &args.https_proxy {
        cmd.args(["--build-arg", &format!("HTTPS_PROXY={proxy}")]);
    }

    cmd.arg(&monorepo_root);

    let status = cmd
        .status()
        .map_err(|e| CliError::io(format!("docker build の実行に失敗: {e}")))?;

    if status.success() {
        out.success(&format!("イメージのビルドが完了しました: {}", tag));
        Ok(())
    } else {
        Err(CliError::validation("docker build が失敗しました"))
    }
}

/// `k1s0 docker compose` を実行
fn execute_compose(args: ComposeArgs) -> Result<()> {
    check_docker()?;
    check_docker_compose()?;

    if !Path::new("docker-compose.yml").exists() {
        return Err(CliError::validation("docker-compose.yml が見つかりません")
            .with_hint("k1s0 new-feature で生成されたディレクトリ内で実行してください"));
    }

    match args.action {
        ComposeAction::Up(up_args) => execute_compose_up(up_args),
        ComposeAction::Down(down_args) => execute_compose_down(down_args),
        ComposeAction::Logs(logs_args) => execute_compose_logs(logs_args),
    }
}

fn execute_compose_up(args: ComposeUpArgs) -> Result<()> {
    let out = output();
    out.info("サービスを起動しています...");

    let mut cmd = Command::new("docker");
    cmd.args(["compose", "up"]);

    if args.detach {
        cmd.arg("-d");
    }
    if args.build {
        cmd.arg("--build");
    }

    let status = cmd
        .status()
        .map_err(|e| CliError::io(format!("docker compose up の実行に失敗: {e}")))?;

    if status.success() {
        out.success("サービスが起動しました");
        Ok(())
    } else {
        Err(CliError::validation("docker compose up が失敗しました"))
    }
}

fn execute_compose_down(args: ComposeDownArgs) -> Result<()> {
    let out = output();
    out.info("サービスを停止しています...");

    let mut cmd = Command::new("docker");
    cmd.args(["compose", "down"]);

    if args.volumes {
        cmd.arg("-v");
    }

    let status = cmd
        .status()
        .map_err(|e| CliError::io(format!("docker compose down の実行に失敗: {e}")))?;

    if status.success() {
        out.success("サービスが停止しました");
        Ok(())
    } else {
        Err(CliError::validation("docker compose down が失敗しました"))
    }
}

fn execute_compose_logs(args: ComposeLogsArgs) -> Result<()> {
    let mut cmd = Command::new("docker");
    cmd.args(["compose", "logs"]);

    if args.follow {
        cmd.arg("-f");
    }
    if let Some(service) = &args.service {
        cmd.arg(service);
    }

    let status = cmd
        .status()
        .map_err(|e| CliError::io(format!("docker compose logs の実行に失敗: {e}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(CliError::validation("docker compose logs が失敗しました"))
    }
}

/// `k1s0 docker status` を実行
fn execute_status(args: StatusArgs) -> Result<()> {
    let out = output();
    check_docker()?;
    check_docker_compose()?;

    if !Path::new("docker-compose.yml").exists() {
        return Err(CliError::validation("docker-compose.yml が見つかりません")
            .with_hint("k1s0 new-feature で生成されたディレクトリ内で実行してください"));
    }

    if args.json {
        let result = Command::new("docker")
            .args(["compose", "ps", "--format", "json"])
            .output()
            .map_err(|e| CliError::io(format!("docker compose ps の実行に失敗: {e}")))?;

        if result.status.success() {
            println!("{}", String::from_utf8_lossy(&result.stdout));
        }
    } else {
        out.info("コンテナ状態:");
        let status = Command::new("docker")
            .args(["compose", "ps"])
            .status()
            .map_err(|e| CliError::io(format!("docker compose ps の実行に失敗: {e}")))?;

        if !status.success() {
            return Err(CliError::validation("docker compose ps が失敗しました"));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    #[derive(Parser)]
    struct TestCli {
        #[command(subcommand)]
        command: super::DockerAction,
    }

    #[test]
    fn test_parse_build() {
        let cli = TestCli::parse_from(["test", "build"]);
        assert!(matches!(cli.command, super::DockerAction::Build(_)));
    }

    #[test]
    fn test_parse_build_with_tag() {
        let cli = TestCli::parse_from(["test", "build", "--tag", "my-app:1.0"]);
        match cli.command {
            super::DockerAction::Build(args) => {
                assert_eq!(args.tag, Some("my-app:1.0".to_string()));
            }
            _ => panic!("Expected Build"),
        }
    }

    #[test]
    fn test_parse_build_no_cache() {
        let cli = TestCli::parse_from(["test", "build", "--no-cache"]);
        match cli.command {
            super::DockerAction::Build(args) => {
                assert!(args.no_cache);
            }
            _ => panic!("Expected Build"),
        }
    }

    #[test]
    fn test_parse_compose_up() {
        let cli = TestCli::parse_from(["test", "compose", "up"]);
        match cli.command {
            super::DockerAction::Compose(args) => {
                assert!(matches!(args.action, super::ComposeAction::Up(_)));
            }
            _ => panic!("Expected Compose"),
        }
    }

    #[test]
    fn test_parse_compose_up_detach_build() {
        let cli = TestCli::parse_from(["test", "compose", "up", "-d", "--build"]);
        match cli.command {
            super::DockerAction::Compose(args) => match args.action {
                super::ComposeAction::Up(up) => {
                    assert!(up.detach);
                    assert!(up.build);
                }
                _ => panic!("Expected Up"),
            },
            _ => panic!("Expected Compose"),
        }
    }

    #[test]
    fn test_parse_compose_down() {
        let cli = TestCli::parse_from(["test", "compose", "down"]);
        match cli.command {
            super::DockerAction::Compose(args) => {
                assert!(matches!(args.action, super::ComposeAction::Down(_)));
            }
            _ => panic!("Expected Compose"),
        }
    }

    #[test]
    fn test_parse_compose_down_volumes() {
        let cli = TestCli::parse_from(["test", "compose", "down", "--volumes"]);
        match cli.command {
            super::DockerAction::Compose(args) => match args.action {
                super::ComposeAction::Down(down) => {
                    assert!(down.volumes);
                }
                _ => panic!("Expected Down"),
            },
            _ => panic!("Expected Compose"),
        }
    }

    #[test]
    fn test_parse_compose_logs() {
        let cli = TestCli::parse_from(["test", "compose", "logs"]);
        match cli.command {
            super::DockerAction::Compose(args) => {
                assert!(matches!(args.action, super::ComposeAction::Logs(_)));
            }
            _ => panic!("Expected Compose"),
        }
    }

    #[test]
    fn test_parse_compose_logs_follow_service() {
        let cli = TestCli::parse_from(["test", "compose", "logs", "-f", "app"]);
        match cli.command {
            super::DockerAction::Compose(args) => match args.action {
                super::ComposeAction::Logs(logs) => {
                    assert!(logs.follow);
                    assert_eq!(logs.service, Some("app".to_string()));
                }
                _ => panic!("Expected Logs"),
            },
            _ => panic!("Expected Compose"),
        }
    }

    #[test]
    fn test_parse_status() {
        let cli = TestCli::parse_from(["test", "status"]);
        assert!(matches!(cli.command, super::DockerAction::Status(_)));
    }

    #[test]
    fn test_parse_status_json() {
        let cli = TestCli::parse_from(["test", "status", "--json"]);
        match cli.command {
            super::DockerAction::Status(args) => {
                assert!(args.json);
            }
            _ => panic!("Expected Status"),
        }
    }

    #[test]
    fn test_parse_build_with_proxy() {
        let cli = TestCli::parse_from(["test", "build", "--http-proxy", "http://proxy:8080", "--https-proxy", "https://proxy:8443"]);
        match cli.command {
            super::DockerAction::Build(args) => {
                assert_eq!(args.http_proxy, Some("http://proxy:8080".to_string()));
                assert_eq!(args.https_proxy, Some("https://proxy:8443".to_string()));
            }
            _ => panic!("Expected Build"),
        }
    }

    #[test]
    fn test_tag_generation() {
        let service_name = "my-service";
        let template_version = "0.1.0";
        let tag = format!("{}:{}", service_name, template_version);
        assert_eq!(tag, "my-service:0.1.0");
    }
}
