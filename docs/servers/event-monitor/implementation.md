# system-event-monitor-server 実装設計

> **注記**: 本ドキュメントは event-monitor-server の実装仕様を含む。共通パターンは [Rust共通実装.md](../_common/Rust共通実装.md) を参照。

system-event-monitor-server（イベントモニタリングサーバー）の Rust 実装仕様。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## アーキテクチャ概要

Clean Architecture に基づく 4 層構成を採用する。

| レイヤー | 責務 | 依存方向 |
|---------|------|---------|
| domain | エンティティ・リポジトリトレイト・ドメインサービス | なし（最内層） |
| usecase | ビジネスロジック（フロー管理・KPI集計・SLO監視・リプレイ） | domain のみ |
| adapter | REST/gRPC ハンドラー・ミドルウェア・リポジトリ実装 | usecase, domain |
| infrastructure | 設定・DB接続・Kafkaコンシューマー・キャッシュ・起動シーケンス | 全レイヤー |

---

## Rust 実装 (regions/system/server/rust/event-monitor/)

### ディレクトリ構成

```
regions/system/server/rust/event-monitor/
├── src/
│   ├── main.rs                              # エントリポイント（startup::run() 委譲）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── event_record.rs              # EventRecord エンティティ（イベントメタデータ）
│   │   │   ├── flow_definition.rs           # FlowDefinition エンティティ（業務フロー定義）
│   │   │   ├── flow_instance.rs             # FlowInstance エンティティ（フローインスタンス）
│   │   │   └── flow_kpi.rs                  # FlowKpi エンティティ（KPI集計結果）
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   ├── event_record_repository.rs   # EventRecordRepository トレイト
│   │   │   ├── flow_definition_repository.rs # FlowDefinitionRepository トレイト
│   │   │   └── flow_instance_repository.rs  # FlowInstanceRepository トレイト
│   │   └── service/
│   │       ├── mod.rs
│   │       ├── flow_matching.rs             # イベント→フロー定義マッチング
│   │       ├── kpi_aggregation.rs           # KPI集計（完了率・平均処理時間・ボトルネック検出）
│   │       ├── slo_calculation.rs           # SLOバーンレート計算・違反検出
│   │       └── timeout_detection.rs         # タイムアウト検出
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── create_flow.rs                   # フロー定義作成
│   │   ├── update_flow.rs                   # フロー定義更新
│   │   ├── delete_flow.rs                   # フロー定義削除
│   │   ├── get_flow.rs                      # フロー定義取得
│   │   ├── list_flows.rs                    # フロー定義一覧
│   │   ├── get_flow_instance.rs             # フローインスタンス取得
│   │   ├── get_flow_instances.rs            # フローインスタンス一覧
│   │   ├── get_flow_kpi.rs                  # フロー別KPI取得
│   │   ├── get_kpi_summary.rs               # KPIサマリー取得
│   │   ├── get_slo_status.rs                # SLOステータス取得
│   │   ├── get_slo_burn_rate.rs             # SLOバーンレート取得
│   │   ├── list_events.rs                   # イベント一覧取得
│   │   ├── trace_by_correlation.rs          # correlation-id ベーストレース
│   │   ├── preview_replay.rs               # リプレイ影響範囲プレビュー
│   │   └── execute_replay.rs               # 選択的リプレイ実行
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── event_monitor_handler.rs     # axum REST ハンドラー
│   │   │   └── health.rs                    # ヘルスチェック
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── event_monitor_grpc.rs        # gRPC サービス実装
│   │   │   └── tonic_service.rs             # tonic サービスラッパー
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs                      # JWT 認証ミドルウェア
│   │   │   └── rbac.rs                      # RBAC ミドルウェア
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── event_record_postgres.rs     # EventRecordRepository PostgreSQL 実装
│   │       ├── flow_definition_postgres.rs  # FlowDefinitionRepository PostgreSQL 実装
│   │       └── flow_instance_postgres.rs    # FlowInstanceRepository PostgreSQL 実装
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── config.rs                        # 設定構造体・読み込み
│   │   ├── database.rs                      # DB 接続プール
│   │   ├── cache.rs                         # moka キャッシュ（KPI集計結果）
│   │   ├── kafka_consumer.rs                # Kafka コンシューマー（全ドメインイベント購読）
│   │   ├── dlq_client.rs                    # dlq-manager クライアント（リプレイ連携）
│   │   └── startup.rs                       # 起動シーケンス・DI
│   └── proto/                               # tonic-build 生成コード
├── config/
│   └── config.yaml
├── build.rs
├── Cargo.toml
└── Dockerfile
```

### 主要コンポーネント

#### ドメインサービス

- **FlowMatching**: Kafka から受信したイベントを業務フロー定義にマッチングし、フローインスタンスの状態を更新する
- **KpiAggregation**: フロー別の完了率・平均処理時間・ボトルネック検出をリアルタイム集計する（PostgreSQL ウィンドウ関数 + インメモリ集計）
- **SloCalculation**: フロー別の SLO 定義に基づくバーンレート計算と違反早期検出
- **TimeoutDetection**: フロー内ステップのタイムアウト検出

#### ユースケース

| ユースケース | 責務 |
|------------|------|
| `CreateFlowUseCase` / `UpdateFlowUseCase` / `DeleteFlowUseCase` | フロー定義の CRUD |
| `GetFlowKpiUseCase` / `GetKpiSummaryUseCase` | KPI 集計結果の取得 |
| `GetSloStatusUseCase` / `GetSloBurnRateUseCase` | SLO ステータス・バーンレート取得 |
| `TraceByCorrelationUseCase` | correlation-id ベースの業務トランザクション追跡 |
| `ExecuteReplayUseCase` | dlq-manager 連携による障害イベントの選択的リプレイ |

#### 外部連携

- **Kafka Consumer** (`infrastructure/kafka_consumer.rs`): `k1s0.*.*.*.v1` パターンの全ドメインイベントを購読し、メタデータを抽出して DB に永続化する
- **DLQ Client** (`infrastructure/dlq_client.rs`): dlq-manager と連携した障害イベントの選択的リプレイ

### エラーハンドリング方針

- ユースケース層で `anyhow::Result` を返却し、adapter 層で HTTP/gRPC ステータスコードに変換する
- エラーコードプレフィックス: `SYS_EVMON_`
- Kafka コンシューマーエラーはログ出力後に継続する（at-least-once 保証）

### テスト方針

| テスト種別 | 対象 | 方針 |
|-----------|------|------|
| 単体テスト | フローマッチング・KPI集計・SLO計算 | mockall によるリポジトリモック |
| 統合テスト | REST/gRPC ハンドラー | axum-test / tonic テストクライアント |
| Kafka テスト | コンシューマー | テスト用 Kafka ブローカーまたはモック |

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義・フロー定義形式
- [Rust共通実装.md](../_common/Rust共通実装.md) -- 共通起動シーケンス・Cargo 依存
