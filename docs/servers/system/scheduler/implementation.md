# system-scheduler-server 実装設計

> **注記**: 本ドキュメントは scheduler-server の実装仕様を含む。共通パターンは [Rust共通実装.md](../../_common/Rust共通実装.md) を参照。

system-scheduler-server（スケジューラーサーバー）の Rust 実装仕様。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## アーキテクチャ概要

Clean Architecture に基づく 4 層構成を採用する。

| レイヤー | 責務 | 依存方向 |
|---------|------|---------|
| domain | エンティティ・リポジトリトレイト・ドメインサービス | なし（最内層） |
| usecase | ビジネスロジック（ジョブ CRUD・トリガー・一時停止/再開・実行履歴） | domain のみ |
| adapter | REST/gRPC ハンドラー・ミドルウェア・リポジトリ実装 | usecase, domain |
| infrastructure | 設定・DB接続・cron エンジン・Kafka・分散ロック・起動シーケンス | 全レイヤー |

---

## Rust 実装 (regions/system/server/rust/scheduler/)

### ディレクトリ構成

```
regions/system/server/rust/scheduler/
├── src/
│   ├── main.rs                                          # エントリポイント
│   ├── lib.rs                                           # ライブラリルート
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── scheduler_job.rs                         # SchedulerJob エンティティ
│   │   │   └── scheduler_execution.rs                   # SchedulerExecution エンティティ
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   ├── scheduler_job_repository.rs              # SchedulerJobRepository トレイト
│   │   │   └── scheduler_execution_repository.rs        # SchedulerExecutionRepository トレイト
│   │   └── service/
│   │       ├── mod.rs
│   │       └── scheduler_domain_service.rs              # cron 式解析・次回実行時刻計算・分散ロック判定
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── create_job.rs                                # ジョブ作成
│   │   ├── update_job.rs                                # ジョブ更新
│   │   ├── delete_job.rs                                # ジョブ削除
│   │   ├── get_job.rs                                   # ジョブ取得
│   │   ├── list_jobs.rs                                 # ジョブ一覧
│   │   ├── trigger_job.rs                               # ジョブ手動トリガー
│   │   ├── pause_job.rs                                 # ジョブ一時停止
│   │   ├── resume_job.rs                                # ジョブ再開
│   │   └── list_executions.rs                           # 実行履歴一覧
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── job_handler.rs                           # axum REST ハンドラー
│   │   │   └── health.rs                                # ヘルスチェック
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── scheduler_grpc.rs                        # gRPC サービス実装
│   │   │   └── tonic_service.rs                         # tonic サービスラッパー
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs                                  # JWT 認証ミドルウェア
│   │   │   ├── rbac.rs                                  # RBAC ミドルウェア
│   │   │   └── grpc_auth.rs                             # gRPC 認証インターセプター
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── scheduler_job_postgres.rs                # SchedulerJobRepository PostgreSQL 実装
│   │       └── scheduler_execution_postgres.rs          # SchedulerExecutionRepository PostgreSQL 実装
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── config.rs                                    # 設定構造体・読み込み
│   │   ├── database.rs                                  # DB 接続プール
│   │   ├── cache.rs                                     # moka キャッシュ
│   │   ├── cron_engine.rs                               # tokio による cron スケジューリングループ
│   │   ├── job_executor.rs                              # ジョブ実行器（HTTP/Kafka ディスパッチ）
│   │   ├── kafka_producer.rs                            # Kafka プロデューサー（ジョブトリガー通知）
│   │   └── startup.rs                                   # 起動シーケンス・DI
│   └── proto/                                           # tonic-build 生成コード
│       ├── mod.rs
│       ├── k1s0.system.scheduler.v1.rs
│       └── k1s0.system.common.v1.rs
├── tests/
│   ├── integration_test.rs                              # 統合テスト
│   └── usecase_test.rs                                  # ユースケーステスト
├── config/
│   └── config.yaml
├── build.rs
├── Cargo.toml
└── Dockerfile
```

### 主要コンポーネント

#### ドメインサービス

- **SchedulerDomainService**: cron 式をパースし、タイムゾーンを考慮した次回実行時刻を計算する。DST（夏時間）遷移時の補正ロジックを含む。分散ロック取得判定も担当する

#### ユースケース

| ユースケース | 責務 |
|------------|------|
| `CreateJobUseCase` | ジョブ作成・cron 式バリデーション・次回実行時刻計算 |
| `UpdateJobUseCase` | ジョブ更新・次回実行時刻再計算 |
| `DeleteJobUseCase` | ジョブ削除（実行中ジョブの保護） |
| `GetJobUseCase` / `ListJobsUseCase` | ジョブ取得・一覧 |
| `TriggerJobUseCase` | 手動トリガー・分散ロック取得・実行記録作成 |
| `PauseJobUseCase` / `ResumeJobUseCase` | ジョブ一時停止・再開 |
| `ListExecutionsUseCase` | 実行履歴一覧（ステータス・期間フィルタ） |

#### 外部連携

- **CronSchedulerEngine** (`infrastructure/cron_engine.rs`): tokio による非同期 cron スケジューリングループ。起動時に全有効ジョブをロードしタイマーを設定する
- **JobExecutor** (`infrastructure/job_executor.rs`): ジョブ実行時に Kafka トピック発行または HTTP POST を実行するディスパッチャー
- **Kafka Producer** (`infrastructure/kafka_producer.rs`): `k1s0.system.scheduler.triggered.v1` トピックにトリガー通知を配信する
- **PostgreSQL 分散ロック**: `SELECT FOR UPDATE SKIP LOCKED` による重複実行防止

### エラーハンドリング方針

- ユースケース層で `anyhow::Result` を返却し、adapter 層で HTTP/gRPC ステータスコードに変換する
- エラーコードプレフィックス: `SYS_SCHED_`
- cron 式不正時は `SYS_SCHED_INVALID_CRON`、タイムゾーン不正時は `SYS_SCHED_INVALID_TIMEZONE` を返却する
- 分散ロック取得失敗時はリトライせず `SYS_SCHED_JOB_RUNNING` を返却する

### テスト方針

| テスト種別 | 対象 | 方針 |
|-----------|------|------|
| 単体テスト | cron 式解析・次回実行時刻計算 | mockall によるリポジトリモック |
| ユースケーステスト | ジョブ CRUD・トリガー・一時停止/再開 | `usecase_test.rs` でモックリポジトリを使用 |
| 統合テスト | REST/gRPC ハンドラー | `integration_test.rs` で axum-test / tonic テストクライアント |
| 分散ロックテスト | 重複実行防止 | テスト用 PostgreSQL で並行トリガーを検証 |

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [Rust共通実装.md](../../_common/Rust共通実装.md) -- 共通起動シーケンス・Cargo 依存
