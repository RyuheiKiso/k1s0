/// Docker 操作モジュール。
///
/// `std::process::Command` で `docker compose` を実行する。
use anyhow::{bail, Result};
use std::path::Path;
use std::process::Command;

/// Docker が利用可能か確認する。
///
/// # Errors
///
/// docker コマンドが見つからない、または実行に失敗した場合にエラーを返す。
pub fn check_docker_available() -> Result<()> {
    let output = Command::new("docker").arg("info").output();

    match output {
        Ok(o) if o.status.success() => Ok(()),
        Ok(_) => bail!("Docker デーモンが起動していません。Docker Desktop を起動してください。"),
        Err(e) => {
            bail!("docker コマンドが見つかりません。Docker をインストールしてください。: {e}")
        }
    }
}

/// docker compose up -d を実行する。
///
/// # Errors
///
/// コマンドの実行に失敗した場合にエラーを返す。
pub fn compose_up(compose_dir: &Path) -> Result<()> {
    let status = Command::new("docker")
        .args(["compose", "up", "-d", "--wait"])
        .current_dir(compose_dir)
        .status()?;

    if !status.success() {
        bail!("docker compose up に失敗しました（終了コード: {status}）");
    }
    Ok(())
}

/// docker compose down を実行する。
///
/// # Errors
///
/// コマンドの実行に失敗した場合にエラーを返す。
pub fn compose_down(compose_dir: &Path, remove_volumes: bool) -> Result<()> {
    let mut args = vec!["compose", "down"];
    if remove_volumes {
        args.push("-v");
    }

    let status = Command::new("docker")
        .args(&args)
        .current_dir(compose_dir)
        .status()?;

    if !status.success() {
        bail!("docker compose down に失敗しました（終了コード: {status}）");
    }
    Ok(())
}

/// docker compose ps の出力を取得する。
///
/// # Errors
///
/// コマンドの実行に失敗した場合にエラーを返す。
pub fn compose_status(compose_dir: &Path) -> Result<String> {
    let output = Command::new("docker")
        .args(["compose", "ps"])
        .current_dir(compose_dir)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("docker compose ps に失敗しました: {stderr}");
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// docker compose logs -f を実行する。
///
/// ログはリアルタイムで stdout に出力される（フォアグラウンド実行）。
///
/// # Errors
///
/// コマンドの実行に失敗した場合にエラーを返す。
pub fn compose_logs(compose_dir: &Path, service: Option<&str>) -> Result<()> {
    let mut args = vec!["compose", "logs", "-f"];
    if let Some(svc) = service {
        args.push(svc);
    }

    let status = Command::new("docker")
        .args(&args)
        .current_dir(compose_dir)
        .status()?;

    if !status.success() {
        bail!("docker compose logs に失敗しました（終了コード: {status}）");
    }
    Ok(())
}

/// 全コンテナが healthy になるまで待機する。
///
/// `docker compose up --wait` が成功していれば通常は不要だが、
/// 追加の確認として使用する。
///
/// # Errors
///
/// タイムアウトした場合にエラーを返す。
pub fn wait_for_healthy(compose_dir: &Path, timeout_secs: u64) -> Result<()> {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(timeout_secs);

    loop {
        if start.elapsed() > timeout {
            bail!(
                "ヘルスチェックがタイムアウトしました（{timeout_secs}秒）。docker compose ps で状態を確認してください。"
            );
        }

        let output = Command::new("docker")
            .args(["compose", "ps", "--format", "json"])
            .current_dir(compose_dir)
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // すべてのコンテナが running であることを確認
            // "unhealthy" や "starting" が含まれていなければ OK
            if !stdout.is_empty() && !stdout.contains("unhealthy") && !stdout.contains("starting") {
                return Ok(());
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
