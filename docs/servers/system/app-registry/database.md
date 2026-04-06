# system-app-registry-server データベース設計

## スキーマ

スキーマ名: `app_registry`

```sql
CREATE SCHEMA IF NOT EXISTS app_registry;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| apps | アプリケーション定義 |
| app_versions | アプリバージョン（プラットフォーム・アーキテクチャ別） |
| download_stats | ダウンロード統計 |

---

## ER 図

```
apps 1──* app_versions (app_id)
apps 1──* download_stats (app_id)
```

---

## テーブル定義

### apps（アプリケーション）

アプリケーションの基本情報を管理する。

```sql
CREATE TABLE IF NOT EXISTS app_registry.apps (
    id          TEXT         PRIMARY KEY,
    name        VARCHAR(255) NOT NULL,
    description TEXT,
    category    VARCHAR(100),
    icon_url    TEXT,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_apps_category ON app_registry.apps (category);
CREATE INDEX IF NOT EXISTS idx_apps_name ON app_registry.apps (name);
CREATE INDEX IF NOT EXISTS idx_apps_created_at ON app_registry.apps (created_at);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | TEXT | PK | アプリ ID（一意識別子） |
| name | VARCHAR(255) | NOT NULL | アプリ名 |
| description | TEXT | | 説明 |
| category | VARCHAR(100) | | カテゴリ |
| icon_url | TEXT | | アイコン URL |
| tenant_id | TEXT | NOT NULL | テナント識別子（ADR-0093 準拠） |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

**Row Level Security:**

```sql
ALTER TABLE app_registry.apps ENABLE ROW LEVEL SECURITY;
ALTER TABLE app_registry.apps FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON app_registry.apps
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));
```

---

### app_versions（アプリバージョン）

アプリケーションのバージョン情報をプラットフォーム・アーキテクチャ別に管理する。

```sql
CREATE TABLE IF NOT EXISTS app_registry.app_versions (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    app_id          TEXT         NOT NULL REFERENCES app_registry.apps(id) ON DELETE CASCADE,
    version         VARCHAR(50)  NOT NULL,
    platform        VARCHAR(20)  NOT NULL,
    arch            VARCHAR(20)  NOT NULL,
    size_bytes      BIGINT       NOT NULL,
    checksum_sha256 VARCHAR(64)  NOT NULL,
    storage_key     TEXT         NOT NULL,
    release_notes    TEXT,
    mandatory        BOOLEAN      NOT NULL DEFAULT false,
    -- STATIC-CRITICAL-002: Cosign 署名（base64）。NULL は未署名（開発環境）または署名なし。
    cosign_signature TEXT,
    published_at     TIMESTAMPTZ,
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_app_versions_app_version_platform_arch
        UNIQUE (app_id, version, platform, arch),
    CONSTRAINT chk_app_versions_platform
        CHECK (platform IN ('windows', 'linux', 'macos'))
);

CREATE INDEX IF NOT EXISTS idx_app_versions_app_id ON app_registry.app_versions (app_id);
CREATE INDEX IF NOT EXISTS idx_app_versions_version ON app_registry.app_versions (app_id, version);
CREATE INDEX IF NOT EXISTS idx_app_versions_platform ON app_registry.app_versions (platform, arch);
CREATE INDEX IF NOT EXISTS idx_app_versions_published_at ON app_registry.app_versions (published_at DESC);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | バージョン ID |
| app_id | TEXT | FK → apps.id, NOT NULL | アプリ ID |
| version | VARCHAR(50) | UNIQUE(app_id, version, platform, arch), NOT NULL | バージョン文字列 |
| platform | VARCHAR(20) | UNIQUE(app_id, version, platform, arch), NOT NULL | プラットフォーム（windows/linux/macos） |
| arch | VARCHAR(20) | UNIQUE(app_id, version, platform, arch), NOT NULL | アーキテクチャ |
| size_bytes | BIGINT | NOT NULL | ファイルサイズ（バイト） |
| checksum_sha256 | VARCHAR(64) | NOT NULL | SHA-256 チェックサム |
| storage_key | TEXT | NOT NULL | サーバー上のファイル保存パス（例: `app-id/1.0.0/windows-x64/app.exe`） |
| release_notes | TEXT | | リリースノート |
| mandatory | BOOLEAN | NOT NULL, DEFAULT false | 強制アップデートフラグ |
| cosign_signature | TEXT | NULL 許可 | Cosign 署名（base64）。STATIC-CRITICAL-002 対応。 |
| published_at | TIMESTAMPTZ | | 公開日時 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |

---

### download_stats（ダウンロード統計）

アプリケーションのダウンロードイベントを記録する。

```sql
CREATE TABLE IF NOT EXISTS app_registry.download_stats (
    id            UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    app_id        TEXT         NOT NULL REFERENCES app_registry.apps(id) ON DELETE CASCADE,
    version       VARCHAR(50)  NOT NULL,
    platform      VARCHAR(20)  NOT NULL,
    user_id       TEXT,
    downloaded_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_download_stats_app_id ON app_registry.download_stats (app_id);
CREATE INDEX IF NOT EXISTS idx_download_stats_app_version ON app_registry.download_stats (app_id, version);
CREATE INDEX IF NOT EXISTS idx_download_stats_downloaded_at ON app_registry.download_stats (downloaded_at);
CREATE INDEX IF NOT EXISTS idx_download_stats_user_id ON app_registry.download_stats (user_id) WHERE user_id IS NOT NULL;
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 統計 ID |
| app_id | TEXT | FK → apps.id, NOT NULL | アプリ ID |
| version | VARCHAR(50) | NOT NULL | バージョン文字列 |
| platform | VARCHAR(20) | NOT NULL | プラットフォーム |
| user_id | TEXT | | ダウンロードユーザー ID |
| downloaded_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | ダウンロード日時 |

---

## マイグレーション

マイグレーションファイルは `regions/system/database/app-registry-db/migrations/` に配置。

| ファイル | 内容 |
| --- | --- |
| `001_create_schema.up.sql` | `app_registry` スキーマ・pgcrypto 拡張・updated_at 関数作成 |
| `001_create_schema.down.sql` | スキーマ削除 |
| `002_create_apps.up.sql` | apps テーブル作成 |
| `002_create_apps.down.sql` | テーブル削除 |
| `003_create_app_versions.up.sql` | app_versions テーブル作成 |
| `003_create_app_versions.down.sql` | テーブル削除 |
| `004_create_download_stats.up.sql` | download_stats テーブル作成 |
| `004_create_download_stats.down.sql` | テーブル削除 |
| `005_seed_initial_data.up.sql` | 初期アプリデータ投入 |
| `005_seed_initial_data.down.sql` | 初期データ削除 |
| `006_rename_s3_key_to_storage_key.up.sql` | `s3_key` → `storage_key` カラムリネーム |
| `006_rename_s3_key_to_storage_key.down.sql` | `storage_key` → `s3_key` ロールバック |
| `007_add_cosign_signature.up.sql` | `app_versions` に `cosign_signature TEXT` カラム追加（STATIC-CRITICAL-002） |
| `007_add_cosign_signature.down.sql` | `cosign_signature` カラム削除 |
| `008_add_tenant_id_rls.up.sql` | `apps` / `app_versions` / `download_stats` に `tenant_id TEXT NOT NULL` と RLS ポリシー（FORCE / AS RESTRICTIVE / WITH CHECK）を追加（CRITICAL-DB-001 対応） |
| `008_add_tenant_id_rls.down.sql` | `tenant_id` カラムと RLS ポリシー削除 |

---

## updated_at 自動更新トリガー

```sql
CREATE OR REPLACE FUNCTION app_registry.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_apps_update_updated_at
    BEFORE UPDATE ON app_registry.apps
    FOR EACH ROW EXECUTE FUNCTION app_registry.update_updated_at();
```
