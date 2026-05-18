// tier2 CQRS 読み取りモデル投影器（設計方針 15 / ReadModelProjector トレイト定義）
// Outbox リレー経由で受信したドメインイベントを読み取りモデルに投影する責務を持つ

// anyhow: Result 型に使用する
use anyhow::Result;
// serde: ドメインイベントのシリアライズ / デシリアライズに使用する
use serde::{Deserialize, Serialize};
// uuid: テナント識別子型に使用する
use uuid::Uuid;

// ドメインイベントの構造体定義（Outbox から受信する共通エンベロープ形式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    // イベント一意識別子（UUID v4）
    pub id: Uuid,
    // テナント識別子（RLS 述語の基底 / AuthContext からのみ注入する）
    pub tenant_id: Uuid,
    // 集約型名（業界中立語のみ使用可）
    pub aggregate_type: String,
    // イベント種別名
    pub event_type: String,
    // ペイロード（JSON Value 形式 / PII フィールドは Outbox 書込前に redact 済み）
    pub payload: serde_json::Value,
    // HLC タイムスタンプ（wall clock TTL 禁止規約により HLC を使用する）
    pub hlc_timestamp: u64,
}

// ReadModelProjector トレイト（すべての読み取りモデル投影器が実装する契約）
// project メソッドはドメインイベントを受け取り読み取りモデルを更新する
pub trait ReadModelProjector: Send + Sync {
    // ドメインイベントを受け取り読み取りモデルを更新する
    // 失敗した場合は anyhow::Error を返す（Outbox リレーがリトライする）
    fn project(&self, event: &DomainEvent) -> Result<()>;

    // 投影器が処理対象とするイベント種別の一覧を返す
    // 登録済み投影器のルーティングに使用する
    fn handled_event_types(&self) -> &[&str];
}

// ReadModelRegistry: 複数の ReadModelProjector を集約するレジストリ
// イベント種別に基づいて対応する投影器へルーティングする
pub struct ReadModelRegistry {
    // 登録済み投影器のリスト（Box<dyn ReadModelProjector> 形式で保持する）
    projectors: Vec<Box<dyn ReadModelProjector>>,
}

impl ReadModelRegistry {
    // 新規レジストリを生成する（空状態から開始する）
    pub fn new() -> Self {
        // 空のレジストリを初期化する
        Self {
            projectors: Vec::new(),
        }
    }

    // 投影器をレジストリに登録する
    pub fn register(&mut self, projector: Box<dyn ReadModelProjector>) {
        // 投影器リストに追加する
        self.projectors.push(projector);
    }

    // ドメインイベントを対応する投影器へルーティングして投影する
    pub fn dispatch(&self, event: &DomainEvent) -> Result<()> {
        // 登録済み投影器を順に検索して対象イベント種別を処理する
        for projector in &self.projectors {
            // 投影器がこのイベント種別を処理するか確認する
            if projector.handled_event_types().contains(&event.event_type.as_str()) {
                // 対応投影器に処理を委譲する
                projector.project(event)?;
            }
        }
        // 全投影器の処理成功
        Ok(())
    }
}

// Default 実装（new() と同義）
impl Default for ReadModelRegistry {
    // デフォルト値として空レジストリを返す
    fn default() -> Self {
        // new() を委譲する
        Self::new()
    }
}
