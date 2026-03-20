# system-quota-server データベース設計

## スキーマ

スキーマ名: `quota`

```sql
CREATE SCHEMA IF NOT EXISTS quota;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| quota_policies | クォータポリシー定義 |
| quota_usage | クォータ使用量 |

---

## ER 図

```
quota_policies 1──* quota_usage
```

---

## テーブル定義

### quota_policies（クォータポリシー）

テナント・ユーザー・API キー単位のクォータ制限を定義する。アラート閾値による通知制御を持つ。

```sql
CREATE TABLE IF NOT EXISTS quota.quota_policies (
    id                      VARCHAR(64)  PRIMARY KEY,
    name                    VARCHAR(255) NOT NULL UNIQUE,
    subject_type            VARCHAR(50)  NOT NULL,
    subject_id              VARCHAR(255) NOT NULL DEFAULT '',
    quota_limit             BIGINT       NOT NULL,
    period                  VARCHAR(50)  NOT NULL,
    enabled                 BOOLEAN      NOT NULL DEFAULT true,
    alert_threshold_percent SMALLINT     NOT NULL DEFAULT 80,
    created_at              TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_quota_policies_subject_type CHECK (subject_type IN ('tenant', 'user', 'api_key')),
    CONSTRAINT chk_quota_policies_period CHECK (period IN ('daily', 'monthly')),
    CONSTRAINT chk_quota_policies_alert_threshold_percent CHECK (alert_threshold_percent BETWEEN 0 AND 100)
);

CREATE INDEX IF NOT EXISTS idx_quota_policies_name ON quota.quota_policies (name);
CREATE INDEX IF NOT EXISTS idx_quota_policies_subject ON quota.quota_policies (subject_type, subject_id);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | VARCHAR(64) | PK | 主キー（プレフィックス付き ID: `quota_`） |
| name | VARCHAR(255) | UNIQUE, NOT NULL | ポリシー名 |
| subject_type | VARCHAR(50) | NOT NULL | 対象種別（tenant/user/api_key） |
| subject_id | VARCHAR(255) | NOT NULL, DEFAULT '' | 対象 ID |
| quota_limit | BIGINT | NOT NULL | クォータ上限 |
| period | VARCHAR(50) | NOT NULL | 期間（daily/monthly） |
| enabled | BOOLEAN | NOT NULL, DEFAULT true | 有効フラグ |
| alert_threshold_percent | SMALLINT | NOT NULL, DEFAULT 80 | アラート閾値（0-100%） |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

### quota_usage（クォータ使用量）

クォータポリシーに対する現在の使用量を追跡する。ウィンドウ開始時刻でリセットタイミングを管理する。

```sql
CREATE TABLE IF NOT EXISTS quota.quota_usage (
    id                  UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_id           VARCHAR(64)  NOT NULL REFERENCES quota.quota_policies(id) ON DELETE CASCADE,
    tenant_id           VARCHAR(255) NOT NULL DEFAULT '',
    subject_type        VARCHAR(50)  NOT NULL DEFAULT 'tenant',
    subject_id          VARCHAR(255) NOT NULL DEFAULT '',
    current_usage       BIGINT       NOT NULL DEFAULT 0,
    window_start        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    last_incremented_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_quota_usage_policy_id ON quota.quota_usage (policy_id);
CREATE INDEX IF NOT EXISTS idx_quota_usage_window ON quota.quota_usage (window_start);
CREATE UNIQUE INDEX IF NOT EXISTS idx_quota_usage_policy_subject
    ON quota.quota_usage (policy_id, subject_type, subject_id);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| policy_id | VARCHAR(64) | FK → quota_policies.id, NOT NULL | ポリシー ID |
| tenant_id | VARCHAR(255) | NOT NULL, DEFAULT '' | テナント ID（レガシー） |
| subject_type | VARCHAR(50) | NOT NULL, DEFAULT 'tenant' | 対象種別 |
| subject_id | VARCHAR(255) | NOT NULL, DEFAULT '' | 対象 ID |
| current_usage | BIGINT | NOT NULL, DEFAULT 0 | 現在の使用量 |
| window_start | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | ウィンドウ開始日時 |
| last_incremented_at | TIMESTAMPTZ | | 最終インクリメント日時 |

---

## マイグレーション

マイグレーションファイルは `regions/system/database/quota-db/migrations/` に配置。

| ファイル | 内容 |
| --- | --- |
| `001_create_schema.up.sql` | `quota` スキーマ・pgcrypto 拡張・updated_at 関数作成 |
| `001_create_schema.down.sql` | スキーマ削除 |
| `002_create_quota_policies.up.sql` | quota_policies テーブル作成 |
| `002_create_quota_policies.down.sql` | テーブル削除 |
| `003_create_quota_usage.up.sql` | quota_usage テーブル作成 |
| `003_create_quota_usage.down.sql` | テーブル削除 |
| `004_add_subject_fields_to_quota_usage.up.sql` | subject_type/subject_id カラム追加・ユニーク制約変更 |
| `004_add_subject_fields_to_quota_usage.down.sql` | カラム削除 |
| `005_alter_alert_threshold_percent_smallint.up.sql` | alert_threshold_percent を SMALLINT に変更 |
| `005_alter_alert_threshold_percent_smallint.down.sql` | 型変更復元 |
| `006_convert_prefixed_ids.up.sql` | UUID → プレフィックス付き VARCHAR(64) ID に変換 |
| `006_convert_prefixed_ids.down.sql` | ID 変換復元 |

---

## updated_at 自動更新トリガー

```sql
CREATE OR REPLACE FUNCTION quota.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_quota_policies_update_updated_at
    BEFORE UPDATE ON quota.quota_policies
    FOR EACH ROW EXECUTE FUNCTION quota.update_updated_at();
```
