# system-saga-database 設計

system Tier の Saga オーケストレーターデータベース（saga-db）の設計を定義する。
配置先: `regions/system/database/saga-db/`

## 概要

saga-db は system Tier に属する PostgreSQL 17 データベースであり、分散トランザクション（Saga）の状態管理を担う。Saga Orchestrator サーバーが各ステップの進行状況・補償トランザクションの実行状態・エラー情報を永続化し、サーバー再起動時のリカバリを可能にする。

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

Saga の実行状態を管理する。各 Saga は一意の ID を持ち、ワークフロー名・現在のステップ・全体のステータスを追跡する。

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
- `idx_saga_states_workflow_name` — workflow_name
- `idx_saga_states_status` — status（リカバリ対象の未完了 Saga 検索に使用）
- `idx_saga_states_correlation_id` — correlation_id（WHERE IS NOT NULL、部分インデックス）
- `idx_saga_states_created_at` — created_at

### saga_step_logs テーブル

各 Saga のステップ実行ログを記録する。EXECUTE（正常実行）と COMPENSATE（補償実行）の両方が記録される。

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
- `idx_saga_step_logs_saga_id_step_index` — (saga_id, step_index)（複合インデックス）

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

### 001_create_schema.up.sql

```sql
-- saga-db: スキーマ・拡張機能・共通関数の作成 (PostgreSQL 17)

-- 拡張機能
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- スキーマ
CREATE SCHEMA IF NOT EXISTS saga;

-- updated_at 自動更新関数
CREATE OR REPLACE FUNCTION saga.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

### 001_create_schema.down.sql

```sql
DROP FUNCTION IF EXISTS saga.update_updated_at();
DROP SCHEMA IF EXISTS saga CASCADE;
DROP EXTENSION IF EXISTS "pgcrypto";
```

### 002_create_saga_states.up.sql

```sql
-- saga-db: saga_states テーブル作成

CREATE TABLE IF NOT EXISTS saga.saga_states (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_name   VARCHAR(255) NOT NULL,
    current_step    INT          NOT NULL DEFAULT 0,
    status          VARCHAR(50)  NOT NULL DEFAULT 'STARTED',
    payload         JSONB,
    correlation_id  VARCHAR(255),
    initiated_by    VARCHAR(255),
    error_message   TEXT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_saga_states_status CHECK (status IN ('STARTED', 'RUNNING', 'COMPLETED', 'COMPENSATING', 'FAILED', 'CANCELLED'))
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_saga_states_workflow_name ON saga.saga_states (workflow_name);
CREATE INDEX IF NOT EXISTS idx_saga_states_status ON saga.saga_states (status);
CREATE INDEX IF NOT EXISTS idx_saga_states_correlation_id ON saga.saga_states (correlation_id) WHERE correlation_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_saga_states_created_at ON saga.saga_states (created_at);

-- updated_at トリガー
CREATE TRIGGER update_saga_states_updated_at
    BEFORE UPDATE ON saga.saga_states
    FOR EACH ROW
    EXECUTE FUNCTION saga.update_updated_at();
```

### 002_create_saga_states.down.sql

```sql
DROP TRIGGER IF EXISTS update_saga_states_updated_at ON saga.saga_states;
DROP TABLE IF EXISTS saga.saga_states;
```

### 003_create_saga_step_logs.up.sql

```sql
-- saga-db: saga_step_logs テーブル作成

CREATE TABLE IF NOT EXISTS saga.saga_step_logs (
    id                UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    saga_id           UUID         NOT NULL REFERENCES saga.saga_states(id) ON DELETE CASCADE,
    step_index        INT          NOT NULL,
    step_name         VARCHAR(255) NOT NULL,
    action            VARCHAR(50)  NOT NULL,
    status            VARCHAR(50)  NOT NULL,
    request_payload   JSONB,
    response_payload  JSONB,
    error_message     TEXT,
    started_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    completed_at      TIMESTAMPTZ,

    CONSTRAINT chk_saga_step_logs_action CHECK (action IN ('EXECUTE', 'COMPENSATE')),
    CONSTRAINT chk_saga_step_logs_status CHECK (status IN ('SUCCESS', 'FAILED', 'TIMEOUT', 'SKIPPED'))
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_saga_step_logs_saga_id_step_index ON saga.saga_step_logs (saga_id, step_index);
```

### 003_create_saga_step_logs.down.sql

```sql
DROP TABLE IF EXISTS saga.saga_step_logs;
```

### 004_add_indexes.up.sql

```sql
-- saga-db: saga_states および saga_step_logs への追加インデックス

-- saga_step_logs: ステップ名での検索用
CREATE INDEX IF NOT EXISTS idx_saga_step_logs_step_name
    ON saga.saga_step_logs (step_name);

-- saga_step_logs: ステータスでのフィルタ用
CREATE INDEX IF NOT EXISTS idx_saga_step_logs_status
    ON saga.saga_step_logs (status);

-- saga_step_logs: アクションでのフィルタ用
CREATE INDEX IF NOT EXISTS idx_saga_step_logs_action
    ON saga.saga_step_logs (action);

-- saga_step_logs: 開始時刻での範囲検索用
CREATE INDEX IF NOT EXISTS idx_saga_step_logs_started_at
    ON saga.saga_step_logs (started_at);
```

### 004_add_indexes.down.sql

```sql
DROP INDEX IF EXISTS saga.idx_saga_step_logs_step_name;
DROP INDEX IF EXISTS saga.idx_saga_step_logs_status;
DROP INDEX IF EXISTS saga.idx_saga_step_logs_action;
DROP INDEX IF EXISTS saga.idx_saga_step_logs_started_at;
```

### 005_add_updated_at_trigger.up.sql

```sql
-- saga-db: saga_step_logs の updated_at 関連拡張
-- 注意: saga_states のトリガーは 002_create_saga_states.up.sql で作成済み
--       saga.update_updated_at() 関数は 001_create_schema.up.sql で作成済み

-- saga_step_logs に updated_at カラムを追加
ALTER TABLE saga.saga_step_logs
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- saga_step_logs の updated_at トリガー
CREATE TRIGGER trigger_saga_step_logs_update_updated_at
    BEFORE UPDATE ON saga.saga_step_logs
    FOR EACH ROW
    EXECUTE FUNCTION saga.update_updated_at();
```

### 005_add_updated_at_trigger.down.sql

```sql
DROP TRIGGER IF EXISTS trigger_saga_step_logs_update_updated_at ON saga.saga_step_logs;
ALTER TABLE saga.saga_step_logs DROP COLUMN IF EXISTS updated_at;
```

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

### 設計方針

- **リカバリクエリの最適化**: 起動時リカバリでは `status IN ('STARTED', 'RUNNING', 'COMPENSATING')` の条件で未完了 Saga を検索する。`idx_saga_states_status` インデックスによりフルスキャンを回避する
- **部分インデックス**: `correlation_id` は NULL が多いため部分インデックスを使用し、インデックスサイズを削減する
- **複合インデックス**: ステップログは `saga_id` + `step_index` の複合インデックスにより、特定 Saga のログを順序付きで効率的に取得できる

---

## トランザクション設計

### ステップログの原子的記録

Saga 状態の更新とステップログの挿入は単一のデータベーストランザクションで実行し、状態の一貫性を保証する。

```sql
-- SagaPostgresRepository::update_with_step_log の実装パターン
BEGIN;
  UPDATE saga.saga_states
  SET current_step = $2,
      status = $3,
      error_message = $4,
      updated_at = NOW()
  WHERE id = $1;

  INSERT INTO saga.saga_step_logs (
    id, saga_id, step_index, step_name, action, status,
    request_payload, response_payload, error_message,
    started_at, completed_at
  ) VALUES ($5, $1, $6, $7, $8, $9, $10, $11, $12, $13, $14);
COMMIT;
```

この原子性により、以下を保証する:
- ステップログが記録されているなら、saga_states も一貫した状態にある
- サーバー障害時も中途半端な状態が残らない
- 起動時リカバリで確実に未完了 Saga を検出できる

---

## 主要クエリパターン

### Saga 管理

```sql
-- 新規 Saga の作成
INSERT INTO saga.saga_states (
    id, workflow_name, current_step, status, payload, correlation_id, initiated_by
) VALUES ($1, $2, 0, 'STARTED', $3, $4, $5)
RETURNING *;

-- Saga の状態をステップログと共に更新（トランザクション内）
UPDATE saga.saga_states
SET current_step = $2, status = $3, error_message = $4, updated_at = NOW()
WHERE id = $1;

-- Saga ID による詳細取得
SELECT * FROM saga.saga_states WHERE id = $1;

-- ステップログ一覧取得（ステップ順）
SELECT * FROM saga.saga_step_logs
WHERE saga_id = $1
ORDER BY step_index ASC, started_at ASC;
```

### 起動時リカバリ

```sql
-- 未完了 Saga の検索（起動時リカバリ）
SELECT * FROM saga.saga_states
WHERE status IN ('STARTED', 'RUNNING', 'COMPENSATING')
ORDER BY created_at ASC;
```

### 一覧取得（ページネーション）

```sql
-- ワークフロー名・ステータス・相関 ID でフィルタリング
SELECT * FROM saga.saga_states
WHERE
    ($1::VARCHAR IS NULL OR workflow_name = $1)
    AND ($2::VARCHAR IS NULL OR status = $2)
    AND ($3::VARCHAR IS NULL OR correlation_id = $3)
ORDER BY created_at DESC
LIMIT $4 OFFSET $5;

-- 総件数
SELECT COUNT(*) FROM saga.saga_states
WHERE
    ($1::VARCHAR IS NULL OR workflow_name = $1)
    AND ($2::VARCHAR IS NULL OR status = $2)
    AND ($3::VARCHAR IS NULL OR correlation_id = $3);
```

---

## 接続設定

[config設計](../../cli/config/config設計.md) の database セクションに従い、saga-db への接続を以下のように設定する。

### config.yaml（saga サーバー用）

```yaml
# config/config.yaml — saga サーバー
app:
  name: "saga-server"
  version: "0.1.0"
  tier: "system"
  environment: "dev"

database:
  host: "postgres.k1s0-system.svc.cluster.local"
  port: 5432
  name: "k1s0_system"
  user: "app"
  password: ""                   # Vault パス: secret/data/k1s0/system/saga/database キー: password
  ssl_mode: "disable"            # dev 環境。staging: require、prod: verify-full
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"
```

### 環境別設定

| 環境 | host | ssl_mode | max_open_conns | max_idle_conns |
|------|------|----------|----------------|----------------|
| dev | localhost (docker-compose) | disable | 10 | 3 |
| staging | postgres.k1s0-system.svc.cluster.local | require | 25 | 5 |
| prod | postgres.k1s0-system.svc.cluster.local | verify-full | 50 | 10 |

### Vault によるクレデンシャル管理

[認証認可設計](../../auth/design/認証認可設計.md) D-006 のシークレットパス体系に従い、以下の Vault パスから DB クレデンシャルを取得する。

| 用途 | Vault パス | 説明 |
|------|-----------|------|
| 静的パスワード | `secret/data/k1s0/system/saga/database` | キー: `password` |
| 動的クレデンシャル（読み書き） | `database/creds/system-saga-rw` | Vault Database エンジンで自動生成（TTL: 24時間） |
| 動的クレデンシャル（読み取り専用） | `database/creds/system-saga-ro` | Vault Database エンジンで自動生成（TTL: 24時間） |

### docker-compose（ローカル開発）

[docker-compose設計](../../infrastructure/docker/docker-compose設計.md) の共通 PostgreSQL インスタンスに `k1s0_system` データベースを使用する。saga-db と auth-db は同一の `k1s0_system` データベース内の異なるスキーマ（`saga` / `auth`）として共存する。

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

### バックアップ実行例

```bash
# フルバックアップ（pg_basebackup）
pg_basebackup -h postgres.k1s0-system.svc.cluster.local -U replication -D /backup/base -Ft -z -P

# 論理バックアップ（スキーマ単位）
pg_dump -h postgres.k1s0-system.svc.cluster.local -U app -d k1s0_system \
    -n saga -Fc -f /backup/k1s0_system_saga.dump
```

---

## 関連ドキュメント

- [system-saga-server設計](server設計.md) — Saga Orchestrator サーバー設計（API・アーキテクチャ・実装）
- [tier-architecture](../../architecture/overview/tier-architecture.md) — Tier アーキテクチャ・データベースアクセスルール
- [メッセージング設計](../../architecture/messaging/メッセージング設計.md) — Saga パターンの基本方針
- [config設計](../../cli/config/config設計.md) — config.yaml スキーマ（database セクション）
- [テンプレート仕様-データベース](../../templates/data/データベース.md) — マイグレーション命名規則・テンプレート
- [docker-compose設計](../../infrastructure/docker/docker-compose設計.md) — ローカル開発用 PostgreSQL
- [kubernetes設計](../../infrastructure/kubernetes/kubernetes設計.md) — Namespace・PVC 設計
- [helm設計](../../infrastructure/kubernetes/helm設計.md) — PostgreSQL Helm Chart・Vault Agent Injector
