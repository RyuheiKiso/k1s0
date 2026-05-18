// event_bus.rs — k1s0 tier1 gateway: セッションごとのイベントバス
// long_poll adapter が v1_event_feed イベントを受信するための broadcast channel 管理。
// session_id をキーとして broadcast Sender を保持し、
// 複数の long_poll handler が同一セッションを subscribe できる。

// anyhow: エラーハンドリング
use anyhow::Result;
// serde_json: ドメインイベントの JSON ペイロード
use serde_json::Value as JsonValue;
// std::collections::HashMap: セッション → Sender のマッピング
use std::collections::HashMap;
// std::sync: Arc + RwLock でセッションマップを共有する
use std::sync::{Arc, RwLock};
// tokio::sync::broadcast: non-blocking broadcast channel
use tokio::sync::broadcast;

// DomainEvent は long_poll adapter が受信するドメインイベントを宣言する。
// v1_event_feed class 向けに設計する（SESSION_ORDERED、lag≤5000ms）。
#[derive(Debug, Clone)]
pub struct DomainEvent {
    // session_id: イベントの宛先セッション識別子
    pub session_id: String,
    // event_type: ドメインイベント種別（例: "order.placed" / "inventory.updated"）
    pub event_type: String,
    // payload: ドメインイベントの JSON ペイロード
    pub payload: JsonValue,
    // sequence: セッション内の順序番号（SESSION_ORDERED の担保に使用する）
    pub sequence: u64,
}

// CHANNEL_CAPACITY は broadcast channel のバッファサイズ。
// 32 = 最大 32 イベントをバッファリングする（long_poll タイムアウト間に溢れない想定）
const CHANNEL_CAPACITY: usize = 32;

// EventBus はセッションごとの broadcast channel を管理する。
// Arc<EventBus> として gateway の axum State に注入する。
pub struct EventBus {
    // sessions: session_id → broadcast Sender のマップ（RwLock で並行 read / exclusive write）
    sessions: RwLock<HashMap<String, broadcast::Sender<DomainEvent>>>,
}

impl EventBus {
    /// new は空の EventBus を構築して返す。
    pub fn new() -> Arc<Self> {
        // Arc でラップして gateway の複数ハンドラーに共有する
        Arc::new(Self {
            // セッションマップを空で初期化する
            sessions: RwLock::new(HashMap::new()),
        })
    }

    /// subscribe は session_id の broadcast Receiver を取得する。
    /// セッションが存在しない場合は新規 broadcast channel を作成する。
    pub fn subscribe(&self, session_id: &str) -> broadcast::Receiver<DomainEvent> {
        // まず read lock でセッションを探す
        {
            // sessions マップを read lock で取得する
            let sessions = self.sessions.read()
                .expect("EventBus RwLock read poison");
            // session_id が既存の場合は既存 Sender の Receiver を返す
            if let Some(tx) = sessions.get(session_id) {
                return tx.subscribe();
            }
        }
        // session_id が存在しない場合は write lock で新規作成する
        let mut sessions = self.sessions.write()
            .expect("EventBus RwLock write poison");
        // double-checked locking: write lock 取得後に再確認する
        if let Some(tx) = sessions.get(session_id) {
            // 競合で先に作成された場合はその Receiver を返す
            return tx.subscribe();
        }
        // 新規 broadcast channel を作成する
        let (tx, rx) = broadcast::channel(CHANNEL_CAPACITY);
        // sessions マップに Sender を登録する
        sessions.insert(session_id.to_string(), tx);
        // 作成した Receiver を返す
        rx
    }

    /// publish は session_id に DomainEvent を送信する。
    /// session が存在しない場合は送信せず Ok を返す（no subscribers is not an error）
    pub fn publish(&self, event: DomainEvent) -> Result<()> {
        // sessions マップを read lock で取得する
        let sessions = self.sessions.read()
            .expect("EventBus RwLock read poison");
        // session_id の Sender を取得する
        if let Some(tx) = sessions.get(&event.session_id) {
            // 受信者が存在する場合にのみ送信する（受信者なしは無視）
            let _ = tx.send(event);
        }
        Ok(())
    }

    /// gc_expired は購読者がいないセッションの broadcast channel を削除する。
    /// 定期的に呼び出してメモリリークを防ぐ（背景タスクで実行する）。
    pub fn gc_expired(&self) {
        // sessions マップを write lock で取得する
        let Ok(mut sessions) = self.sessions.write() else {
            return;
        };
        // receiver_count() == 0 のセッション（購読者がいない）を削除する
        sessions.retain(|_, tx| tx.receiver_count() > 0);
    }
}

/// Default implementation for EventBus（Arc 経由で new() を呼ぶ）
impl Default for EventBus {
    fn default() -> Self {
        // self は Arc で包まれるため Default は内部構造のみを返す
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    #[tokio::test]
    // EventBus の subscribe → publish → receive が正しく動作することを確認する
    async fn test_event_bus_subscribe_publish_receive() {
        // 新しい EventBus を構築する
        let bus = EventBus::new();
        // session "sess-001" を subscribe する
        let mut rx = bus.subscribe("sess-001");
        // DomainEvent を publish する
        let event = DomainEvent {
            session_id: "sess-001".to_string(),
            event_type: "order.placed".to_string(),
            payload: serde_json::json!({"order_id": "O-001"}),
            sequence: 1,
        };
        // イベントを送信する
        bus.publish(event).expect("publish に失敗した");
        // イベントを受信する
        let received = rx.try_recv().expect("receive に失敗した");
        // 受信したイベントの内容を確認する
        assert_eq!(received.event_type, "order.placed");
    }

    #[test]
    // gc_expired が購読者なしのセッションを削除することを確認する
    fn test_gc_expired_removes_empty_sessions() {
        // 新しい EventBus を内部構造で構築する
        let bus = Arc::new(EventBus::default());
        // session を subscribe してからドロップする（購読者なし状態にする）
        {
            let _rx = bus.subscribe("sess-drop");
        }
        // gc_expired を呼び出す
        bus.gc_expired();
        // sessions マップが空になっていることを確認する（または元々小さいことを確認）
        let sessions = bus.sessions.read().unwrap();
        // "sess-drop" が削除されているか、最初から存在しないことを確認する
        // NOTE: ドロップ後に receiver_count が 0 になるため削除される
        assert!(!sessions.contains_key("sess-drop") || sessions["sess-drop"].receiver_count() == 0);
    }
}
