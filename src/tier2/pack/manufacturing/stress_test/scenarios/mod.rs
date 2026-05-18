//! mod.rs — 製造業 pack stress test シナリオモジュール宣言
//! テナント容量適合仕様 09 §製造業 pack stress test シナリオ 1-8 を宣言する

// シナリオ 1: FA 操作 burst (v1_per_tenant_qps で 429 が返ることを確認する)
pub mod s01_fa_operation_burst;
// シナリオ 2: 複数テナント同時アクセス (クロステナント汚染がないことを確認する)
pub mod s02_concurrent_tenant_access;
// シナリオ 3: ボリューム上限超過 (Kafka quota でスロットリングされることを確認する)
pub mod s03_volume_overflow;
// シナリオ 4: ML/AI 推論 load shedding (高負荷時に 503 が返ることを確認する)
pub mod s04_ml_load_shed;
// シナリオ 5: Domain Event burst (Outbox → CDC → Kafka パイプラインを検証する)
pub mod s05_domain_event_burst;
// シナリオ 6: Noisy Neighbor 分離 (テナント A 高負荷時にテナント B の SLO を確認する)
pub mod s06_neighbor_isolation;
// シナリオ 7: ワークフローキュー飽和 (全 workflow が完了することを確認する)
pub mod s07_workflow_queue_saturation;
// シナリオ 8: ClickHouse スロット圧力 (高スループット書込が正常に完了することを確認する)
pub mod s08_clickhouse_slot_pressure;
