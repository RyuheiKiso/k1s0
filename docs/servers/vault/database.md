# system-vault-server データベース設計

## スキーマ

スキーマ名: `vault`

```sql
CREATE SCHEMA IF NOT EXISTS vault;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| secrets | シークレット定義 |
| secret_versions | シークレットバージョン（暗号化データ） |
| access_logs | アクセスログ |
| access_policies | アクセスポリシー |

---

## ER 図

```
secrets 1──* secret_versions
access_logs (secret_id で論理的に紐付け、FK なし)
access_policies（単独テーブル、FK なし）
```

---

## テーブル定義

### secrets（シークレット）

シークレットのキーパスとメタデータを管理する。実際の暗号化データは secret_versions に格納する。

```sql
CREATE TABLE IF NOT EXISTS vault.secrets (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    key_path        VARCHAR(512) NOT NULL UNIQUE,
    current_version INT          NOT NULL DEFAULT 1,
    metadata        JSONB        NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_secrets_key_path ON vault.secrets (key_path);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| key_path | VARCHAR(512) | UNIQUE, NOT NULL | キーパス（階層構造） |
| current_version | INT | NOT NULL, DEFAULT 1 | 現在のバージョン番号 |
| metadata | JSONB | NOT NULL, DEFAULT '{}' | メタデータ |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

### secret_versions（シークレットバージョン）

シークレットの暗号化データをバージョンごとに保持する。encrypted_data と nonce で AES-GCM 暗号化を行う。

```sql
CREATE TABLE IF NOT EXISTS vault.secret_versions (
    id             UUID    PRIMARY KEY DEFAULT gen_random_uuid(),
    secret_id      UUID    NOT NULL REFERENCES vault.secrets(id) ON DELETE CASCADE,
    version        INT     NOT NULL,
    encrypted_data BYTEA   NOT NULL,
    nonce          BYTEA   NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_secret_versions_secret_version UNIQUE (secret_id, version)
);

CREATE INDEX IF NOT EXISTS idx_secret_versions_secret_id ON vault.secret_versions (secret_id);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| secret_id | UUID | FK → secrets.id, NOT NULL | シークレット ID |
| version | INT | UNIQUE(secret_id, version), NOT NULL | バージョン番号 |
| encrypted_data | BYTEA | NOT NULL | 暗号化データ |
| nonce | BYTEA | NOT NULL | 暗号化ナンス |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |

---

### access_logs（アクセスログ）

シークレットへのアクセス履歴を記録する。成功・失敗の両方を記録する。

```sql
CREATE TABLE IF NOT EXISTS vault.access_logs (
    id         UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    secret_id  UUID,
    key_path   VARCHAR(512) NOT NULL,
    action     VARCHAR(50)  NOT NULL,
    actor_id   VARCHAR(255) NOT NULL DEFAULT '',
    ip_address VARCHAR(45),
    success    BOOLEAN      NOT NULL DEFAULT true,
    error_msg  TEXT,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_access_logs_action CHECK (action IN ('read', 'write', 'delete', 'list'))
);

CREATE INDEX IF NOT EXISTS idx_access_logs_key_path ON vault.access_logs (key_path);
CREATE INDEX IF NOT EXISTS idx_access_logs_created_at ON vault.access_logs (created_at DESC);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| secret_id | UUID | | シークレット ID（削除済みの場合 NULL） |
| key_path | VARCHAR(512) | NOT NULL | キーパス |
| action | VARCHAR(50) | NOT NULL | アクション（read/write/delete/list） |
| actor_id | VARCHAR(255) | NOT NULL, DEFAULT '' | 操作者 ID |
| ip_address | VARCHAR(45) | | IP アドレス |
| success | BOOLEAN | NOT NULL, DEFAULT true | 成功フラグ |
| error_msg | TEXT | | エラーメッセージ |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 記録日時 |

---

### access_policies（アクセスポリシー）

SPIFFE ID ベースのシークレットアクセスポリシーを定義する。パスパターンで対象シークレットを指定する。

```sql
CREATE TABLE IF NOT EXISTS vault.access_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    secret_path_pattern VARCHAR(1024) NOT NULL,
    allowed_spiffe_ids TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_access_policies_path ON vault.access_policies (secret_path_pattern);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| secret_path_pattern | VARCHAR(1024) | NOT NULL | シークレットパスパターン |
| allowed_spiffe_ids | TEXT[] | NOT NULL, DEFAULT '{}' | 許可 SPIFFE ID 一覧 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |

---

## マイグレーション

マイグレーションファイルは `regions/system/database/vault-db/migrations/` に配置。

| ファイル | 内容 |
| --- | --- |
| `001_create_schema.up.sql` | `vault` スキーマ・pgcrypto 拡張・updated_at 関数作成 |
| `001_create_schema.down.sql` | スキーマ削除 |
| `002_create_secrets.up.sql` | secrets テーブル作成 |
| `002_create_secrets.down.sql` | テーブル削除 |
| `003_create_secret_versions.up.sql` | secret_versions テーブル作成 |
| `003_create_secret_versions.down.sql` | テーブル削除 |
| `004_create_access_logs.up.sql` | access_logs テーブル作成 |
| `004_create_access_logs.down.sql` | テーブル削除 |
| `005_create_access_policies.up.sql` | access_policies テーブル作成 |
| `005_create_access_policies.down.sql` | テーブル削除 |

---

## updated_at 自動更新トリガー

```sql
CREATE OR REPLACE FUNCTION vault.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_secrets_update_updated_at
    BEFORE UPDATE ON vault.secrets
    FOR EACH ROW EXECUTE FUNCTION vault.update_updated_at();
```
