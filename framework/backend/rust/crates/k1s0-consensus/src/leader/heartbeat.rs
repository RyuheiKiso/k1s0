//! リーダーリースのハートビート（自動更新）。

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::watch;

use crate::config::LeaderConfig;
use crate::leader::{LeaderElector, LeaderLease};

/// ハートビートタスクのハンドル。
pub struct HeartbeatHandle {
    cancel_tx: watch::Sender<bool>,
}

impl HeartbeatHandle {
    /// ハートビートを停止する。
    pub fn stop(self) {
        let _ = self.cancel_tx.send(true);
    }
}

/// ハートビートタスクを開始する。
///
/// リーダーリースを定期的に更新し、更新に失敗した場合はログを出力する。
/// 返された `HeartbeatHandle` をドロップまたは `stop()` でタスクを停止できる。
pub fn start_heartbeat(
    elector: Arc<dyn LeaderElector>,
    lease: LeaderLease,
    config: &LeaderConfig,
) -> HeartbeatHandle {
    let (cancel_tx, mut cancel_rx) = watch::channel(false);
    let interval = Duration::from_secs(config.heartbeat_interval_secs);

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        // 最初の tick はすぐに発火するのでスキップ
        ticker.tick().await;

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    match elector.renew(&lease).await {
                        Ok(true) => {
                            tracing::debug!(
                                lease_key = %lease.lease_key,
                                holder_id = %lease.holder_id,
                                "heartbeat: lease renewed"
                            );
                        }
                        Ok(false) => {
                            tracing::warn!(
                                lease_key = %lease.lease_key,
                                holder_id = %lease.holder_id,
                                "heartbeat: lease renewal failed, may have lost leadership"
                            );
                            super::metrics::heartbeat_failures().inc();
                            break;
                        }
                        Err(e) => {
                            tracing::error!(
                                lease_key = %lease.lease_key,
                                error = %e,
                                "heartbeat: error during lease renewal"
                            );
                            super::metrics::heartbeat_failures().inc();
                        }
                    }
                }
                _ = cancel_rx.changed() => {
                    tracing::info!(
                        lease_key = %lease.lease_key,
                        "heartbeat: stopped"
                    );
                    break;
                }
            }
        }
    });

    HeartbeatHandle { cancel_tx }
}
