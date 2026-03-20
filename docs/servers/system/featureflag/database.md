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

---

## ER 図

```
feature_flags（単独テーブル、FK なし）
```

---

## テーブル定義

### feature_flags（フィーチャーフラグ定義）

フィーチャーフラグのキー・有効状態・バリアント・ルールを管理する。

```sql
CREATE TABLE IF NOT EXISTS featureflag.feature_flags (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    flag_key    VARCHAR(255) NOT NULL UNIQUE,
    description TEXT         NOT NULL DEFAULT '',
    enabled     BOOLEAN      NOT NULL DEFAULT false,
    variants    JSONB        NOT NULL DEFAULT '[]',
    rules       JSONB        NOT NULL DEFAULT '[]',
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_feature_flags_flag_key ON featureflag.feature_flags (flag_key);
CREATE INDEX IF NOT EXISTS idx_feature_flags_enabled ON featureflag.feature_flags (enabled);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| flag_key | VARCHAR(255) | UNIQUE, NOT NULL | フラグキー（一意識別子） |
| description | TEXT | NOT NULL, DEFAULT '' | 説明 |
| enabled | BOOLEAN | NOT NULL, DEFAULT false | 有効フラグ |
| variants | JSONB | NOT NULL, DEFAULT '[]' | バリアント定義（A/B テスト等） |
| rules | JSONB | NOT NULL, DEFAULT '[]' | 評価ルール定義 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

## マイグレーション

マイグレーションファイルは `regions/system/database/featureflag-db/migrations/` に配置。

| ファイル | 内容 |
| --- | --- |
| `001_create_schema.up.sql` | `featureflag` スキーマ・pgcrypto 拡張・updated_at 関数作成 |
| `001_create_schema.down.sql` | スキーマ削除 |
| `002_create_feature_flags.up.sql` | feature_flags テーブル作成 |
| `002_create_feature_flags.down.sql` | テーブル削除 |

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
