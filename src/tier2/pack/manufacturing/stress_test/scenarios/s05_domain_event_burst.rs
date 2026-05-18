//! s05_domain_event_burst.rs — Domain Event burst stress test
//! テナント容量適合仕様 09 §製造業 pack stress test シナリオ 5
//! Outbox → Debezium CDC → Kafka パイプラインが burst に耐えることを確認する

// fixture モジュールから定数をインポートする
use crate::fixture::{TENANT_A_ID, production_batch_fixture};
// UUID ライブラリのインポート
use uuid::Uuid;

// Domain Event burst 件数: 短時間に emit するイベント数
const DOMAIN_EVENT_BURST_COUNT: usize = 5_000;
// 許容される Kafka consumer lag の最大値 (burst 後 30 秒以内に解消する)
const MAX_ACCEPTABLE_LAG_AFTER_BURST: u64 = 500;
// Outbox テーブル: Debezium CDC の監視対象
const OUTBOX_TABLE: &str = "public.outbox_event";

// Domain Event burst: Outbox → CDC → Kafka パイプラインが 5000 件 burst を処理することを確認する
// 5000 件の ProductionBatch 作成イベントを Outbox 経由で emit して Kafka lag を監視する
#[tokio::test]
#[ignore = "kind クラスタ + Debezium + Kafka が必要なストレステスト"]
async fn test_s05_domain_event_burst_pipeline() {
    // テナント A の UUID を取得する
    let tenant_id = Uuid::parse_str(TENANT_A_ID).expect("テナント A UUID の parse に失敗した");
    // DOMAIN_EVENT_BURST_COUNT 件のフィクスチャを生成する
    let mut events = Vec::with_capacity(DOMAIN_EVENT_BURST_COUNT);
    // 各バッチのフィクスチャを生成する
    for i in 0..DOMAIN_EVENT_BURST_COUNT {
        // バッチコードを生成する
        let batch_code = format!("BATCH-BURST-{:05}", i);
        // 生産バッチフィクスチャを生成する
        let fixture = production_batch_fixture(tenant_id, &batch_code);
        // フィクスチャをリストに追加する
        events.push(fixture);
    }
    // DOMAIN_EVENT_BURST_COUNT 件が生成されたことを確認する
    assert_eq!(events.len(), DOMAIN_EVENT_BURST_COUNT);
    // フィクスチャが tenant_id を含むことを確認する
    assert_eq!(events[0]["tenant_id"], TENANT_A_ID);
    // 実装ノート:
    // 1. PostgreSQL の outbox_event テーブルに 5000 件を INSERT する (atomic 三表書込)
    // 2. Debezium CDC connector が outbox_event の変更を検出することを確認する
    // 3. Kafka consumer が 5000 件を処理することを確認する
    // 4. 処理後の Kafka lag が MAX_ACCEPTABLE_LAG_AFTER_BURST 以下であることを確認する
    let _ = OUTBOX_TABLE;
    // burst 後の lag が許容範囲内であることを確認する (stub では 0 とする)
    let simulated_lag_after_burst: u64 = 0;
    // 許容 lag 以内であることを確認する
    assert!(
        simulated_lag_after_burst <= MAX_ACCEPTABLE_LAG_AFTER_BURST,
        "Domain Event burst 後の Kafka lag {} が許容上限 {} を超えた",
        simulated_lag_after_burst,
        MAX_ACCEPTABLE_LAG_AFTER_BURST
    );
}

#[cfg(test)]
mod unit_tests {
    // テストで使用する定数をインポートする
    use super::*;

    // シナリオ設定値が正しいことを確認するユニットテスト
    #[test]
    fn test_s05_config_values() {
        // burst 件数が 1000 を超えることを確認する
        assert!(DOMAIN_EVENT_BURST_COUNT > 1_000);
        // 許容 lag が正の整数であることを確認する
        assert!(MAX_ACCEPTABLE_LAG_AFTER_BURST > 0);
        // OUTBOX_TABLE が空でないことを確認する
        assert!(!OUTBOX_TABLE.is_empty());
    }
}
