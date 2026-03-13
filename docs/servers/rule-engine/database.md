# system-rule-engine-database 設計

system Tier のルールエンジンデータベース（rule-engine-db）の設計を定義する。
配置先: `regions/system/database/rule-engine-db/`

## 概要

rule-engine-db は system Tier に属する PostgreSQL 17 データベースであり、業務ルール定義・ルールセット・バージョン管理・評価監査ログを管理する。

[tier-architecture.md](../../architecture/overview/tier-architecture.md) の設計原則に従い、rule-engine-db へのアクセスは **system Tier のサーバーからのみ** 許可する。

### 技術スタック

| コンポーネント | 技術 | バージョン |
|----------------|------|------------|
| RDBMS | PostgreSQL | 17 |
| マイグレーション | sqlx-cli | - |
| ORM / クエリビルダー | sqlx | - |
| シークレット管理 | HashiCorp Vault | 1.17 |

---

## テーブル定義

### rules テーブル

個別の業務ルール定義を管理する。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | ルールの一意識別子 |
| name | VARCHAR(255) | NOT NULL | ルール名（例: tax-rate-standard） |
| description | TEXT | | ルールの説明 |
| condition | JSONB | NOT NULL | 条件式（AST 形式の JSON） |
| action | JSONB | NOT NULL | 条件マッチ時のアクション定義 |
| priority | INT | NOT NULL DEFAULT 0 | 評価優先度（数値が大きいほど優先） |
| enabled | BOOLEAN | NOT NULL DEFAULT TRUE | 有効フラグ |
| metadata | JSONB | NOT NULL DEFAULT '{}' | ルールのメタデータ（タグ・カテゴリ等） |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 更新日時（トリガーで自動更新） |

### rule_sets テーブル

複数のルールをグループ化したルールセットを管理する。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | ルールセットの一意識別子 |
| name | VARCHAR(255) | UNIQUE NOT NULL | ルールセット名（例: pricing-rules） |
| description | TEXT | | ルールセットの説明 |
| enabled | BOOLEAN | NOT NULL DEFAULT TRUE | 有効フラグ |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 更新日時（トリガーで自動更新） |

### rule_set_versions テーブル

ルールセットのバージョン管理。ルールセットに含まれるルール群のスナップショットを保持する。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | バージョンの一意識別子 |
| rule_set_id | UUID | FK rule_sets(id) ON DELETE CASCADE, NOT NULL | 所属するルールセットの ID |
| version | INT | NOT NULL | バージョン番号 |
| rule_ids | JSONB | NOT NULL | バージョンに含まれるルール ID 配列 |
| is_active | BOOLEAN | NOT NULL DEFAULT FALSE | アクティブバージョンフラグ |
| published_at | TIMESTAMPTZ | | 公開日時 |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |
| | | UNIQUE(rule_set_id, version) | ルールセットとバージョンの組み合わせで一意 |

### evaluation_logs テーブル

ルール評価の監査ログ。「なぜこの判定になったか」のトレーサビリティを提供する。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | 評価ログの一意識別子 |
| rule_set_id | UUID | NOT NULL | 評価対象のルールセット ID |
| rule_set_version | INT | NOT NULL | 評価に使用したルールセットバージョン |
| input_data | JSONB | NOT NULL | 評価入力データ |
| matched_rules | JSONB | NOT NULL DEFAULT '[]' | マッチしたルール ID と結果の配列 |
| result | JSONB | NOT NULL | 最終評価結果 |
| evaluation_time_ms | INT | NOT NULL | 評価処理時間（ミリ秒） |
| correlation_id | VARCHAR(255) | | 業務相関 ID |
| trace_id | VARCHAR(64) | | OpenTelemetry トレース ID |
| evaluated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 評価日時 |

---

## インデックス設計

| テーブル | インデックス名 | カラム | 種別 | 用途 |
|----------|---------------|--------|------|------|
| rules | idx_rules_name | name | B-tree | ルール名による検索 |
| rules | idx_rules_enabled | enabled | B-tree | 有効フラグでのフィルタリング |
| rules | idx_rules_priority | priority | B-tree | 優先度順ソート |
| rule_sets | idx_rule_sets_name | name | B-tree | ルールセット名による検索 |
| rule_sets | idx_rule_sets_enabled | enabled | B-tree | 有効フラグでのフィルタリング |
| rule_set_versions | idx_rule_set_versions_rule_set_id | rule_set_id | B-tree | ルールセット ID による検索 |
| rule_set_versions | idx_rule_set_versions_is_active | is_active (WHERE is_active = TRUE) | B-tree（部分） | アクティブバージョンの検索 |
| evaluation_logs | idx_evaluation_logs_rule_set_id | rule_set_id | B-tree | ルールセット ID による検索 |
| evaluation_logs | idx_evaluation_logs_evaluated_at | evaluated_at | B-tree | 評価日時による範囲検索 |
| evaluation_logs | idx_evaluation_logs_correlation_id | correlation_id (WHERE NOT NULL) | B-tree（部分） | 相関 ID による検索 |
| evaluation_logs | idx_evaluation_logs_trace_id | trace_id (WHERE NOT NULL) | B-tree（部分） | トレース ID による検索 |

---

## マイグレーションファイル

配置先: `regions/system/database/rule-engine-db/migrations/`

命名規則は [テンプレート仕様-データベース](../../templates/data/データベース.md) に準拠する。

```
migrations/
├── 001_create_schema.up.sql                    # スキーマ・拡張機能・共通関数
├── 001_create_schema.down.sql
├── 002_create_rules.up.sql                     # rules テーブル
├── 002_create_rules.down.sql
├── 003_create_rule_sets.up.sql                 # rule_sets テーブル
├── 003_create_rule_sets.down.sql
├── 004_create_rule_set_versions.up.sql         # rule_set_versions テーブル
├── 004_create_rule_set_versions.down.sql
├── 005_create_evaluation_logs.up.sql           # evaluation_logs テーブル
└── 005_create_evaluation_logs.down.sql
```

### マイグレーション SQL

#### 001_create_schema.up.sql

```sql
-- rule-engine-db: スキーマ・拡張機能・共通関数の作成 (PostgreSQL 17)

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE SCHEMA IF NOT EXISTS rule_engine;

CREATE OR REPLACE FUNCTION rule_engine.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

#### 002_create_rules.up.sql

```sql
-- rule-engine-db: rules テーブル作成

CREATE TABLE IF NOT EXISTS rule_engine.rules (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(255) NOT NULL,
    description TEXT,
    condition   JSONB        NOT NULL,
    action      JSONB        NOT NULL,
    priority    INT          NOT NULL DEFAULT 0,
    enabled     BOOLEAN      NOT NULL DEFAULT TRUE,
    metadata    JSONB        NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_rules_name
    ON rule_engine.rules (name);
CREATE INDEX IF NOT EXISTS idx_rules_enabled
    ON rule_engine.rules (enabled);
CREATE INDEX IF NOT EXISTS idx_rules_priority
    ON rule_engine.rules (priority);

CREATE TRIGGER trigger_rules_update_updated_at
    BEFORE UPDATE ON rule_engine.rules
    FOR EACH ROW
    EXECUTE FUNCTION rule_engine.update_updated_at();
```

#### 003_create_rule_sets.up.sql

```sql
-- rule-engine-db: rule_sets テーブル作成

CREATE TABLE IF NOT EXISTS rule_engine.rule_sets (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(255) UNIQUE NOT NULL,
    description TEXT,
    enabled     BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_rule_sets_name
    ON rule_engine.rule_sets (name);
CREATE INDEX IF NOT EXISTS idx_rule_sets_enabled
    ON rule_engine.rule_sets (enabled);

CREATE TRIGGER trigger_rule_sets_update_updated_at
    BEFORE UPDATE ON rule_engine.rule_sets
    FOR EACH ROW
    EXECUTE FUNCTION rule_engine.update_updated_at();
```

#### 004_create_rule_set_versions.up.sql

```sql
-- rule-engine-db: rule_set_versions テーブル作成

CREATE TABLE IF NOT EXISTS rule_engine.rule_set_versions (
    id           UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_set_id  UUID         NOT NULL REFERENCES rule_engine.rule_sets(id) ON DELETE CASCADE,
    version      INT          NOT NULL,
    rule_ids     JSONB        NOT NULL,
    is_active    BOOLEAN      NOT NULL DEFAULT FALSE,
    published_at TIMESTAMPTZ,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_rule_set_versions_rule_set_version UNIQUE (rule_set_id, version)
);

CREATE INDEX IF NOT EXISTS idx_rule_set_versions_rule_set_id
    ON rule_engine.rule_set_versions (rule_set_id);
CREATE INDEX IF NOT EXISTS idx_rule_set_versions_is_active
    ON rule_engine.rule_set_versions (is_active)
    WHERE is_active = TRUE;
```

#### 005_create_evaluation_logs.up.sql

```sql
-- rule-engine-db: evaluation_logs テーブル作成

CREATE TABLE IF NOT EXISTS rule_engine.evaluation_logs (
    id                  UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_set_id         UUID         NOT NULL,
    rule_set_version    INT          NOT NULL,
    input_data          JSONB        NOT NULL,
    matched_rules       JSONB        NOT NULL DEFAULT '[]',
    result              JSONB        NOT NULL,
    evaluation_time_ms  INT          NOT NULL,
    correlation_id      VARCHAR(255),
    trace_id            VARCHAR(64),
    evaluated_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_evaluation_logs_rule_set_id
    ON rule_engine.evaluation_logs (rule_set_id);
CREATE INDEX IF NOT EXISTS idx_evaluation_logs_evaluated_at
    ON rule_engine.evaluation_logs (evaluated_at);
CREATE INDEX IF NOT EXISTS idx_evaluation_logs_correlation_id
    ON rule_engine.evaluation_logs (correlation_id)
    WHERE correlation_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_evaluation_logs_trace_id
    ON rule_engine.evaluation_logs (trace_id)
    WHERE trace_id IS NOT NULL;
```

---

## 接続設定

[config設計](../../cli/config/config設計.md) の database セクションに従い、rule-engine-db への接続を設定する。

### 環境別設定

| 環境 | host | ssl_mode | max_open_conns | max_idle_conns |
|------|------|----------|----------------|----------------|
| dev | localhost (docker-compose) | disable | 10 | 3 |
| staging | postgres.k1s0-system.svc.cluster.local | require | 25 | 5 |
| prod | postgres.k1s0-system.svc.cluster.local | verify-full | 50 | 10 |

### Vault によるクレデンシャル管理

| 用途 | Vault パス | 説明 |
|------|-----------|------|
| 静的パスワード | `secret/data/k1s0/system/rule-engine/database` | キー: `password` |
| 動的クレデンシャル（読み書き） | `database/creds/system-rule-engine-rw` | Vault Database エンジンで自動生成（TTL: 24時間） |
| 動的クレデンシャル（読み取り専用） | `database/creds/system-rule-engine-ro` | Vault Database エンジンで自動生成（TTL: 24時間） |

### docker-compose（ローカル開発）

[docker-compose設計](../../infrastructure/docker/docker-compose設計.md) の共通 PostgreSQL インスタンスに `k1s0_system` データベースを使用する。rule-engine-db は `k1s0_system` データベース内の `rule_engine` スキーマとして共存する。

---

## 関連ドキュメント

- [system-rule-engine-server設計](server.md) -- ルールエンジンサーバー設計
- [tier-architecture](../../architecture/overview/tier-architecture.md) -- Tier アーキテクチャ・データベースアクセスルール
- [config設計](../../cli/config/config設計.md) -- config.yaml スキーマ（database セクション）
- [テンプレート仕様-データベース](../../templates/data/データベース.md) -- マイグレーション命名規則・テンプレート
- [docker-compose設計](../../infrastructure/docker/docker-compose設計.md) -- ローカル開発用 PostgreSQL
