# system-ratelimit-server データベース設計

## スキーマ

スキーマ名: `ratelimit`

```sql
CREATE SCHEMA IF NOT EXISTS ratelimit;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| rate_limit_rules | レートリミットルール定義 |

---

## ER 図

```
rate_limit_rules（単独テーブル、FK なし）
```

---

## テーブル定義

### rate_limit_rules（レートリミットルール）

API リクエストのレートリミットルールを定義する。複数のアルゴリズム（token_bucket, fixed_window, sliding_window, leaky_bucket）をサポートし、スコープと識別子パターンでルール適用対象を制御する。

```sql
CREATE TABLE IF NOT EXISTS ratelimit.rate_limit_rules (
    id                 UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name               VARCHAR(255) NOT NULL UNIQUE,
    key                TEXT,
    limit_count        BIGINT       NOT NULL,
    window_secs        BIGINT       NOT NULL,
    algorithm          VARCHAR(50)  NOT NULL DEFAULT 'token_bucket',
    enabled            BOOLEAN      NOT NULL DEFAULT true,
    scope              VARCHAR(50)  NOT NULL DEFAULT 'service',
    identifier_pattern VARCHAR(255) NOT NULL DEFAULT '*',
    created_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_ratelimit_rules_algorithm CHECK (algorithm IN ('token_bucket', 'fixed_window', 'sliding_window', 'leaky_bucket')),
    CONSTRAINT chk_ratelimit_rules_limit_count CHECK (limit_count > 0),
    CONSTRAINT chk_ratelimit_rules_window_secs CHECK (window_secs > 0)
);

CREATE INDEX IF NOT EXISTS idx_ratelimit_rules_name ON ratelimit.rate_limit_rules (name);
CREATE INDEX IF NOT EXISTS idx_ratelimit_rules_enabled ON ratelimit.rate_limit_rules (enabled) WHERE enabled = true;
CREATE INDEX IF NOT EXISTS idx_ratelimit_rules_scope_identifier ON ratelimit.rate_limit_rules (scope, identifier_pattern);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| name | VARCHAR(255) | UNIQUE, NOT NULL | ルール名 |
| key | TEXT | | レートリミットキー |
| limit_count | BIGINT | NOT NULL, CHECK > 0 | リクエスト上限数 |
| window_secs | BIGINT | NOT NULL, CHECK > 0 | ウィンドウ秒数 |
| algorithm | VARCHAR(50) | NOT NULL, DEFAULT 'token_bucket' | アルゴリズム（token_bucket/fixed_window/sliding_window/leaky_bucket） |
| enabled | BOOLEAN | NOT NULL, DEFAULT true | 有効フラグ |
| scope | VARCHAR(50) | NOT NULL, DEFAULT 'service' | スコープ |
| identifier_pattern | VARCHAR(255) | NOT NULL, DEFAULT '*' | 識別子パターン |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

## マイグレーション

マイグレーションファイルは `regions/system/database/ratelimit-db/migrations/` に配置。

| ファイル | 内容 |
| --- | --- |
| `001_create_schema.up.sql` | `ratelimit` スキーマ・pgcrypto 拡張・updated_at 関数作成 |
| `001_create_schema.down.sql` | スキーマ削除 |
| `002_create_rate_limit_rules.up.sql` | rate_limit_rules テーブル作成 |
| `002_create_rate_limit_rules.down.sql` | テーブル削除 |
| `003_add_scope_and_identifier_pattern.up.sql` | scope・identifier_pattern カラム追加 |
| `003_add_scope_and_identifier_pattern.down.sql` | カラム削除 |
| `004_add_leaky_bucket_algorithm.up.sql` | algorithm チェック制約に leaky_bucket 追加 |
| `004_add_leaky_bucket_algorithm.down.sql` | チェック制約復元 |

---

## updated_at 自動更新トリガー

```sql
CREATE OR REPLACE FUNCTION ratelimit.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_ratelimit_rules_update_updated_at
    BEFORE UPDATE ON ratelimit.rate_limit_rules
    FOR EACH ROW EXECUTE FUNCTION ratelimit.update_updated_at();
```
