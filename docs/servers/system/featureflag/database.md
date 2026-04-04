# system-featureflag-server データベース設計

## スキーマ

スキーマ名: `featureflag`

```sql
CREATE SCHEMA IF NOT EXISTS featureflag;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| feature_flags | フィーチャーフラグ定義 |
| flag_audit_logs | フラグ変更監査ログ |

> **tenant_id 型**: `006_alter_tenant_id_to_text.up.sql`（C-004 監査対応）により `UUID` から `TEXT` に変更済み（ADR-0093）。

---

## ER 図

```
feature_flags
  └── flag_audit_logs (flag_id -> feature_flags.id)
```

---

## テーブル定義

### feature_flags（フィーチャーフラグ定義）

フィーチャーフラグのキー・有効状態・バリアント・ルールをテナントスコープで管理する。

```sql
CREATE TABLE IF NOT EXISTS featureflag.feature_flags (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   TEXT         NOT NULL,
    flag_key    VARCHAR(255) NOT NULL,
    description TEXT         NOT NULL DEFAULT '',
    enabled     BOOLEAN      NOT NULL DEFAULT false,
    variants    JSONB        NOT NULL DEFAULT '[]',
    rules       JSONB        NOT NULL DEFAULT '[]',
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_feature_flags_tenant_key UNIQUE (tenant_id, flag_key)
);

CREATE INDEX IF NOT EXISTS idx_feature_flags_tenant_id ON featureflag.feature_flags (tenant_id);
CREATE INDEX IF NOT EXISTS idx_feature_flags_tenant_key ON featureflag.feature_flags (tenant_id, flag_key);
CREATE INDEX IF NOT EXISTS idx_feature_flags_enabled ON featureflag.feature_flags (enabled);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| tenant_id | TEXT | NOT NULL | テナントID（STATIC-CRITICAL-001 テナント分離・C-004 対応で UUID→TEXT 変更済み） |
| flag_key | VARCHAR(255) | NOT NULL | フラグキー（テナント内一意） |
| description | TEXT | NOT NULL, DEFAULT '' | 説明 |
| enabled | BOOLEAN | NOT NULL, DEFAULT false | 有効フラグ |
| variants | JSONB | NOT NULL, DEFAULT '[]' | バリアント定義（A/B テスト等） |
| rules | JSONB | NOT NULL, DEFAULT '[]' | 評価ルール定義 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

> **テナント分離**: `(tenant_id, flag_key)` の複合 UNIQUE 制約により、テナント間でのフラグキー衝突を防ぐ。全クエリに `WHERE tenant_id = $X` を必須とする。

---

### flag_audit_logs（フラグ変更監査ログ）

フラグの変更履歴をテナントスコープで記録する。

```sql
CREATE TABLE IF NOT EXISTS featureflag.flag_audit_logs (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   TEXT         NOT NULL,
    flag_id     UUID         NOT NULL,
    flag_key    VARCHAR(255) NOT NULL,
    action      VARCHAR(50)  NOT NULL,
    before_json JSONB,
    after_json  JSONB,
    changed_by  VARCHAR(255) NOT NULL,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_flag_audit_logs_tenant_id ON featureflag.flag_audit_logs (tenant_id);
CREATE INDEX IF NOT EXISTS idx_flag_audit_logs_flag_id ON featureflag.flag_audit_logs (flag_id);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| tenant_id | TEXT | NOT NULL | テナントID（STATIC-CRITICAL-001 テナント分離・C-004 対応で UUID→TEXT 変更済み） |
| flag_id | UUID | NOT NULL | 対象フラグの ID |
| flag_key | VARCHAR(255) | NOT NULL | 対象フラグキー（非正規化） |
| action | VARCHAR(50) | NOT NULL | 操作種別（CREATED / UPDATED / DELETED） |
| before_json | JSONB | NULL 許可 | 変更前スナップショット |
| after_json | JSONB | NULL 許可 | 変更後スナップショット |
| changed_by | VARCHAR(255) | NOT NULL | 変更者識別子（ユーザー名等） |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 記録日時 |

---

## マイグレーション

マイグレーションファイルは `regions/system/database/featureflag-db/migrations/` に配置。

| ファイル | 内容 |
| --- | --- |
| `001_create_schema.up.sql` | `featureflag` スキーマ・pgcrypto 拡張・updated_at 関数作成 |
| `001_create_schema.down.sql` | スキーマ削除 |
| `002_create_feature_flags.up.sql` | feature_flags テーブル作成 |
| `002_create_feature_flags.down.sql` | テーブル削除 |
| `003_create_flag_audit_logs.up.sql` | flag_audit_logs テーブル作成 |
| `003_create_flag_audit_logs.down.sql` | テーブル削除 |
| `004_add_tenant_id.up.sql` | feature_flags / flag_audit_logs に tenant_id カラム追加（STATIC-CRITICAL-001） |
| `004_add_tenant_id.down.sql` | tenant_id カラム削除 |
| `005_add_rls.up.sql` | feature_flags テーブルに RLS（行レベルセキュリティ）追加（CRIT-001 監査対応） |
| `005_add_rls.down.sql` | RLS ポリシー・設定削除 |
| `006_alter_tenant_id_to_text.up.sql` | feature_flags / flag_audit_logs の tenant_id を UUID→TEXT に変更（C-004 監査対応・ADR-0093） |
| `006_alter_tenant_id_to_text.down.sql` | tenant_id を TEXT→UUID に戻す（注意: 非 UUID 値が存在する場合は失敗） |

### 005_add_rls.up.sql
**目的**: Row Level Security (RLS) によるテナント分離の強化

| 変更内容 | 詳細 |
|---------|------|
| RLS 有効化 | `ALTER TABLE featureflag.feature_flags ENABLE ROW LEVEL SECURITY;` |
| FORCE RLS | `ALTER TABLE featureflag.feature_flags FORCE ROW LEVEL SECURITY;` |
| テナント分離ポリシー | `CREATE POLICY tenant_isolation ON featureflag.feature_flags USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);` |

**CRIT-001 監査対応**: アプリケーション層の `WHERE tenant_id = $1` に加え、DB レベルの RLS を二重防御として設定する。
アプリケーションコードは全クエリの冒頭で `SELECT set_config('app.current_tenant_id', $1, true)` を呼び出す必要がある。

### 006_alter_tenant_id_to_text.up.sql
**目的**: tenant_id カラムの型を UUID から TEXT に変更し、全サービスで型を統一する（C-004 監査対応・ADR-0093）

| 変更内容 | 詳細 |
|---------|------|
| feature_flags.tenant_id 型変更 | `UUID` → `TEXT`（`USING tenant_id::TEXT`）。既存の UUID 値は文字列として透過的に保持される |
| feature_flags.tenant_id デフォルト変更 | UUID 定数 → `'system'`（他サービスと統一） |
| flag_audit_logs.tenant_id 型変更 | `UUID` → `TEXT`（`USING tenant_id::TEXT`） |
| flag_audit_logs.tenant_id デフォルト変更 | UUID 定数 → `'system'`（他サービスと統一） |

**C-004 監査対応**: `current_setting('app.current_tenant_id', true)` が `TEXT` を返すため、`tenant_id` も `TEXT` 型に統一することで、RLS ポリシーの `::TEXT` キャストが理論的に不要になる。後方互換性のため `005_add_rls.up.sql` の既存ポリシーのキャストはそのまま残す。

---

## updated_at 自動更新トリガー

```sql
CREATE OR REPLACE FUNCTION featureflag.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_feature_flags_update_updated_at
    BEFORE UPDATE ON featureflag.feature_flags
    FOR EACH ROW EXECUTE FUNCTION featureflag.update_updated_at();
```

---

## テナント分離設計（STATIC-CRITICAL-001）

- 全テーブルに `tenant_id TEXT NOT NULL` カラムを設け、テナント境界を DB レベルで強制する（C-004 対応により UUID→TEXT 変更済み・ADR-0093）
- リポジトリ実装の全クエリに `WHERE tenant_id = $X` を付与する
- システムテナントデフォルト値: `'system'`（006 マイグレーション以降。旧: UUID `00000000-0000-0000-0000-000000000001`）
- キャッシュキー形式: `{tenant_id}:{flag_key}`（テナント間のキャッシュ汚染を防ぐ）
