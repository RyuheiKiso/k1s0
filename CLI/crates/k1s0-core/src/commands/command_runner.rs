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

pub(crate) fn run_streaming_command(
    cmd: &str,
    args: &[String],
    cwd: &Path,
    mut on_log: impl FnMut(String),
) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_run_streaming_command_captures_stdout_and_stderr() {
        let tmp = TempDir::new().unwrap();
        let (cmd, args) = echo_commands();
        let mut logs = Vec::new();

        run_streaming_command(&cmd, &args, tmp.path(), |message| logs.push(message)).unwrap();

        assert!(logs.iter().any(|message| message.contains("stdout-line")));
        assert!(logs
            .iter()
            .any(|message| message.contains("stderr | stderr-line")));
    }

    #[test]
    fn test_run_streaming_command_returns_exit_code_error() {
        let tmp = TempDir::new().unwrap();
        let (cmd, args) = failing_command();

        let error = run_streaming_command(&cmd, &args, tmp.path(), |_| {}).unwrap_err();

        assert!(error.to_string().contains("exited with"));
    }

    fn echo_commands() -> (String, Vec<String>) {
        if cfg!(windows) {
            (
                "cmd".to_string(),
                vec![
                    "/C".to_string(),
                    "(echo stdout-line) & (echo stderr-line 1>&2)".to_string(),
                ],
            )
        } else {
            (
                "sh".to_string(),
                vec![
                    "-lc".to_string(),
                    "printf 'stdout-line\\n'; printf 'stderr-line\\n' >&2".to_string(),
                ],
            )
        }
    }

    fn failing_command() -> (String, Vec<String>) {
        if cfg!(windows) {
            (
                "cmd".to_string(),
                vec!["/C".to_string(), "exit /B 7".to_string()],
            )
        } else {
            (
                "sh".to_string(),
                vec!["-lc".to_string(), "exit 7".to_string()],
            )
        }
    }
}
