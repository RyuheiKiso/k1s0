//! リーダー変更監視のユーティリティ。

use std::sync::Arc;
use std::time::Duration;

use crate::leader::{LeaderElector, LeaderEvent, LeaderWatcher};

/// リーダー変更をポーリング監視し、`LeaderWatcher` を返すユーティリティ。
///
/// `LeaderElector::watch()` のデフォルト実装がないバックエンド向けに、
/// `current_leader()` をポーリングして変更を検出する汎用実装。
pub fn poll_leader_changes(
    elector: Arc<dyn LeaderElector>,
    node_id: String,
    poll_interval: Duration,
) -> LeaderWatcher {
    let (tx, rx) = tokio::sync::watch::channel(LeaderEvent::Lost);

    tokio::spawn(async move {
        let mut last_leader: Option<String> = None;
        let mut interval = tokio::time::interval(poll_interval);

        loop {
            interval.tick().await;

            match elector.current_leader().await {
                Ok(Some(lease)) => {
                    let changed = last_leader.as_deref() != Some(&lease.holder_id);
                    if changed {
                        let event = if lease.holder_id == node_id {
                            LeaderEvent::Elected {
                                fence_token: lease.fence_token,
                            }
                        } else {
                            LeaderEvent::Changed {
                                new_leader: lease.holder_id.clone(),
                            }
                        };
                        last_leader = Some(lease.holder_id);
                        if tx.send(event).is_err() {
                            break;
                        }
                    }
                }
                Ok(None) => {
                    if last_leader.take().is_some()
                        && tx.send(LeaderEvent::Lost).is_err()
                    {
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "leader poll failed");
                }
            }
        }
    });

    LeaderWatcher::new(rx)
}
