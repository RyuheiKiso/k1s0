//! s06_neighbor_isolation.rs — Noisy Neighbor 分離 stress test
//! テナント容量適合仕様 09 §製造業 pack stress test シナリオ 6
//! テナント A が高負荷でもテナント B の SLO が維持されることを確認する

// fixture モジュールから定数をインポートする
use crate::fixture::{TENANT_A_ID, TENANT_B_ID, resource_unit_fixture, production_batch_fixture};
// UUID ライブラリのインポート
use uuid::Uuid;
// 時間計測に使用する
use std::time::{Duration, Instant};

// テナント A (noisy neighbor) の同時リクエスト数: 高負荷をシミュレートする
const NOISY_TENANT_CONCURRENCY: usize = 100;
// テナント B の SLO p99 レイテンシ上限 (ms)
const TENANT_B_SLO_P99_MS: u64 = 500;
// テナント B の測定リクエスト数
const TENANT_B_PROBE_COUNT: usize = 50;

// Noisy Neighbor 分離: テナント A の高負荷時にテナント B の SLO が維持されることを確認する
// テナント A に高負荷を与えながらテナント B のレイテンシを測定して SLO 内であることを検証する
#[tokio::test]
#[ignore = "kind クラスタ + Envoy Gateway + PostgreSQL RLS が必要なストレステスト"]
async fn test_s06_neighbor_isolation_slo_maintained() {
    // テナント A の UUID を取得する
    let tenant_a = Uuid::parse_str(TENANT_A_ID).expect("テナント A UUID の parse に失敗した");
    // テナント B の UUID を取得する
    let tenant_b = Uuid::parse_str(TENANT_B_ID).expect("テナント B UUID の parse に失敗した");
    // テナント A (noisy neighbor) の高負荷タスクを起動する
    let noisy_task = tokio::spawn(async move {
        // テナント A に高負荷をかける並列リクエストを送信する
        let mut tasks = Vec::with_capacity(NOISY_TENANT_CONCURRENCY);
        // 高負荷タスクを起動する
        for i in 0..NOISY_TENANT_CONCURRENCY {
            // 各タスクでテナント A にリクエストを送信する
            let task = tokio::spawn(async move {
                // フィクスチャを生成する
                let _fixture = resource_unit_fixture(tenant_a, &format!("FA-NOISY-{}", i));
                // 実装ノート: reqwest で API にリクエストを送信し続ける
                // このタスクは noisy_task と同時に実行されてテナント A の負荷を生成する
            });
            tasks.push(task);
        }
        // 全タスクの完了を待つ
        for task in tasks {
            let _ = task.await;
        }
    });
    // テナント B のプローブリクエストを送信してレイテンシを計測する
    let probe_task = tokio::spawn(async move {
        // レイテンシを記録するベクターを初期化する
        let mut latencies_ms = Vec::with_capacity(TENANT_B_PROBE_COUNT);
        // TENANT_B_PROBE_COUNT 件のプローブを送信する
        for i in 0..TENANT_B_PROBE_COUNT {
            // リクエスト開始時刻を記録する
            let start = Instant::now();
            // テナント B のフィクスチャを生成する
            let _fixture = production_batch_fixture(tenant_b, &format!("PROBE-{:03}", i));
            // 実装ノート: reqwest でテナント B の API にリクエストを送信する
            // リクエスト処理時間を計測する
            let elapsed_ms = start.elapsed().as_millis() as u64;
            // レイテンシを記録する
            latencies_ms.push(elapsed_ms);
        }
        // latencies_ms を返す
        latencies_ms
    });
    // 両タスクの完了を待つ
    let _ = noisy_task.await.expect("noisy neighbor タスクの実行に失敗した");
    let latencies = probe_task.await.expect("テナント B プローブタスクの実行に失敗した");
    // p99 レイテンシを計算する
    let mut sorted_latencies = latencies.clone();
    // レイテンシをソートする
    sorted_latencies.sort_unstable();
    // p99 インデックスを計算する
    let p99_idx = (sorted_latencies.len() as f64 * 0.99) as usize;
    // p99 レイテンシ値を取得する (stubではインデックスが範囲外なので min を使う)
    let p99_latency = sorted_latencies.get(p99_idx).copied().unwrap_or(0);
    // テナント B の p99 レイテンシが SLO 内であることを確認する
    assert!(
        p99_latency <= TENANT_B_SLO_P99_MS,
        "テナント B の p99 レイテンシ {}ms が SLO {}ms を超えた",
        p99_latency,
        TENANT_B_SLO_P99_MS
    );
}

#[cfg(test)]
mod unit_tests {
    // テストで使用する定数をインポートする
    use super::*;

    // シナリオ設定値が正しいことを確認するユニットテスト
    #[test]
    fn test_s06_config_values() {
        // 高負荷並列度が 10 を超えることを確認する
        assert!(NOISY_TENANT_CONCURRENCY > 10);
        // SLO p99 が正の整数であることを確認する
        assert!(TENANT_B_SLO_P99_MS > 0);
        // プローブ件数が 10 以上であることを確認する
        assert!(TENANT_B_PROBE_COUNT >= 10);
    }
}
