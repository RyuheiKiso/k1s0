use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use anyhow::{anyhow, Result};

enum StreamKind {
    Stdout,
    Stderr,
}

/// CLI から実行を許可するコマンドのホワイトリスト（H-013 監査対応）
/// これ以外のコマンドは実行を拒否してコマンドインジェクションを防止する。
///
/// CLI-HIGH-002 監査対応で以下を除去:
///   - sh / bash / cmd: シェルインタプリタ経由のコマンドインジェクションリスクがあるため除去
///   - cosign / trivy: CLI 内に使用箇所が存在しないデッドコードのため除去
const ALLOWED_COMMANDS: &[&str] = &[
    "docker",
    "docker-compose",
    "kubectl",
    "helm",
    "cargo",
    "go",
    "flutter",
    "dart",
    "npm",
    "pnpm",
    "node",
    "buf",
    "just",
    "git",
];

pub(crate) fn run_streaming_command(
    cmd: &str,
    args: &[String],
    cwd: &Path,
    mut on_log: impl FnMut(String),
) -> Result<()> {
    // H-013 監査対応: ホワイトリスト外のコマンド実行を拒否してコマンドインジェクションを防止する
    if !ALLOWED_COMMANDS.contains(&cmd) {
        anyhow::bail!(
            "許可されていないコマンドです: '{cmd}'. 使用可能なコマンド: {ALLOWED_COMMANDS:?}"
        );
    }

    let mut child = Command::new(cmd)
        .args(args.iter().map(String::as_str))
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| anyhow!("failed to start {cmd}: {error}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("failed to capture stdout for {cmd}"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("failed to capture stderr for {cmd}"))?;

    let (tx, rx) = mpsc::channel();
    let stdout_handle = spawn_reader(stdout, StreamKind::Stdout, tx.clone());
    let stderr_handle = spawn_reader(stderr, StreamKind::Stderr, tx);

    for message in rx {
        on_log(message);
    }

    let status = child
        .wait()
        .map_err(|error| anyhow!("failed to wait for {cmd}: {error}"))?;

    join_reader(stdout_handle, cmd)?;
    join_reader(stderr_handle, cmd)?;

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("{cmd} exited with {}", status.code().unwrap_or(-1))
    }
}

fn spawn_reader<R>(
    reader: R,
    stream: StreamKind,
    tx: mpsc::Sender<String>,
) -> JoinHandle<Result<()>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let reader = BufReader::new(reader);
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let message = match stream {
                StreamKind::Stdout => line,
                StreamKind::Stderr => format!("stderr | {line}"),
            };

            if tx.send(message).is_err() {
                break;
            }
        }
        Ok(())
    })
}

fn join_reader(handle: JoinHandle<Result<()>>, cmd: &str) -> Result<()> {
    match handle.join() {
        Ok(result) => result,
        Err(_) => Err(anyhow!("log reader thread panicked while running {cmd}")),
    }
}

// テストコードでは unwrap() の使用を許可する
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_run_streaming_command_captures_stdout_and_stderr() {
        // CLI-HIGH-002 監査対応: sh/cmd はホワイトリストから除去されたため、
        // git --version を使ってストリーミング出力のキャプチャを検証する。
        let tmp = TempDir::new().unwrap();
        let mut logs = Vec::new();

        run_streaming_command(
            "git",
            &["--version".to_string()],
            tmp.path(),
            |message| logs.push(message),
        )
        .unwrap();

        assert!(logs.iter().any(|message| message.contains("git")));
    }

    #[test]
    fn test_run_streaming_command_returns_exit_code_error() {
        // CLI-HIGH-002 監査対応: sh/cmd はホワイトリストから除去されたため、
        // git に不正な引数を渡して非ゼロ終了コードを発生させて検証する。
        let tmp = TempDir::new().unwrap();

        let error = run_streaming_command(
            "git",
            &["invalid-subcommand-that-does-not-exist-xyz".to_string()],
            tmp.path(),
            |_| {},
        )
        .unwrap_err();

        assert!(error.to_string().contains("exited with"));
    }

    /// ホワイトリスト外のコマンドが拒否されることを検証する（H-013 監査対応テスト）
    #[test]
    fn run_streaming_command_rejects_unknown_commands() {
        let tmp = TempDir::new().unwrap();
        let result = run_streaming_command("malicious", &[], tmp.path(), |_| {});
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("許可されていないコマンド"));
    }
}
