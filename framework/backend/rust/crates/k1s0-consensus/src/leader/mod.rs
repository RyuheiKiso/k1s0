//! リーダー選出モジュール。
//!
//! リース方式によるリーダー選出、ハートビート、変更監視を提供する。

#[cfg(feature = "postgres")]
pub mod db;
pub mod heartbeat;
pub mod metrics;
pub mod watcher;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::ConsensusResult;

/// リーダー選出の抽象 trait。
#[async_trait]
pub trait LeaderElector: Send + Sync {
    /// リースの取得を試みる。成功した場合は `LeaderLease` を返す。
    async fn try_acquire(&self) -> ConsensusResult<Option<LeaderLease>>;

    /// 既存のリースを更新する。
    async fn renew(&self, lease: &LeaderLease) -> ConsensusResult<bool>;

    /// リースを解放する。
    async fn release(&self, lease: &LeaderLease) -> ConsensusResult<()>;

    /// 現在のリーダーを取得する。
    async fn current_leader(&self) -> ConsensusResult<Option<LeaderLease>>;

    /// リーダー変更を監視する `LeaderWatcher` を返す。
    async fn watch(&self) -> ConsensusResult<LeaderWatcher>;
}

/// リーダーリース情報。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderLease {
    /// リースキー。
    pub lease_key: String,
    /// リースを保持しているノード ID。
    pub holder_id: String,
    /// フェンシングトークン（単調増加）。
    pub fence_token: u64,
    /// リースの有効期限。
    pub expires_at: DateTime<Utc>,
}

impl LeaderLease {
    /// リースが有効かどうか判定する。
    #[must_use]
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }

    /// リースの残り時間（秒）を返す。期限切れの場合は 0。
    #[must_use]
    pub fn remaining_secs(&self) -> u64 {
        let remaining = self.expires_at - Utc::now();
        remaining.num_seconds().max(0).cast_unsigned()
    }
}

/// リーダー変更イベント。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeaderEvent {
    /// 自ノードがリーダーに選出された。
    Elected {
        /// フェンシングトークン。
        fence_token: u64,
    },
    /// リーダーシップを喪失した。
    Lost,
    /// リーダーが変更された。
    Changed {
        /// 新しいリーダーのノード ID。
        new_leader: String,
    },
}

/// リーダー変更を監視するウォッチャー。
pub struct LeaderWatcher {
    rx: tokio::sync::watch::Receiver<LeaderEvent>,
}

impl LeaderWatcher {
    /// 新しいウォッチャーを作成する。
    #[must_use]
    pub fn new(rx: tokio::sync::watch::Receiver<LeaderEvent>) -> Self {
        Self { rx }
    }

    /// 次のイベントを待機する。
    ///
    /// # Errors
    ///
    /// 送信側がドロップされた場合にエラーを返す。
    pub async fn next_event(&mut self) -> ConsensusResult<LeaderEvent> {
        self.rx
            .changed()
            .await
            .map_err(|e| crate::error::ConsensusError::Config(e.to_string()))?;
        Ok(self.rx.borrow().clone())
    }

    /// 現在のイベントを取得する（待機なし）。
    #[must_use]
    pub fn current(&self) -> LeaderEvent {
        self.rx.borrow().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lease_validity() {
        let lease = LeaderLease {
            lease_key: "test".into(),
            holder_id: "node-1".into(),
            fence_token: 1,
            expires_at: Utc::now() + chrono::Duration::seconds(30),
        };
        assert!(lease.is_valid());
        assert!(lease.remaining_secs() > 0);
    }

    #[test]
    fn test_lease_expired() {
        let lease = LeaderLease {
            lease_key: "test".into(),
            holder_id: "node-1".into(),
            fence_token: 1,
            expires_at: Utc::now() - chrono::Duration::seconds(10),
        };
        assert!(!lease.is_valid());
        assert_eq!(lease.remaining_secs(), 0);
    }

    #[test]
    fn test_leader_event_equality() {
        assert_eq!(
            LeaderEvent::Elected { fence_token: 1 },
            LeaderEvent::Elected { fence_token: 1 }
        );
        assert_ne!(LeaderEvent::Lost, LeaderEvent::Elected { fence_token: 1 });
    }
}
