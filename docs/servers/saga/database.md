# system-saga-database 設計

> **ガイド**: 設計背景・実装例は [database.guide.md](./database.guide.md) を参照。

system Tier の Saga オーケストレーターデータベース（saga-db）の設計を定義する。
配置先: `regions/system/database/saga-db/`

## 概要

saga-db は system Tier に属する PostgreSQL 17 データベースであり、分散トランザクション（Saga）の状態管理を担う。

[tier-architecture.md](../../architecture/overview/tier-architecture.md) の設計原則に従い、saga-db へのアクセスは **system Tier のサーバーからのみ** 許可する。

### 技術スタック

| コンポーネント | 技術 | バージョン |
|----------------|------|------------|
| RDBMS | PostgreSQL | 17 |
| マイグレーション | sqlx-cli | - |
| ORM / クエリビルダー | sqlx | - |
| シークレット管理 | HashiCorp Vault | 1.17 |

---

## ER 図

```
┌──────────────────────────┐
│       saga_states        │
├──────────────────────────┤
│ id (PK)                  │──┐
│ workflow_name            │  │
│ current_step             │  │
│ status                   │  │
│ payload (JSONB)          │  │
│ correlation_id           │  │
│ initiated_by             │  │
│ error_message            │  │
│ created_at               │  │
│ updated_at               │  │
└──────────────────────────┘  │
                               │ 1:N
┌──────────────────────────┐  │
│    saga_step_logs        │  │
├──────────────────────────┤  │
│ id (PK)                  │  │
│ saga_id (FK)             │<─┘
│ step_index               │
│ step_name                │
│ action                   │
│ status                   │
│ request_payload (JSONB)  │
│ response_payload (JSONB) │
│ error_message            │
│ started_at               │
│ completed_at             │
└──────────────────────────┘
```

### リレーション

| 関係 | カーディナリティ | 説明 |
|------|-----------------|------|
| saga_states - saga_step_logs | 1:N | 1 つの Saga は複数のステップログを持つ |

---

## テーブル定義

### saga_states テーブル

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | Saga の一意識別子 |
| workflow_name | VARCHAR(255) | NOT NULL | 実行中のワークフロー名 |
| current_step | INT | NOT NULL DEFAULT 0 | 現在のステップインデックス（0 始まり） |
| status | VARCHAR(50) | NOT NULL DEFAULT 'STARTED', CHECK制約 | Saga ステータス（下記参照） |
| payload | JSONB | | 各ステップに渡す JSON ペイロード |
| correlation_id | VARCHAR(255) | | 業務相関 ID（トレーサビリティ用） |
| initiated_by | VARCHAR(255) | | 呼び出し元サービス名 |
| error_message | TEXT | | 失敗・補償時のエラーメッセージ |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | Saga 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 最終更新日時（トリガーで自動更新） |

**status の有効値（CHECK 制約）:**

| ステータス | 説明 |
|-----------|------|
| `STARTED` | Saga が作成された初期状態 |
| `RUNNING` | ステップが実行中 |
| `COMPLETED` | 全ステップが正常完了（終端状態） |
| `COMPENSATING` | ステップ失敗により補償処理を実行中 |
| `FAILED` | 補償処理完了後の失敗状態（終端状態） |
| `CANCELLED` | ユーザーによるキャンセル（終端状態） |

**インデックス:**
- `idx_saga_states_workflow_name` -- workflow_name
- `idx_saga_states_status` -- status（リカバリ対象の未完了 Saga 検索に使用）
- `idx_saga_states_correlation_id` -- correlation_id（WHERE IS NOT NULL、部分インデックス）
- `idx_saga_states_created_at` -- created_at

### saga_step_logs テーブル

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | ステップログの一意識別子 |
| saga_id | UUID | FK saga_states(id) ON DELETE CASCADE, NOT NULL | 所属する Saga の ID |
| step_index | INT | NOT NULL | ステップインデックス（0 始まり） |
| step_name | VARCHAR(255) | NOT NULL | ステップ名（ワークフロー定義の name フィールド） |
| action | VARCHAR(50) | NOT NULL, CHECK ('EXECUTE', 'COMPENSATE') | 実行アクション種別 |
| status | VARCHAR(50) | NOT NULL, CHECK ('SUCCESS', 'FAILED', 'TIMEOUT', 'SKIPPED') | 実行結果 |
| request_payload | JSONB | | gRPC リクエストに渡したペイロード |
| response_payload | JSONB | | gRPC レスポンスのペイロード |
| error_message | TEXT | | エラー発生時のメッセージ |
| started_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | ステップ開始日時 |
| completed_at | TIMESTAMPTZ | | ステップ完了日時（実行中は NULL） |

**action の有効値（CHECK 制約）:**

| action | 説明 |
|--------|------|
| `EXECUTE` | ワークフローのステップを順方向に実行 |
| `COMPENSATE` | 失敗時の補償処理（ロールバック）を実行 |

**status の有効値（CHECK 制約）:**

| status | 説明 |
|--------|------|
| `SUCCESS` | ステップが正常に完了 |
| `FAILED` | ステップが失敗（リトライ上限到達等） |
| `TIMEOUT` | タイムアウトによる失敗 |
| `SKIPPED` | 補償メソッド未定義等によりスキップ |

**インデックス:**
- `idx_saga_step_logs_saga_id_step_index` -- (saga_id, step_index)（複合インデックス）

---

## マイグレーションファイル

配置先: `regions/system/database/saga-db/migrations/`

命名規則は [テンプレート仕様-データベース](../../templates/data/データベース.md) に準拠する。

```
migrations/
├── 001_create_schema.up.sql                    # スキーマ・拡張機能・共通関数
├── 001_create_schema.down.sql
├── 002_create_saga_states.up.sql               # saga_states テーブル
├── 002_create_saga_states.down.sql
├── 003_create_saga_step_logs.up.sql            # saga_step_logs テーブル
├── 003_create_saga_step_logs.down.sql
├── 004_add_indexes.up.sql                      # saga_step_logs への追加インデックス
├── 004_add_indexes.down.sql
├── 005_add_updated_at_trigger.up.sql           # saga_step_logs に updated_at カラム追加 + トリガー
└── 005_add_updated_at_trigger.down.sql
```

> マイグレーション SQL 全文は [database.guide.md](./database.guide.md#マイグレーション-sql) を参照。

---

## インデックス設計

### インデックス一覧

| テーブル | インデックス名 | カラム | 種別 | 用途 |
|----------|---------------|--------|------|------|
| saga_states | idx_saga_states_workflow_name | workflow_name | B-tree | ワークフロー名によるフィルタリング |
| saga_states | idx_saga_states_status | status | B-tree | ステータスによるフィルタリング（リカバリ対象検索） |
| saga_states | idx_saga_states_correlation_id | correlation_id (WHERE NOT NULL) | B-tree（部分） | 業務相関 ID による Saga 検索 |
| saga_states | idx_saga_states_created_at | created_at | B-tree | 作成日時による範囲検索・ソート |
| saga_step_logs | idx_saga_step_logs_saga_id_step_index | (saga_id, step_index) | B-tree（複合） | Saga に紐づくステップログの順序付き取得 |
| saga_step_logs | idx_saga_step_logs_step_name | step_name | B-tree | ステップ名での検索 |
| saga_step_logs | idx_saga_step_logs_status | status | B-tree | ステータスでのフィルタリング |
| saga_step_logs | idx_saga_step_logs_action | action | B-tree | アクション種別でのフィルタリング |
| saga_step_logs | idx_saga_step_logs_started_at | started_at | B-tree | 開始時刻での範囲検索 |

> インデックス設計方針は [database.guide.md](./database.guide.md#インデックス設計方針) を参照。

---

## 接続設定

[config設計](../../cli/config/config設計.md) の database セクションに従い、saga-db への接続を設定する。

### 環境別設定

| 環境 | host | ssl_mode | max_open_conns | max_idle_conns |
|------|------|----------|----------------|----------------|
| dev | localhost (docker-compose) | disable | 10 | 3 |
| staging | postgres.k1s0-system.svc.cluster.local | require | 25 | 5 |
| prod | postgres.k1s0-system.svc.cluster.local | verify-full | 50 | 10 |

### Vault によるクレデンシャル管理

[認証認可設計](../../architecture/auth/認証認可設計.md) D-006 のシークレットパス体系に従う。

| 用途 | Vault パス | 説明 |
|------|-----------|------|
| 静的パスワード | `secret/data/k1s0/system/saga/database` | キー: `password` |
| 動的クレデンシャル（読み書き） | `database/creds/system-saga-rw` | Vault Database エンジンで自動生成（TTL: 24時間） |
| 動的クレデンシャル（読み取り専用） | `database/creds/system-saga-ro` | Vault Database エンジンで自動生成（TTL: 24時間） |

### docker-compose（ローカル開発）

[docker-compose設計](../../infrastructure/docker/docker-compose設計.md) の共通 PostgreSQL インスタンスに `k1s0_system` データベースを使用する。saga-db と auth-db は同一の `k1s0_system` データベース内の異なるスキーマ（`saga` / `auth`）として共存する。

> config.yaml 例は [database.guide.md](./database.guide.md#接続設定例) を参照。

---

## バックアップ・リストア

### バックアップ方針

| 項目 | 値 |
|------|-----|
| フルバックアップ | 毎日深夜（0:00） |
| WAL アーカイブ | 継続的（PITR 対応） |
| バックアップ先 | Ceph オブジェクトストレージ |
| 保持期間 | フルバックアップ: 30日、WAL: 7日 |
| リストアテスト | 月次で staging 環境にリストアし検証 |

**注意**: saga_states は分散トランザクションの状態管理が目的であり、終端状態（COMPLETED / FAILED / CANCELLED）に達した Saga の長期保存は必要に応じてアーカイブする。

> バックアップ実行例は [database.guide.md](./database.guide.md#バックアップ実行例) を参照。

---

## 関連ドキュメント

- [system-saga-server設計](server.md) -- Saga Orchestrator サーバー設計（API・アーキテクチャ・実装）
- [tier-architecture](../../architecture/overview/tier-architecture.md) -- Tier アーキテクチャ・データベースアクセスルール
- [メッセージング設計](../../architecture/messaging/メッセージング設計.md) -- Saga パターンの基本方針
- [config設計](../../cli/config/config設計.md) -- config.yaml スキーマ（database セクション）
- [テンプレート仕様-データベース](../../templates/data/データベース.md) -- マイグレーション命名規則・テンプレート
- [docker-compose設計](../../infrastructure/docker/docker-compose設計.md) -- ローカル開発用 PostgreSQL
- [kubernetes設計](../../infrastructure/kubernetes/kubernetes設計.md) -- Namespace・PVC 設計
- [helm設計](../../infrastructure/kubernetes/helm設計.md) -- PostgreSQL Helm Chart・Vault Agent Injector
