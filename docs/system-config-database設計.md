# system-config-database設計

system Tier の設定管理データベース（config-db）の設計を定義する。
配置先: `regions/system/database/config-db/`

## 概要

config-db は system Tier に属する PostgreSQL 17 データベースであり、アプリケーション全体の動的設定データを管理する。各サービスの設定値を一元管理し、設定変更の監査ログを保持する。

[tier-architecture.md](tier-architecture.md) の設計原則に従い、config-db へのアクセスは **system Tier のサーバーからのみ** 許可する。下位 Tier（business / service）がサービス設定を必要とする場合は、config-server が提供する gRPC API を経由する。

### 技術スタック

| コンポーネント | 技術 | バージョン |
|----------------|------|------------|
| RDBMS | PostgreSQL | 17 |
| マイグレーション | sqlx-cli | - |
| ORM / クエリビルダー | sqlx（Rust） | - |
| シークレット管理 | HashiCorp Vault | 1.17 |

---

## ER図

```
┌─────────────────────┐
│   config_entries     │
├─────────────────────┤
│ id (PK)             │
│ namespace            │──┐
│ key                  │  │
│ value_json (JSONB)   │  │
│ version              │  │    ┌──────────────────────────┐
│ description          │  │    │  config_change_logs       │
│ created_by           │  │    ├──────────────────────────┤
│ updated_by           │  ├───>│ id (PK)                  │
│ created_at           │  │    │ config_entry_id (FK)     │
│ updated_at           │  │    │ namespace                │
└─────────────────────┘  │    │ key                      │
         │                │    │ change_type              │
         │                │    │ old_value_json (JSONB)   │
         │                │    │ new_value_json (JSONB)   │
         │                │    │ changed_by               │
         │                │    │ trace_id                 │
         │                │    │ created_at               │
         │                │    └──────────────────────────┘
         │
         │    ┌─────────────────────────────┐
         └───>│  service_config_mappings     │
              ├─────────────────────────────┤
              │ id (PK)                     │
              │ service_name                │
              │ config_entry_id (FK)        │
              │ created_at                  │
              └─────────────────────────────┘

┌──────────────────────────┐
│     config_schemas       │
├──────────────────────────┤
│ id (PK)                  │
│ service_name (UNIQUE)    │
│ namespace_prefix         │
│ schema_json (JSONB)      │
│ updated_by               │
│ created_at               │
│ updated_at               │
└──────────────────────────┘
```

### リレーション

| 関係 | カーディナリティ | 説明 |
|------|-----------------|------|
| config_entries - config_change_logs | 1:N | 設定エントリは複数の変更ログを持つ |
| config_entries - service_config_mappings | 1:N | 設定エントリは複数のサービスにマッピングされる |
| config_schemas（独立） | -- | サービス名をキーに設定エディタのスキーマ定義を管理（config_entries との直接 FK なし） |

---

## テーブル定義

### config_entries テーブル

設定値を namespace + key の組み合わせで一意に管理する。value_json は JSONB 型とし、スカラー値（文字列、数値）から複雑なオブジェクトまで柔軟に格納する。楽観的ロック用の version カラムを持ち、同時更新を検知する。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | 設定エントリ識別子 |
| namespace | VARCHAR(255) | NOT NULL | 設定の名前空間（例: system.auth.database） |
| key | VARCHAR(255) | NOT NULL | 設定キー（例: host, port） |
| value_json | JSONB | NOT NULL DEFAULT '{}' | 設定値（JSON 形式） |
| version | INT | NOT NULL DEFAULT 1 | 楽観的ロック用バージョン番号 |
| description | TEXT | | 設定の説明 |
| created_by | VARCHAR(255) | NOT NULL | 作成者（ユーザー名またはシステム名） |
| updated_by | VARCHAR(255) | NOT NULL | 最終更新者 |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 更新日時（トリガーで自動更新） |
| | | UNIQUE(namespace, key) | 名前空間とキーの組み合わせで一意 |

### config_change_logs テーブル

設定値の変更履歴を記録する監査ログテーブル。変更前後の値を JSONB で保持し、OpenTelemetry の trace_id と連携する。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | ログ識別子 |
| config_entry_id | UUID | FK config_entries(id) ON DELETE SET NULL | 対象設定エントリの ID |
| namespace | VARCHAR(255) | NOT NULL | 設定の名前空間（削除後の参照用に非正規化） |
| key | VARCHAR(255) | NOT NULL | 設定キー（削除後の参照用に非正規化） |
| change_type | VARCHAR(20) | NOT NULL | 変更種別（CREATED / UPDATED / DELETED） |
| old_value_json | JSONB | | 変更前の値（CREATED 時は NULL） |
| new_value_json | JSONB | | 変更後の値（DELETED 時は NULL） |
| changed_by | VARCHAR(255) | NOT NULL | 変更者 |
| trace_id | VARCHAR(64) | | OpenTelemetry トレース ID |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 記録日時 |

### service_config_mappings テーブル

サービスと設定エントリの関連付けを管理する中間テーブル。サービス名をキーにそのサービスが参照する設定エントリ一覧を取得可能にする。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | マッピング識別子 |
| service_name | VARCHAR(255) | NOT NULL | サービス名（例: auth-server, config-server） |
| config_entry_id | UUID | FK config_entries(id) ON DELETE CASCADE, NOT NULL | 設定エントリ ID |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |
| | | UNIQUE(service_name, config_entry_id) | 同一サービスに同一設定は1回のみ |

### config_schemas テーブル

サービスごとの設定エディタスキーマ定義を管理する。クライアント（Flutter / React）の ConfigInterpreter が `GET /api/v1/config-schema/:service` で取得するスキーマの永続化先。schema_json にはカテゴリ・フィールド定義が JSONB で格納される。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | スキーマ識別子 |
| service_name | VARCHAR(255) | UNIQUE NOT NULL | サービス名（例: auth-server） |
| namespace_prefix | VARCHAR(255) | NOT NULL | 設定値の名前空間プレフィックス |
| schema_json | JSONB | NOT NULL | スキーマ定義（categories, fields 等） |
| updated_by | VARCHAR(255) | NOT NULL DEFAULT 'system' | 最終更新者 |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 更新日時（トリガーで自動更新） |

---

## インデックス設計

### インデックス一覧

| テーブル | インデックス名 | カラム | 種別 | 用途 |
|----------|---------------|--------|------|------|
| config_entries | idx_config_entries_namespace | namespace | B-tree | 名前空間による設定検索 |
| config_entries | idx_config_entries_namespace_key | (namespace, key) | B-tree | 名前空間+キーによる設定取得（UNIQUE 制約とは別） |
| config_entries | idx_config_entries_created_at | created_at | B-tree | 作成日時による範囲検索 |
| config_change_logs | idx_config_change_logs_config_entry_id | config_entry_id | B-tree | 設定エントリ別の変更履歴取得 |
| config_change_logs | idx_config_change_logs_namespace_key | (namespace, key) | B-tree | 名前空間+キーによる変更履歴検索 |
| config_change_logs | idx_config_change_logs_change_type_created_at | (change_type, created_at) | B-tree | 変更種別別の時系列検索 |
| config_change_logs | idx_config_change_logs_trace_id | trace_id (WHERE NOT NULL) | B-tree (部分) | OpenTelemetry トレース ID による検索 |
| service_config_mappings | idx_service_config_mappings_service_name | service_name | B-tree | サービス名による設定マッピング取得 |
| service_config_mappings | idx_service_config_mappings_config_entry_id | config_entry_id | B-tree | 設定エントリに紐づくサービス取得 |
| config_schemas | idx_config_schemas_service_name | service_name | B-tree | サービス名によるスキーマ取得（UNIQUE 制約とは別にパフォーマンス用） |

### 設計方針

- **UNIQUE 制約**: namespace + key の複合ユニーク制約により、同一名前空間に同一キーの重複を防止する
- **部分インデックス**: trace_id は NULL が多いため部分インデックスを使用しインデックスサイズを削減する
- **非正規化**: config_change_logs の namespace / key は config_entries の削除後も参照可能にするため非正規化している

---

## データフロー

```
┌──────────────┐    gRPC     ┌───────────────┐    SQL     ┌──────────────┐
│  auth-server │───────────>│ config-server  │─────────>│   config-db  │
│  saga-server │            │               │          │              │
│  dlq-manager │            │  (設定取得)     │          │ config_entries│
│  bff-proxy   │            │  (設定更新)     │          │ change_logs  │
└──────────────┘            │  (変更通知)     │          │ mappings     │
                            └───────────────┘          └──────────────┘
                                    │
                                    │ Kafka
                                    ▼
                            ┌───────────────┐
                            │ config-change  │
                            │   topic        │
                            │ (変更イベント)  │
                            └───────────────┘
```

### フロー詳細

1. **設定取得**: サービスが config-server の gRPC API を呼び出し、namespace + key で設定値を取得する
2. **設定更新**: 管理者が config-server の gRPC API を通じて設定値を更新する。楽観的ロック（version）で同時更新を検知する
3. **変更記録**: 設定更新時に config_change_logs に変更前後の値を記録する
4. **変更通知**: 設定変更を Kafka の config-change トピックに発行し、関連サービスがリアルタイムで設定変更を検知する

---

## 主要クエリパターン

### 設定取得

```sql
-- 名前空間+キーで設定値を取得
SELECT id, namespace, key, value_json, version
FROM config.config_entries
WHERE namespace = $1 AND key = $2;

-- 名前空間配下の全設定を取得
SELECT id, namespace, key, value_json, version
FROM config.config_entries
WHERE namespace = $1
ORDER BY key;
```

### サービス別設定取得

```sql
-- サービスに紐づく全設定を取得
SELECT ce.namespace, ce.key, ce.value_json, ce.version
FROM config.config_entries ce
INNER JOIN config.service_config_mappings scm ON scm.config_entry_id = ce.id
WHERE scm.service_name = $1
ORDER BY ce.namespace, ce.key;
```

### 設定更新（楽観的ロック）

```sql
-- 楽観的ロック付き更新
UPDATE config.config_entries
SET value_json = $1, version = version + 1, updated_by = $2
WHERE id = $3 AND version = $4;
```

### 変更履歴取得

```sql
-- 設定エントリの変更履歴
SELECT id, change_type, old_value_json, new_value_json, changed_by, created_at
FROM config.config_change_logs
WHERE config_entry_id = $1
ORDER BY created_at DESC
LIMIT $2 OFFSET $3;

-- トレース ID による検索
SELECT id, namespace, key, change_type, changed_by, created_at
FROM config.config_change_logs
WHERE trace_id = $1;
```

---

## マイグレーションファイル

配置先: `regions/system/database/config-db/migrations/`

命名規則は [テンプレート仕様-データベース](テンプレート仕様-データベース.md) に準拠する。

```
migrations/
├── 001_create_schema.up.sql                    # スキーマ・拡張機能・共通関数
├── 001_create_schema.down.sql
├── 002_create_config_entries.up.sql             # config_entries テーブル・インデックス・トリガー
├── 002_create_config_entries.down.sql
├── 003_create_config_change_logs.up.sql         # config_change_logs テーブル・インデックス
├── 003_create_config_change_logs.down.sql
├── 004_seed_initial_data.up.sql                 # 初期データ投入（system Tier サービス設定値）
├── 004_seed_initial_data.down.sql
├── 005_create_service_config_mappings.up.sql    # service_config_mappings テーブル・インデックス
├── 005_create_service_config_mappings.down.sql
├── 006_create_config_schemas.up.sql             # config_schemas テーブル・インデックス・トリガー
└── 006_create_config_schemas.down.sql
```

### マイグレーション一覧

| 番号 | ファイル名 | 説明 |
|------|-----------|------|
| 001 | create_schema | pgcrypto 拡張・config スキーマ・`update_updated_at()` 関数の作成 |
| 002 | create_config_entries | config_entries テーブル・インデックス・updated_at トリガー |
| 003 | create_config_change_logs | config_change_logs テーブル・インデックス・change_type 制約 |
| 004 | seed_initial_data | system Tier サービスの初期設定値投入（auth / config の DB・サーバー設定） |
| 005 | create_service_config_mappings | service_config_mappings テーブル・インデックス・ユニーク制約 |
| 006 | create_config_schemas | config_schemas テーブル・インデックス・updated_at トリガー |

### 001_create_schema.up.sql

```sql
-- config-db: スキーマ・拡張機能・共通関数の作成 (PostgreSQL 17)

CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE SCHEMA IF NOT EXISTS config;

CREATE OR REPLACE FUNCTION config.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

### 002_create_config_entries.up.sql

```sql
-- config-db: config_entries テーブル作成

CREATE TABLE IF NOT EXISTS config.config_entries (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    namespace   VARCHAR(255) NOT NULL,
    key         VARCHAR(255) NOT NULL,
    value_json  JSONB        NOT NULL DEFAULT '{}',
    version     INT          NOT NULL DEFAULT 1,
    description TEXT,
    created_by  VARCHAR(255) NOT NULL,
    updated_by  VARCHAR(255) NOT NULL,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_config_entries_namespace_key UNIQUE (namespace, key)
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_config_entries_namespace
    ON config.config_entries (namespace);
CREATE INDEX IF NOT EXISTS idx_config_entries_namespace_key
    ON config.config_entries (namespace, key);
CREATE INDEX IF NOT EXISTS idx_config_entries_created_at
    ON config.config_entries (created_at);

-- updated_at トリガー
CREATE TRIGGER trigger_config_entries_update_updated_at
    BEFORE UPDATE ON config.config_entries
    FOR EACH ROW
    EXECUTE FUNCTION config.update_updated_at();
```

### 003_create_config_change_logs.up.sql

```sql
-- config-db: config_change_logs テーブル作成

CREATE TABLE IF NOT EXISTS config.config_change_logs (
    id               UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    config_entry_id  UUID         REFERENCES config.config_entries(id) ON DELETE SET NULL,
    namespace        VARCHAR(255) NOT NULL,
    key              VARCHAR(255) NOT NULL,
    change_type      VARCHAR(20)  NOT NULL,
    old_value_json   JSONB,
    new_value_json   JSONB,
    changed_by       VARCHAR(255) NOT NULL,
    trace_id         VARCHAR(64),
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_config_change_logs_change_type
        CHECK (change_type IN ('CREATED', 'UPDATED', 'DELETED'))
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_config_change_logs_config_entry_id
    ON config.config_change_logs (config_entry_id);
CREATE INDEX IF NOT EXISTS idx_config_change_logs_namespace_key
    ON config.config_change_logs (namespace, key);
CREATE INDEX IF NOT EXISTS idx_config_change_logs_change_type_created_at
    ON config.config_change_logs (change_type, created_at);
CREATE INDEX IF NOT EXISTS idx_config_change_logs_trace_id
    ON config.config_change_logs (trace_id)
    WHERE trace_id IS NOT NULL;
```

### 004_seed_initial_data.up.sql

初期データ投入。system Tier サービス（auth / config）の DB 接続設定とサーバー設定を投入する。

```sql
-- config-db: 初期データ投入（system Tier サービスの設定値）

-- system.auth.database -- 認証サーバー DB 接続設定
INSERT INTO config.config_entries (namespace, key, value_json, description, created_by, updated_by) VALUES
    ('system.auth.database', 'host',     '"localhost"',           'DB ホスト名',          'migration', 'migration'),
    ('system.auth.database', 'port',     '5432',                 'DB ポート番号',        'migration', 'migration'),
    ('system.auth.database', 'name',     '"k1s0_system"',        'DB 名',               'migration', 'migration'),
    ('system.auth.database', 'ssl_mode', '"disable"',            'SSL モード',           'migration', 'migration')
ON CONFLICT (namespace, key) DO NOTHING;

-- system.auth.server -- 認証サーバー設定
INSERT INTO config.config_entries (namespace, key, value_json, description, created_by, updated_by) VALUES
    ('system.auth.server', 'port',          '8081',   'gRPC リッスンポート',     'migration', 'migration'),
    ('system.auth.server', 'read_timeout',  '30',     '読み取りタイムアウト（秒）', 'migration', 'migration'),
    ('system.auth.server', 'write_timeout', '30',     '書き込みタイムアウト（秒）', 'migration', 'migration')
ON CONFLICT (namespace, key) DO NOTHING;

-- system.config.database -- 設定サーバー DB 接続設定
INSERT INTO config.config_entries (namespace, key, value_json, description, created_by, updated_by) VALUES
    ('system.config.database', 'host',     '"localhost"',           'DB ホスト名',          'migration', 'migration'),
    ('system.config.database', 'port',     '5432',                 'DB ポート番号',        'migration', 'migration'),
    ('system.config.database', 'name',     '"k1s0_system"',        'DB 名',               'migration', 'migration'),
    ('system.config.database', 'ssl_mode', '"disable"',            'SSL モード',           'migration', 'migration')
ON CONFLICT (namespace, key) DO NOTHING;

-- system.config.server -- 設定サーバー設定
INSERT INTO config.config_entries (namespace, key, value_json, description, created_by, updated_by) VALUES
    ('system.config.server', 'port',          '8082',   'gRPC リッスンポート',     'migration', 'migration'),
    ('system.config.server', 'read_timeout',  '30',     '読み取りタイムアウト（秒）', 'migration', 'migration'),
    ('system.config.server', 'write_timeout', '30',     '書き込みタイムアウト（秒）', 'migration', 'migration')
ON CONFLICT (namespace, key) DO NOTHING;
```

### 005_create_service_config_mappings.up.sql

```sql
-- config-db: service_config_mappings テーブル作成

CREATE TABLE IF NOT EXISTS config.service_config_mappings (
    id               UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    service_name     VARCHAR(255) NOT NULL,
    config_entry_id  UUID         NOT NULL REFERENCES config.config_entries(id) ON DELETE CASCADE,
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_service_config_mappings_service_entry UNIQUE (service_name, config_entry_id)
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_service_config_mappings_service_name
    ON config.service_config_mappings (service_name);
CREATE INDEX IF NOT EXISTS idx_service_config_mappings_config_entry_id
    ON config.service_config_mappings (config_entry_id);
```

### 006_create_config_schemas.up.sql

```sql
-- config-db: config_schemas テーブル作成

CREATE TABLE IF NOT EXISTS config.config_schemas (
    id               UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    service_name     VARCHAR(255) NOT NULL UNIQUE,
    namespace_prefix VARCHAR(255) NOT NULL,
    schema_json      JSONB        NOT NULL,
    updated_by       VARCHAR(255) NOT NULL DEFAULT 'system',
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_config_schemas_service_name
    ON config.config_schemas (service_name);

-- updated_at 自動更新トリガー
CREATE TRIGGER trigger_config_schemas_update_updated_at
    BEFORE UPDATE ON config.config_schemas
    FOR EACH ROW EXECUTE FUNCTION config.update_updated_at();
```

---

## 接続設定

### config.yaml（config サーバー用）

```yaml
app:
  name: "config-server"
  version: "1.0.0"
  tier: "system"
  environment: "dev"

database:
  host: "postgres.k1s0-system.svc.cluster.local"
  port: 5432
  name: "config_db"
  user: "app"
  password: ""
  ssl_mode: "disable"
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"
```

### Vault によるクレデンシャル管理

| 用途 | Vault パス | 説明 |
|------|-----------|------|
| 静的パスワード | `secret/data/k1s0/system/config-server/database` | キー: `password` |
| 動的クレデンシャル（読み書き） | `database/creds/config-server-rw` | TTL: 24時間 |
| 動的クレデンシャル（読み取り専用） | `database/creds/config-server-ro` | TTL: 24時間 |

---

## 関連ドキュメント

- [system-database設計](system-database設計.md) -- auth-db テーブル設計（参照パターン）
- [tier-architecture](tier-architecture.md) -- Tier アーキテクチャ・データベースアクセスルール
- [config設計](config設計.md) -- config.yaml スキーマ
- [docker-compose設計](docker-compose設計.md) -- ローカル開発用 PostgreSQL
- [可観測性設計](可観測性設計.md) -- OpenTelemetry トレース ID 連携
