# system-service-catalog-server データベース設計

## スキーマ

スキーマ名: `service_catalog`

```sql
CREATE SCHEMA IF NOT EXISTS service_catalog;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| services | サービス定義（メタデータ・ライフサイクル・ティア） |
| teams | チーム定義（オーナー情報） |
| service_dependencies | サービス間の依存関係 |
| service_health | サービスヘルスステータス |
| service_docs | サービスドキュメントリンク |
| service_scorecards | サービススコアカード（品質メトリクス） |

---

## ER 図

![ER図](../../diagrams/service-catalog-er.svg)

---

## テーブル定義

### teams（チーム）

サービスを所有するチームの情報を管理する。

```sql
CREATE TABLE IF NOT EXISTS service_catalog.teams (
    id            TEXT         PRIMARY KEY,
    name          VARCHAR(255) NOT NULL UNIQUE,
    description   TEXT,
    contact_email VARCHAR(255),
    slack_channel VARCHAR(255),
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_teams_name ON service_catalog.teams (name);
CREATE INDEX IF NOT EXISTS idx_teams_created_at ON service_catalog.teams (created_at);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | TEXT | PK | チーム ID（一意識別子） |
| name | VARCHAR(255) | NOT NULL, UNIQUE | チーム名 |
| description | TEXT | | 説明 |
| contact_email | VARCHAR(255) | | 連絡先メール |
| slack_channel | VARCHAR(255) | | Slack チャンネル |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

### services（サービス）

サービスの基本情報・メタデータを管理する。

```sql
CREATE TABLE IF NOT EXISTS service_catalog.services (
    id             TEXT         PRIMARY KEY,
    name           VARCHAR(255) NOT NULL UNIQUE,
    display_name   VARCHAR(255),
    description    TEXT,
    owner_team_id  TEXT         NOT NULL REFERENCES service_catalog.teams(id),
    lifecycle      VARCHAR(50)  NOT NULL,
    tier           VARCHAR(50)  NOT NULL,
    repository_url TEXT,
    tags           JSONB        NOT NULL DEFAULT '[]'::jsonb,
    metadata       JSONB        NOT NULL DEFAULT '{}'::jsonb,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_services_lifecycle
        CHECK (lifecycle IN ('development', 'staging', 'production', 'deprecated')),
    CONSTRAINT chk_services_tier
        CHECK (tier IN ('system', 'business', 'service'))
);

CREATE INDEX IF NOT EXISTS idx_services_name ON service_catalog.services (name);
CREATE INDEX IF NOT EXISTS idx_services_owner_team_id ON service_catalog.services (owner_team_id);
CREATE INDEX IF NOT EXISTS idx_services_lifecycle ON service_catalog.services (lifecycle);
CREATE INDEX IF NOT EXISTS idx_services_tier ON service_catalog.services (tier);
CREATE INDEX IF NOT EXISTS idx_services_tags ON service_catalog.services USING GIN (tags);
CREATE INDEX IF NOT EXISTS idx_services_created_at ON service_catalog.services (created_at);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | TEXT | PK | サービス ID（一意識別子） |
| name | VARCHAR(255) | NOT NULL, UNIQUE | サービス名 |
| display_name | VARCHAR(255) | | 表示名 |
| description | TEXT | | 説明 |
| owner_team_id | TEXT | FK → teams.id, NOT NULL | オーナーチーム ID |
| lifecycle | VARCHAR(50) | NOT NULL, CHECK | ライフサイクル（development/staging/production/deprecated） |
| tier | VARCHAR(50) | NOT NULL, CHECK | ティア（system/business/service） |
| repository_url | TEXT | | リポジトリ URL |
| tags | JSONB | NOT NULL, DEFAULT '[]' | タグ一覧 |
| metadata | JSONB | NOT NULL, DEFAULT '{}' | メタデータ（任意の key-value） |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

### service_dependencies（サービス依存関係）

サービス間の依存関係を管理する。

```sql
CREATE TABLE IF NOT EXISTS service_catalog.service_dependencies (
    id                TEXT        PRIMARY KEY,
    source_service_id TEXT        NOT NULL REFERENCES service_catalog.services(id) ON DELETE CASCADE,
    target_service_id TEXT        NOT NULL REFERENCES service_catalog.services(id) ON DELETE CASCADE,
    dependency_type   VARCHAR(50) NOT NULL,
    description       TEXT,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_service_dependencies_source_target
        UNIQUE (source_service_id, target_service_id),
    CONSTRAINT chk_service_dependencies_type
        CHECK (dependency_type IN ('runtime', 'build', 'optional')),
    CONSTRAINT chk_service_dependencies_no_self
        CHECK (source_service_id != target_service_id)
);

CREATE INDEX IF NOT EXISTS idx_service_dependencies_source ON service_catalog.service_dependencies (source_service_id);
CREATE INDEX IF NOT EXISTS idx_service_dependencies_target ON service_catalog.service_dependencies (target_service_id);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | TEXT | PK | 依存関係 ID |
| source_service_id | TEXT | FK → services.id, NOT NULL | 依存元サービス ID |
| target_service_id | TEXT | FK → services.id, NOT NULL | 依存先サービス ID |
| dependency_type | VARCHAR(50) | NOT NULL, CHECK | 依存種別（runtime/build/optional） |
| description | TEXT | | 説明 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |

---

### service_health（サービスヘルスステータス）

各サービスの最新ヘルスステータスを管理する。

```sql
CREATE TABLE IF NOT EXISTS service_catalog.service_health (
    id            TEXT        PRIMARY KEY,
    service_id    TEXT        NOT NULL UNIQUE REFERENCES service_catalog.services(id) ON DELETE CASCADE,
    status        VARCHAR(50) NOT NULL,
    last_check_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    details       JSONB       NOT NULL DEFAULT '{}'::jsonb,
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_service_health_status
        CHECK (status IN ('healthy', 'degraded', 'unhealthy', 'unknown'))
);

CREATE INDEX IF NOT EXISTS idx_service_health_service_id ON service_catalog.service_health (service_id);
CREATE INDEX IF NOT EXISTS idx_service_health_status ON service_catalog.service_health (status);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | TEXT | PK | ヘルス ID |
| service_id | TEXT | FK → services.id, NOT NULL, UNIQUE | サービス ID |
| status | VARCHAR(50) | NOT NULL, CHECK | ステータス（healthy/degraded/unhealthy/unknown） |
| last_check_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 最終チェック日時 |
| details | JSONB | NOT NULL, DEFAULT '{}' | 詳細情報 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

### service_docs（サービスドキュメント）

サービスに関連するドキュメントリンクを管理する。

```sql
CREATE TABLE IF NOT EXISTS service_catalog.service_docs (
    id         TEXT         PRIMARY KEY,
    service_id TEXT         NOT NULL REFERENCES service_catalog.services(id) ON DELETE CASCADE,
    title      VARCHAR(255) NOT NULL,
    url        TEXT         NOT NULL,
    doc_type   VARCHAR(50)  NOT NULL,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_service_docs_type
        CHECK (doc_type IN ('api_spec', 'design', 'runbook', 'other'))
);

CREATE INDEX IF NOT EXISTS idx_service_docs_service_id ON service_catalog.service_docs (service_id);
CREATE INDEX IF NOT EXISTS idx_service_docs_doc_type ON service_catalog.service_docs (doc_type);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | TEXT | PK | ドキュメント ID |
| service_id | TEXT | FK → services.id, NOT NULL | サービス ID |
| title | VARCHAR(255) | NOT NULL | タイトル |
| url | TEXT | NOT NULL | ドキュメント URL |
| doc_type | VARCHAR(50) | NOT NULL, CHECK | ドキュメント種別（api_spec/design/runbook/other） |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

### service_scorecards（サービススコアカード）

サービスの品質メトリクス・SLO 達成率を管理する。

```sql
CREATE TABLE IF NOT EXISTS service_catalog.service_scorecards (
    id                  TEXT        PRIMARY KEY,
    service_id          TEXT        NOT NULL UNIQUE REFERENCES service_catalog.services(id) ON DELETE CASCADE,
    documentation_score INT         NOT NULL DEFAULT 0,
    test_coverage       INT         NOT NULL DEFAULT 0,
    slo_compliance      INT         NOT NULL DEFAULT 0,
    security_score      INT         NOT NULL DEFAULT 0,
    overall_score       INT         NOT NULL DEFAULT 0,
    evaluated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_scorecards_documentation CHECK (documentation_score BETWEEN 0 AND 100),
    CONSTRAINT chk_scorecards_test_coverage CHECK (test_coverage BETWEEN 0 AND 100),
    CONSTRAINT chk_scorecards_slo CHECK (slo_compliance BETWEEN 0 AND 100),
    CONSTRAINT chk_scorecards_security CHECK (security_score BETWEEN 0 AND 100),
    CONSTRAINT chk_scorecards_overall CHECK (overall_score BETWEEN 0 AND 100)
);

CREATE INDEX IF NOT EXISTS idx_service_scorecards_service_id ON service_catalog.service_scorecards (service_id);
CREATE INDEX IF NOT EXISTS idx_service_scorecards_overall ON service_catalog.service_scorecards (overall_score);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | TEXT | PK | スコアカード ID |
| service_id | TEXT | FK → services.id, NOT NULL, UNIQUE | サービス ID |
| documentation_score | INT | NOT NULL, DEFAULT 0, CHECK 0-100 | ドキュメント充実度スコア |
| test_coverage | INT | NOT NULL, DEFAULT 0, CHECK 0-100 | テストカバレッジスコア |
| slo_compliance | INT | NOT NULL, DEFAULT 0, CHECK 0-100 | SLO 達成率スコア |
| security_score | INT | NOT NULL, DEFAULT 0, CHECK 0-100 | セキュリティスコア |
| overall_score | INT | NOT NULL, DEFAULT 0, CHECK 0-100 | 総合スコア |
| evaluated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 最終評価日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

## マイグレーション

マイグレーションファイルは `regions/system/database/service-catalog-db/migrations/` に配置。

| ファイル | 内容 |
| --- | --- |
| `001_create_schema.up.sql` | `service_catalog` スキーマ・pgcrypto 拡張・updated_at 関数作成 |
| `001_create_schema.down.sql` | スキーマ削除 |
| `002_create_teams.up.sql` | teams テーブル・インデックス作成 |
| `002_create_teams.down.sql` | テーブル削除 |
| `003_create_services.up.sql` | services テーブル・制約・インデックス作成 |
| `003_create_services.down.sql` | テーブル削除 |
| `004_create_service_dependencies.up.sql` | service_dependencies テーブル・制約・インデックス作成 |
| `004_create_service_dependencies.down.sql` | テーブル削除 |
| `005_create_service_health.up.sql` | service_health テーブル・インデックス作成 |
| `005_create_service_health.down.sql` | テーブル削除 |
| `006_create_service_docs.up.sql` | service_docs テーブル・インデックス作成 |
| `006_create_service_docs.down.sql` | テーブル削除 |
| `007_create_service_scorecards.up.sql` | service_scorecards テーブル・制約・インデックス作成 |
| `007_create_service_scorecards.down.sql` | テーブル削除 |
| `008_add_uuid_defaults.up.sql` | services・teams・service_docs の PK に DEFAULT gen_random_uuid() 追加（H-011 監査対応） |
| `008_add_uuid_defaults.down.sql` | UUID デフォルト値削除 |

---

## updated_at 自動更新トリガー

```sql
CREATE OR REPLACE FUNCTION service_catalog.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_teams_update_updated_at
    BEFORE UPDATE ON service_catalog.teams
    FOR EACH ROW EXECUTE FUNCTION service_catalog.update_updated_at();

CREATE TRIGGER trigger_services_update_updated_at
    BEFORE UPDATE ON service_catalog.services
    FOR EACH ROW EXECUTE FUNCTION service_catalog.update_updated_at();

CREATE TRIGGER trigger_service_health_update_updated_at
    BEFORE UPDATE ON service_catalog.service_health
    FOR EACH ROW EXECUTE FUNCTION service_catalog.update_updated_at();

CREATE TRIGGER trigger_service_docs_update_updated_at
    BEFORE UPDATE ON service_catalog.service_docs
    FOR EACH ROW EXECUTE FUNCTION service_catalog.update_updated_at();

CREATE TRIGGER trigger_service_scorecards_update_updated_at
    BEFORE UPDATE ON service_catalog.service_scorecards
    FOR EACH ROW EXECUTE FUNCTION service_catalog.update_updated_at();
```
