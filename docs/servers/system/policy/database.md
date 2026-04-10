# system-policy-server データベース設計

## スキーマ

スキーマ名: `policy`

```sql
CREATE SCHEMA IF NOT EXISTS policy;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| policies | OPA Rego ポリシー定義 |
| policy_bundles | ポリシーバンドル |

---

## ER 図

```
policy_bundles ──* policies (bundle_id で論理的に紐付け)
```

---

## テーブル定義

### policies（ポリシー定義）

OPA（Open Policy Agent）で使用する Rego ポリシーを管理する。バージョン管理とバンドル紐付けを持つ。

```sql
CREATE TABLE IF NOT EXISTS policy.policies (
    id           UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name         VARCHAR(255) NOT NULL UNIQUE,
    description  TEXT         NOT NULL DEFAULT '',
    rego_content TEXT         NOT NULL,
    package_path VARCHAR(255) NOT NULL DEFAULT '',
    enabled      BOOLEAN      NOT NULL DEFAULT true,
    version      INT          NOT NULL DEFAULT 1,
    bundle_id    VARCHAR(255),
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_policies_name ON policy.policies (name);
CREATE INDEX IF NOT EXISTS idx_policies_enabled ON policy.policies (enabled);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| name | VARCHAR(255) | UNIQUE, NOT NULL | ポリシー名 |
| description | TEXT | NOT NULL, DEFAULT '' | 説明 |
| rego_content | TEXT | NOT NULL | Rego ポリシー本体 |
| package_path | VARCHAR(255) | NOT NULL, DEFAULT '' | Rego パッケージパス |
| enabled | BOOLEAN | NOT NULL, DEFAULT true | 有効フラグ |
| version | INT | NOT NULL, DEFAULT 1 | バージョン番号 |
| bundle_id | VARCHAR(255) | | バンドル ID |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

### policy_bundles（ポリシーバンドル）

複数ポリシーをグループ化するバンドルを管理する。policies カラムで含まれるポリシー ID の一覧を保持する。

```sql
CREATE TABLE IF NOT EXISTS policy.policy_bundles (
    id         UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name       VARCHAR(255) NOT NULL,
    policies   JSONB        NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_policy_bundles_name ON policy.policy_bundles (name);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| name | VARCHAR(255) | NOT NULL | バンドル名 |
| policies | JSONB | NOT NULL, DEFAULT '[]' | 含まれるポリシー ID 一覧 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |

---

## マイグレーション

マイグレーションファイルは `regions/system/database/policy-db/migrations/` に配置。

| ファイル | 内容 |
| --- | --- |
| `001_create_schema.up.sql` | `policy` スキーマ・pgcrypto 拡張・updated_at 関数作成 |
| `001_create_schema.down.sql` | スキーマ削除 |
| `002_create_policies.up.sql` | policies テーブル作成 |
| `002_create_policies.down.sql` | テーブル削除 |
| `003_create_policy_bundles.up.sql` | policy_bundles テーブル作成 |
| `003_create_policy_bundles.down.sql` | テーブル削除 |
| `004_add_bundle_id.up.sql` | policies に bundle_id カラム追加 |
| `004_add_bundle_id.down.sql` | カラム削除 |
| `005_add_tenant_id_rls.up.sql` | 全テーブルに `tenant_id VARCHAR(255) NOT NULL` と RLS ポリシー追加 |
| `006_add_rls_with_check.up.sql` | RLS ポリシーに AS RESTRICTIVE + WITH CHECK 追加 |
| `007_alter_tenant_id_to_text.up.sql` | `tenant_id` を TEXT 型に変更、UNIQUE(name) → UNIQUE(tenant_id, name)（CRITICAL-DB-002 + HIGH-DB-007 対応） |

---

## マルチテナント対応（CRITICAL-DB-002 / HIGH-DB-007）

`policies` / `policy_bundles` の `tenant_id` を TEXT 型に統一（migration 007）。

- `policies`: UNIQUE(name) を UNIQUE(tenant_id, name) に変更

```sql
ALTER TABLE policy.{table} ENABLE ROW LEVEL SECURITY;
ALTER TABLE policy.{table} FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON policy.{table}
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));
```

---

## updated_at 自動更新トリガー

```sql
CREATE OR REPLACE FUNCTION policy.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_policies_update_updated_at
    BEFORE UPDATE ON policy.policies
    FOR EACH ROW EXECUTE FUNCTION policy.update_updated_at();
```
