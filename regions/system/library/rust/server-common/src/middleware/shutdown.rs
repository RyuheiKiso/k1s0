/// OS シグナル（Ctrl+C / SIGTERM）を待機する汎用シャットダウンシグナル。
pub async fn shutdown_signal() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut terminate = signal(SignalKind::terminate())?;
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = terminate.recv() => {}
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await?;
    }

    Ok(())
}
