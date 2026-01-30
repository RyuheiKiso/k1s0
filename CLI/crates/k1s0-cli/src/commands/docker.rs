//! `k1s0 docker` コマンド
//!
//! Docker Compose を使用したサービスの管理。

use std::path::PathBuf;
use std::process::Command;

use clap::{Args, Subcommand};

use crate::error::{CliError, Result};
use crate::output::output;
use crate::settings::Settings;

/// `k1s0 docker` の引数
#[derive(Args, Debug)]
#[command(about = "Docker Compose でサービスを管理する", after_long_help = r#"例:
  k1s0 docker up                    # サービスを起動
  k1s0 docker up --monorepo         # monorepo モードで起動
  k1s0 docker down --volumes        # サービスを停止しボリュームを削除
  k1s0 docker logs -f my-service    # ログをフォロー
  k1s0 docker build --no-cache      # キャッシュなしでビルド
  k1s0 docker build --tag v1.0.0    # タグを指定してビルド
  k1s0 docker ps                    # ステータス表示
"#)]
pub struct DockerArgs {
    /// サブコマンド
    #[command(subcommand)]
    pub command: DockerCommand,
}

/// Docker サブコマンド
#[derive(Subcommand, Debug)]
pub enum DockerCommand {
    /// サービスを起動する
    Up(DockerUpArgs),
    /// サービスを停止する
    Down(DockerDownArgs),
    /// サービスのログを表示する
    Logs(DockerLogsArgs),
    /// Docker イメージをビルドする
    Build(DockerBuildArgs),
    /// サービスのステータスを表示する
    Ps(DockerPsArgs),
}

/// `k1s0 docker up` の引数
#[derive(Args, Debug)]
pub struct DockerUpArgs {
    /// デタッチモードで起動する
    #[arg(short, long, default_value_t = true)]
    pub detach: bool,

    /// monorepo モードを使用する
    #[arg(long)]
    pub monorepo: bool,

    /// 対象サービスのディレクトリ
    #[arg(short, long)]
    pub path: Option<String>,

    /// 全サービスを統合起動する
    #[arg(long)]
    pub all: bool,

    /// 追加の docker compose 引数
    #[arg(trailing_var_arg = true)]
    pub extra_args: Vec<String>,
}

/// `k1s0 docker down` の引数
#[derive(Args, Debug)]
pub struct DockerDownArgs {
    /// ボリュームも削除する
    #[arg(long)]
    pub volumes: bool,

    /// monorepo モードを使用する
    #[arg(long)]
    pub monorepo: bool,

    /// 対象サービスのディレクトリ
    #[arg(short, long)]
    pub path: Option<String>,
}

/// `k1s0 docker logs` の引数
#[derive(Args, Debug)]
pub struct DockerLogsArgs {
    /// ログをフォローする
    #[arg(short, long)]
    pub follow: bool,

    /// 表示する行数
    #[arg(short, long, default_value = "100")]
    pub tail: String,

    /// 対象サービス名
    pub service: Option<String>,

    /// 対象サービスのディレクトリ
    #[arg(short, long)]
    pub path: Option<String>,

    /// monorepo モードを使用する
    #[arg(long)]
    pub monorepo: bool,
}

/// `k1s0 docker build` の引数
#[derive(Args, Debug)]
pub struct DockerBuildArgs {
    /// キャッシュを使用しない
    #[arg(long)]
    pub no_cache: bool,

    /// monorepo モードを使用する
    #[arg(long)]
    pub monorepo: bool,

    /// 対象サービスのディレクトリ
    #[arg(short, long)]
    pub path: Option<String>,

    /// イメージタグ
    #[arg(long)]
    pub tag: Option<String>,

    /// レジストリへ Push する
    #[arg(long)]
    pub push: bool,

    /// ビルドプラットフォーム（例: linux/amd64,linux/arm64）
    #[arg(long)]
    pub platform: Option<String>,
}

/// `k1s0 docker ps` の引数
#[derive(Args, Debug)]
pub struct DockerPsArgs {
    /// 対象サービスのディレクトリ
    #[arg(short, long)]
    pub path: Option<String>,

    /// monorepo モードを使用する
    #[arg(long)]
    pub monorepo: bool,

    /// JSON 形式で出力する
    #[arg(long)]
    pub json: bool,
}

/// `k1s0 docker` を実行する
pub fn execute(args: DockerArgs) -> Result<()> {
    // Docker が利用可能かチェック
    check_docker_available()?;

    match args.command {
        DockerCommand::Up(sub) => execute_up(sub),
        DockerCommand::Down(sub) => execute_down(sub),
        DockerCommand::Logs(sub) => execute_logs(sub),
        DockerCommand::Build(sub) => execute_build(sub),
        DockerCommand::Ps(sub) => execute_ps(sub),
    }
}

/// Docker が利用可能かチェック
fn check_docker_available() -> Result<()> {
    // Docker コマンドの存在チェック
    let version_result = Command::new("docker").arg("--version").output();
    match version_result {
        Ok(output) if output.status.success() => {}
        _ => {
            return Err(CliError::config("Docker が見つかりません")
                .with_hint("Docker Desktop をインストールしてください")
                .with_recovery("https://docs.docker.com/get-docker/", "Docker のインストール"));
        }
    }

    // Docker デーモンの起動チェック
    let info_result = Command::new("docker").arg("info").output();
    match info_result {
        Ok(output) if output.status.success() => {}
        _ => {
            return Err(CliError::config("Docker デーモンが起動していません")
                .with_hint("Docker Desktop を起動するか、Docker サービスを開始してください")
                .with_recovery("docker desktop", "Docker Desktop の起動"));
        }
    }

    // Docker Compose V2 チェック
    let compose_result = Command::new("docker").args(["compose", "version"]).output();
    match compose_result {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err(CliError::config("Docker Compose V2 が利用できません")
            .with_hint("Docker Compose プラグインをインストールしてください")
            .with_recovery("https://docs.docker.com/compose/install/", "Docker Compose のインストール")),
    }
}

/// compose ファイルのパスを解決する
fn resolve_compose_file(path: Option<&str>, monorepo: bool) -> Result<PathBuf> {
    let base_dir = if let Some(p) = path {
        PathBuf::from(p)
    } else {
        std::env::current_dir().map_err(|e| {
            CliError::io(format!("カレントディレクトリの取得に失敗: {}", e))
        })?
    };

    let filename = if monorepo {
        "compose.monorepo.yaml"
    } else {
        "compose.yaml"
    };

    let compose_path = base_dir.join(filename);
    if !compose_path.exists() {
        return Err(CliError::config(format!(
            "{} が見つかりません",
            filename
        ))
        .with_target(compose_path.display().to_string())
        .with_hint("k1s0 new-feature で生成されたディレクトリから実行してください"));
    }

    Ok(compose_path)
}

/// docker compose コマンドを実行する
fn run_compose(
    compose_file: &PathBuf,
    args: &[&str],
) -> Result<()> {
    let out = output();

    let compose_dir = compose_file.parent().unwrap_or_else(|| std::path::Path::new("."));

    out.info(&format!(
        "docker compose -f {} {}",
        compose_file.display(),
        args.join(" ")
    ));

    let mut cmd = Command::new("docker");
    cmd.arg("compose")
        .arg("-f")
        .arg(compose_file)
        .args(args)
        .current_dir(compose_dir);

    let status = cmd
        .status()
        .map_err(|e| CliError::io(format!("docker compose の実行に失敗: {}", e)))?;

    if !status.success() {
        return Err(CliError::internal(format!(
            "docker compose がエラーコード {} で終了しました",
            status.code().unwrap_or(-1)
        ))
        .with_hint(
            "失敗した場合: (1) ポート競合 → compose.yaml の ports を変更 \
             (2) イメージ不在 → k1s0 docker build を先に実行 \
             (3) ネットワーク問題 → docker network prune を実行",
        ));
    }

    Ok(())
}

/// compose で定義されたイメージにタグを付与する
fn tag_images(compose_file: &PathBuf, tag: &str) -> Result<()> {
    let compose_dir = compose_file.parent().unwrap_or_else(|| std::path::Path::new("."));

    // docker compose images でイメージ一覧を取得
    let output_result = Command::new("docker")
        .arg("compose")
        .arg("-f")
        .arg(compose_file)
        .arg("images")
        .arg("--format")
        .arg("json")
        .current_dir(compose_dir)
        .output()
        .map_err(|e| CliError::io(format!("docker compose images の実行に失敗: {}", e)))?;

    if !output_result.status.success() {
        return Err(CliError::internal("docker compose images に失敗しました".to_string()));
    }

    let stdout = String::from_utf8_lossy(&output_result.stdout);
    // JSON の各行からイメージ名を抽出してタグ付け
    for line in stdout.lines() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(repository) = json.get("Repository").and_then(|v| v.as_str()) {
                if !repository.is_empty() && repository != "<none>" {
                    let new_tag = format!("{}:{}", repository, tag);
                    let current = if let Some(t) = json.get("Tag").and_then(|v| v.as_str()) {
                        format!("{}:{}", repository, t)
                    } else {
                        format!("{}:latest", repository)
                    };

                    let status = Command::new("docker")
                        .args(["tag", &current, &new_tag])
                        .status()
                        .map_err(|e| CliError::io(format!("docker tag の実行に失敗: {}", e)))?;

                    if status.success() {
                        let out = output();
                        out.info(&format!("  {} → {}", current, new_tag));
                    }
                }
            }
        }
    }

    Ok(())
}

/// レジストリ設定を `.k1s0/settings.yaml` から読み込む
fn load_registry_config() -> Option<String> {
    let settings = Settings::load(None).ok()?;
    settings.registry.url.filter(|v| !v.is_empty())
}

/// プロジェクトルートを検索する
fn find_project_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()
        .map_err(|e| CliError::io(format!("カレントディレクトリの取得に失敗: {}", e)))?;

    for _ in 0..10 {
        if current.join(".k1s0").exists() || current.join("compose.yaml").exists() {
            return Ok(current);
        }
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }

    Err(CliError::config("プロジェクトルートが見つかりません")
        .with_hint("k1s0 init でプロジェクトを初期化してください"))
}

/// `k1s0 docker up` を実行
fn execute_up(args: DockerUpArgs) -> Result<()> {
    if args.all {
        let project_root = find_project_root()?;
        let services = crate::docker::compose_aggregator::discover_compose_files(&project_root)?;
        if services.is_empty() {
            return Err(CliError::config("feature サービスが見つかりません")
                .with_hint("k1s0 new-feature でサービスを作成してください"));
        }

        let out = output();
        out.info(&format!("{} 個のサービスを検出しました", services.len()));
        for svc in &services {
            out.list_item("service", &svc.name);
        }

        // Generate aggregate compose
        let aggregate = crate::docker::compose_aggregator::generate_aggregate_compose(&services, args.monorepo);
        let temp_compose = std::env::temp_dir().join("k1s0-aggregate-compose.yaml");
        std::fs::write(&temp_compose, &aggregate)
            .map_err(|e| CliError::io(format!("aggregate compose の書き込みに失敗: {}", e)))?;

        let mut compose_args = vec!["up"];
        if args.detach {
            compose_args.push("-d");
        }

        run_compose(&temp_compose, &compose_args)?;

        // Cleanup
        let _ = std::fs::remove_file(&temp_compose);

        out.success("全サービスを起動しました");
        return Ok(());
    }

    let compose_file = resolve_compose_file(args.path.as_deref(), args.monorepo)?;

    let mut compose_args = vec!["up"];
    if args.detach {
        compose_args.push("-d");
    }

    for arg in &args.extra_args {
        compose_args.push(arg);
    }

    run_compose(&compose_file, &compose_args)?;

    let out = output();
    out.success("サービスを起動しました");
    Ok(())
}

/// `k1s0 docker down` を実行
fn execute_down(args: DockerDownArgs) -> Result<()> {
    let compose_file = resolve_compose_file(args.path.as_deref(), args.monorepo)?;

    let mut compose_args = vec!["down"];
    if args.volumes {
        compose_args.push("-v");
    }

    run_compose(&compose_file, &compose_args)?;

    let out = output();
    out.success("サービスを停止しました");
    Ok(())
}

/// `k1s0 docker logs` を実行
fn execute_logs(args: DockerLogsArgs) -> Result<()> {
    let compose_file = resolve_compose_file(args.path.as_deref(), args.monorepo)?;

    let mut compose_args = vec!["logs"];
    if args.follow {
        compose_args.push("-f");
    }
    compose_args.push("--tail");
    compose_args.push(&args.tail);

    if let Some(ref service) = args.service {
        compose_args.push(service);
    }

    run_compose(&compose_file, &compose_args)
}

/// `k1s0 docker build` を実行
fn execute_build(args: DockerBuildArgs) -> Result<()> {
    let compose_file = resolve_compose_file(args.path.as_deref(), args.monorepo)?;

    let mut compose_args = vec!["build"];
    if args.no_cache {
        compose_args.push("--no-cache");
    }

    run_compose(&compose_file, &compose_args)?;

    // タグ付け
    if let Some(ref tag) = args.tag {
        let out = output();
        out.info(&format!("イメージにタグ '{}' を付与中...", tag));
        tag_images(&compose_file, tag)?;

        // レジストリ設定があれば表示
        if let Some(registry) = load_registry_config() {
            out.info(&format!("レジストリ: {}", registry));
        }
    }

    // Push
    if args.push {
        let out = output();
        out.info("イメージを Push 中...");
        run_compose(&compose_file, &["push"])?;
    }

    let out = output();
    out.success("イメージをビルドしました");
    Ok(())
}

/// `k1s0 docker ps` を実行
fn execute_ps(args: DockerPsArgs) -> Result<()> {
    let compose_file = resolve_compose_file(args.path.as_deref(), args.monorepo)?;

    if args.json {
        run_compose(&compose_file, &["ps", "--format", "json"])
    } else {
        run_compose(&compose_file, &["ps"])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Args, Command};

    fn docker_cmd() -> Command {
        DockerArgs::augment_args(Command::new("docker"))
    }

    #[test]
    fn test_docker_args_parse_up() {
        let m = docker_cmd()
            .try_get_matches_from(["docker", "up"])
            .expect("parse failed");
        let sub = m.subcommand_matches("up").expect("no up subcommand");
        // detach defaults to true
        assert_eq!(sub.get_one::<bool>("detach").copied(), Some(true));
        assert_eq!(sub.get_one::<bool>("monorepo").copied(), Some(false));
    }

    #[test]
    fn test_docker_args_parse_down_volumes() {
        let m = docker_cmd()
            .try_get_matches_from(["docker", "down", "--volumes"])
            .expect("parse failed");
        let sub = m.subcommand_matches("down").expect("no down subcommand");
        assert_eq!(sub.get_one::<bool>("volumes").copied(), Some(true));
    }

    #[test]
    fn test_docker_args_parse_build_tag_push_platform() {
        let m = docker_cmd()
            .try_get_matches_from([
                "docker",
                "build",
                "--tag",
                "v1.0.0",
                "--push",
                "--platform",
                "linux/amd64",
            ])
            .expect("parse failed");
        let sub = m.subcommand_matches("build").expect("no build subcommand");
        assert_eq!(
            sub.get_one::<String>("tag").map(String::as_str),
            Some("v1.0.0")
        );
        assert_eq!(sub.get_one::<bool>("push").copied(), Some(true));
        assert_eq!(
            sub.get_one::<String>("platform").map(String::as_str),
            Some("linux/amd64")
        );
    }

    #[test]
    fn test_docker_args_parse_logs_follow_tail() {
        let m = docker_cmd()
            .try_get_matches_from(["docker", "logs", "-f", "--tail", "50"])
            .expect("parse failed");
        let sub = m.subcommand_matches("logs").expect("no logs subcommand");
        assert_eq!(sub.get_one::<bool>("follow").copied(), Some(true));
        assert_eq!(
            sub.get_one::<String>("tail").map(String::as_str),
            Some("50")
        );
    }

    #[test]
    fn test_docker_args_parse_ps_json() {
        let m = docker_cmd()
            .try_get_matches_from(["docker", "ps", "--json"])
            .expect("parse failed");
        let sub = m.subcommand_matches("ps").expect("no ps subcommand");
        assert_eq!(sub.get_one::<bool>("json").copied(), Some(true));
    }

    #[test]
    fn test_resolve_compose_file_not_found() {
        let tmp = tempfile::tempdir().expect("failed to create tempdir");
        let result = resolve_compose_file(Some(tmp.path().to_str().unwrap()), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_compose_file_found() {
        let tmp = tempfile::tempdir().expect("failed to create tempdir");
        std::fs::write(tmp.path().join("compose.yaml"), "version: '3'")
            .expect("failed to write compose.yaml");
        let result = resolve_compose_file(Some(tmp.path().to_str().unwrap()), false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), tmp.path().join("compose.yaml"));
    }

    #[test]
    fn test_resolve_compose_file_monorepo() {
        let tmp = tempfile::tempdir().expect("failed to create tempdir");
        std::fs::write(tmp.path().join("compose.monorepo.yaml"), "version: '3'")
            .expect("failed to write compose.monorepo.yaml");
        let result = resolve_compose_file(Some(tmp.path().to_str().unwrap()), true);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            tmp.path().join("compose.monorepo.yaml")
        );
    }

    #[test]
    fn test_help_output_contains_examples() {
        let mut cmd = docker_cmd();
        let help = cmd.render_long_help().to_string();
        assert!(help.contains("k1s0 docker up"));
        assert!(help.contains("k1s0 docker down --volumes"));
        assert!(help.contains("k1s0 docker build --tag v1.0.0"));
    }
}
