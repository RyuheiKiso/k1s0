# system-database設計

> **ガイド**: 設計背景・マイグレーション SQL・クエリパターンは [database.guide.md](./database.guide.md) を参照。

system Tier の認証・認可データベース（auth-db）の設計仕様。配置先: `regions/system/database/auth-db/`

## 概要

auth-db は PostgreSQL 17 データベースであり、Keycloak が管理する認証情報とは独立したアプリケーション固有データ（ユーザープロフィール、RBAC、監査ログ、API キー）を保持する。

[tier-architecture.md](../../architecture/overview/tier-architecture.md) の設計原則に従い、auth-db へのアクセスは **system Tier のサーバーからのみ** 許可する。

### 技術スタック

| コンポーネント | 技術 | バージョン |
|----------------|------|------------|
| RDBMS | PostgreSQL | 17 |
| マイグレーション（Go） | golang-migrate | - |
| マイグレーション（Rust） | sqlx-cli | - |
| ORM / クエリビルダー | sqlx（Go / Rust 共通） | - |
| シークレット管理 | HashiCorp Vault | 1.17 |

### Keycloak DB との役割分担

| データ | 管理先 | 理由 |
|--------|--------|------|
| ユーザー認証情報（パスワード等） | Keycloak DB | Keycloak が管理する認証基盤のデータ |
| OAuth2 クライアント設定 | Keycloak DB | Keycloak が管理するプロトコル設定 |
| LDAP / AD 連携設定 | Keycloak DB | Keycloak の User Federation 機能 |
| セッション管理 | Redis Sentinel | BFF セッションストア（[認証認可設計](../../architecture/auth/認証認可設計.md) 参照） |
| ユーザープロフィール（アプリ固有） | auth-db | Keycloak の sub と紐づくアプリケーション固有データ |
| ロール・権限マッピング（アプリ固有） | auth-db | [認証認可設計](../../architecture/auth/認証認可設計.md) D-005 の細粒度 RBAC |
| 監査ログ | auth-db | 長期保存・全文検索・コンプライアンス対応 |
| API キー | auth-db | サービス間認証のフォールバック |

---

## ER図

```
┌─────────────┐       ┌──────────────────┐       ┌─────────────────┐
│   users     │       │   user_roles     │       │     roles       │
├─────────────┤       ├──────────────────┤       ├─────────────────┤
│ id (PK)     │──┐    │ id (PK)          │    ┌──│ id (PK)         │
│ keycloak_sub│  └───>│ user_id (FK)     │    │  │ name            │
│ username    │       │ role_id (FK)     │<───┘  │ description     │
│ email       │       │ assigned_by      │       │ tier            │
│ display_name│       │ assigned_at      │       │ created_at      │
│ status      │       └──────────────────┘       └─────────────────┘
│ created_at  │                                          │
│ updated_at  │                                          │
└─────────────┘       ┌──────────────────┐               │
       │              │ role_permissions  │               │
       │              ├──────────────────┤       ┌───────┘
       │              │ id (PK)          │       │
       │              │ role_id (FK)     │<──────┘
       │              │ permission_id(FK)│───┐
       │              │ granted_at       │   │
       │              └──────────────────┘   │
       │                                     │
       │              ┌──────────────────┐   │
       │              │  permissions     │   │
       │              ├──────────────────┤   │
       │              │ id (PK)          │<──┘
       │              │ resource         │
       │              │ action           │
       │              │ description      │
       │              └──────────────────┘
       │
       │              ┌──────────────────────┐
       │              │  audit_logs          │
       │              │  (user_id は TEXT、   │
       │              │   FK なし)            │
       │              ├──────────────────────┤
       │              │ id (PK composite)    │
       │              │ user_id (TEXT)       │
       │              │ event_type           │
       │              │ action               │
       │              │ resource             │
       │              │ resource_id          │
       │              │ result               │
       │              │ detail (JSONB)       │
       │              │ ip_address (TEXT)    │
       │              │ user_agent           │
       │              │ trace_id             │
       │              │ created_at (PK comp) │
       │              └──────────────────────┘

┌─────────────────┐
│   api_keys      │
├─────────────────┤
│ id (PK)         │
│ name            │
│ key_hash        │
│ key_prefix      │
│ service_name    │
│ tier            │
│ permissions     │
│ expires_at      │
│ last_used_at    │
│ is_active       │
│ created_by (FK) │──> users(id)
│ created_at      │
│ updated_at      │
└─────────────────┘
```

### リレーション

| 関係 | カーディナリティ | 説明 |
|------|-----------------|------|
| users - user_roles | 1:N | ユーザーは複数のロールを持てる |
| roles - user_roles | 1:N | ロールは複数のユーザーに割り当てられる |
| roles - role_permissions | 1:N | ロールは複数の権限を持てる |
| permissions - role_permissions | 1:N | 権限は複数のロールに付与される |
| users - audit_logs | 1:N（論理的。FK なし） | ユーザーは複数の監査ログを生成する（user_id は TEXT 型で FK 制約なし） |
| users - api_keys | 1:N | ユーザーは複数の API キーを作成できる |

---

## テーブル定義

### users テーブル

Keycloak の `sub` claim（UUID）と紐づくアプリケーション固有ユーザーデータ。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | アプリケーション内部の識別子 |
| keycloak_sub | VARCHAR(255) | UNIQUE NOT NULL | Keycloak の sub claim（ユーザーの一意識別子） |
| username | VARCHAR(255) | UNIQUE NOT NULL | ログイン ID（Keycloak の preferred_username と同期） |
| email | VARCHAR(255) | UNIQUE NOT NULL | メールアドレス（Keycloak の email と同期） |
| display_name | VARCHAR(255) | NOT NULL | 表示名 |
| status | VARCHAR(50) | NOT NULL DEFAULT 'active' | active / inactive / suspended |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 更新日時（トリガーで自動更新） |

### roles テーブル

[認証認可設計](../../architecture/auth/認証認可設計.md) D-005 の Tier 別ロール定義。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | ロール識別子 |
| name | VARCHAR(100) | UNIQUE NOT NULL | ロール名（例: sys_admin, svc_order_user） |
| description | TEXT | | ロールの説明 |
| tier | VARCHAR(20) | NOT NULL | system / business / service |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |

### permissions テーブル

リソースに対する操作権限。[認証認可設計](../../architecture/auth/認証認可設計.md) D-005 のパーミッションマトリクスに対応。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | 権限識別子 |
| resource | VARCHAR(255) | NOT NULL | 操作対象リソース（例: orders, users, config） |
| action | VARCHAR(50) | NOT NULL | 操作種別（read, write, delete, admin） |
| description | TEXT | | 権限の説明 |
| | | UNIQUE(resource, action) | リソースと操作の組み合わせで一意 |

### user_roles テーブル（多対多）

ユーザーとロールの割り当て中間テーブル。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | レコード識別子 |
| user_id | UUID | FK users(id) ON DELETE CASCADE, NOT NULL | ユーザー ID |
| role_id | UUID | FK roles(id) ON DELETE CASCADE, NOT NULL | ロール ID |
| assigned_by | UUID | FK users(id) ON DELETE SET NULL | 割り当てた管理者の ID |
| assigned_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 割り当て日時 |
| | | UNIQUE(user_id, role_id) | 同一ユーザーに同一ロールは1回のみ |

### role_permissions テーブル（多対多）

ロールと権限の関連付け中間テーブル。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | レコード識別子 |
| role_id | UUID | FK roles(id) ON DELETE CASCADE, NOT NULL | ロール ID |
| permission_id | UUID | FK permissions(id) ON DELETE CASCADE, NOT NULL | 権限 ID |
| granted_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 付与日時 |
| | | UNIQUE(role_id, permission_id) | 同一ロールに同一権限は1回のみ |

### audit_logs テーブル

操作履歴の監査ログ。月次パーティショニング、OpenTelemetry trace_id 連携。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | NOT NULL, DEFAULT gen_random_uuid() | ログ識別子 |
| user_id | TEXT | | 操作ユーザーの ID（文字列型。FK なし） |
| event_type | VARCHAR(100) | NOT NULL | イベント種別（LOGIN_SUCCESS, LOGIN_FAILURE, LOGOUT, PERMISSION_CHANGE 等） |
| action | VARCHAR(100) | NOT NULL | 操作種別（login, logout, permission_change, user_create, user_update, role_assign, api_key_create 等） |
| resource | VARCHAR(255) | | 操作対象リソース種別 |
| resource_id | VARCHAR(255) | | 操作対象リソースの ID |
| result | VARCHAR(50) | NOT NULL DEFAULT 'SUCCESS' | 操作結果（SUCCESS, FAILURE, DENIED） |
| detail | JSONB | | 操作の詳細情報（変更前後の値等） |
| ip_address | TEXT | | クライアント IP アドレス（TEXT 型で IPv4/IPv6 文字列を柔軟に格納） |
| user_agent | TEXT | | クライアント User-Agent |
| trace_id | VARCHAR(64) | | OpenTelemetry トレース ID |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 記録日時 |
| | | PRIMARY KEY (id, created_at) | パーティションキーを含む複合主キー |

> **注意**: パーティションキー（`created_at`）を PRIMARY KEY に含める必要があるため複合主キーを使用。`user_id` は `TEXT` 型（FK なし）、`ip_address` は `TEXT` 型（INET 型ではない）。

**インデックス**: `(user_id, created_at)`, `(event_type, created_at)`, `(action, created_at)`, `(trace_id WHERE NOT NULL)`, `(resource, resource_id WHERE NOT NULL)`

### api_keys テーブル

サービス間認証フォールバック用 API キー。mTLS + Client Credentials が利用できない場合の代替手段。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | API キー識別子 |
| name | VARCHAR(255) | NOT NULL | API キーの名前（識別用） |
| key_hash | VARCHAR(255) | UNIQUE NOT NULL | API キーの SHA-256 ハッシュ |
| key_prefix | VARCHAR(10) | NOT NULL | キーの先頭8文字（識別・ログ表示用） |
| service_name | VARCHAR(255) | NOT NULL | 使用するサービス名 |
| tier | VARCHAR(20) | NOT NULL | system / business / service |
| permissions | JSONB | NOT NULL DEFAULT '[]' | 許可する操作の一覧（JSON 配列） |
| expires_at | TIMESTAMPTZ | | 有効期限（NULL の場合は無期限） |
| last_used_at | TIMESTAMPTZ | | 最終使用日時 |
| is_active | BOOLEAN | NOT NULL DEFAULT true | 有効フラグ |
| created_by | UUID | FK users(id) ON DELETE SET NULL | 作成者の ID |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 更新日時 |

---

## マイグレーションファイル

配置先: `regions/system/database/auth-db/migrations/`

命名規則は [テンプレート仕様-データベース](../../templates/data/データベース.md) に準拠。SQL は [ガイド](./database.guide.md#マイグレーション-sql) を参照。

```
migrations/
├── 001_create_schema.up.sql                            # スキーマ・拡張機能・共通関数
├── 001_create_schema.down.sql
├── 002_create_users.up.sql                             # users テーブル
├── 002_create_users.down.sql
├── 003_create_roles.up.sql                             # roles テーブル
├── 003_create_roles.down.sql
├── 004_create_permissions.up.sql                       # permissions テーブル
├── 004_create_permissions.down.sql
├── 005_create_user_roles_and_role_permissions.up.sql   # user_roles + role_permissions 中間テーブル
├── 005_create_user_roles_and_role_permissions.down.sql
├── 006_create_audit_logs.up.sql                        # audit_logs テーブル（月次パーティション）
├── 006_create_audit_logs.down.sql
├── 007_create_api_keys.up.sql                          # api_keys テーブル
├── 007_create_api_keys.down.sql
├── 008_seed_initial_data.up.sql                        # 初期データ投入（ロール・権限・マッピング）
├── 008_seed_initial_data.down.sql
├── 009_align_audit_log_columns.up.sql                  # audit_logs カラム名変更（detail→metadata, created_at→recorded_at）
├── 009_align_audit_log_columns.down.sql
├── 010_fix_audit_log_columns.up.sql                    # audit_logs カラム名を正規設計に戻す（metadata→detail, recorded_at→created_at）
├── 010_fix_audit_log_columns.down.sql
├── 011_create_partition_management.up.sql              # pg_partman による自動パーティション管理
└── 011_create_partition_management.down.sql
```

---

## インデックス設計

インデックス設計方針は [ガイド](./database.guide.md#インデックス設計方針) を参照。

### インデックス一覧

| テーブル | インデックス名 | カラム | 種別 | 用途 |
|----------|---------------|--------|------|------|
| users | idx_users_keycloak_sub | keycloak_sub | B-tree | JWT の sub からのユーザー検索 |
| users | idx_users_status | status | B-tree | ステータスによるフィルタリング |
| users | idx_users_created_at | created_at | B-tree | 作成日時による範囲検索・ソート |
| roles | idx_roles_tier | tier | B-tree | Tier によるロール検索 |
| roles | idx_roles_name | name | B-tree | ロール名による検索 |
| permissions | idx_permissions_resource | resource | B-tree | リソースによる権限検索 |
| user_roles | idx_user_roles_user_id | user_id | B-tree | ユーザーに紐づくロール取得 |
| user_roles | idx_user_roles_role_id | role_id | B-tree | ロールに紐づくユーザー取得 |
| role_permissions | idx_role_permissions_role_id | role_id | B-tree | ロールに紐づく権限取得 |
| role_permissions | idx_role_permissions_permission_id | permission_id | B-tree | 権限に紐づくロール取得 |
| audit_logs | idx_audit_logs_user_id_created_at | (user_id, created_at) | B-tree | ユーザー別の監査ログ時系列検索 |
| audit_logs | idx_audit_logs_event_type_created_at | (event_type, created_at) | B-tree | イベント種別別の監査ログ時系列検索 |
| audit_logs | idx_audit_logs_action_created_at | (action, created_at) | B-tree | 操作種別別の監査ログ時系列検索 |
| audit_logs | idx_audit_logs_trace_id | trace_id (WHERE NOT NULL) | B-tree (部分) | OpenTelemetry トレース ID による検索 |
| audit_logs | idx_audit_logs_resource | (resource, resource_id) (WHERE NOT NULL) | B-tree (部分) | リソース種別・ID による検索 |
| api_keys | idx_api_keys_key_hash | key_hash (WHERE is_active) | B-tree (部分) | API キー認証時のハッシュ検索（アクティブのみ） |
| api_keys | idx_api_keys_service_name | service_name | B-tree | サービス名による API キー検索 |
| api_keys | idx_api_keys_key_prefix | key_prefix | B-tree | プレフィックスによる API キー識別 |
| api_keys | idx_api_keys_expires_at | expires_at (WHERE NOT NULL) | B-tree (部分) | 期限切れ API キーの検出 |

---

## パーティショニング

audit_logs テーブルに月次レンジパーティショニングを適用。パーティション作成例・アーカイブ手順は [ガイド](./database.guide.md#パーティショニング設計背景) を参照。

### パーティション運用ルール

| 項目 | ルール |
|------|--------|
| パーティション単位 | 月次（1ヶ月 = 1パーティション） |
| 事前作成 | 3ヶ月先のパーティションを常に準備 |
| 保持期間 | 24ヶ月（コンプライアンス要件に応じて調整） |
| アーカイブ | 保持期間超過のパーティションは DETACH 後にアーカイブストレージへ移動 |
| デフォルトパーティション | 範囲外データの受け皿として常に存在させる |
| 自動化 | pg_partman または cron ジョブでパーティション作成・削除を自動化 |

---

## 接続設定

[config設計](../../cli/config/config設計.md) の database セクション準拠。config.yaml 例は [ガイド](./database.guide.md#接続設定の実装例) を参照。

### 環境別設定

| 環境 | host | ssl_mode | max_open_conns | max_idle_conns |
|------|------|----------|----------------|----------------|
| dev | localhost (docker-compose) | disable | 10 | 3 |
| staging | postgres.k1s0-system.svc.cluster.local | require | 25 | 5 |
| prod | postgres.k1s0-system.svc.cluster.local | verify-full | 50 | 10 |

### Vault によるクレデンシャル管理

[認証認可設計](../../architecture/auth/認証認可設計.md) D-006 のシークレットパス体系に従う。

| 用途 | Vault パス | 説明 |
|------|-----------|------|
| 静的パスワード | `secret/data/k1s0/system/auth/database` | キー: `password` |
| 動的クレデンシャル（読み書き） | `database/creds/system-auth-rw` | Vault Database エンジンで自動生成（TTL: 24時間） |
| 動的クレデンシャル（読み取り専用） | `database/creds/system-auth-ro` | Vault Database エンジンで自動生成（TTL: 24時間） |

---

## スキーマ定義ファイル

参照用スキーマ定義の配置先。マイグレーションファイルが正。

```
schema/
├── auth.sql              # 全テーブルの CREATE 文を統合したリファレンス
└── er-diagram.md         # ER 図（テキスト形式）
```

---

## バックアップ・リストア

バックアップ実行例は [ガイド](./database.guide.md#バックアップリストア) を参照。

### バックアップ方針

| 項目 | 値 |
|------|-----|
| フルバックアップ | 毎日深夜（0:00） |
| WAL アーカイブ | 継続的（PITR 対応） |
| バックアップ先 | Ceph オブジェクトストレージ |
| 保持期間 | フルバックアップ: 30日、WAL: 7日 |
| リストアテスト | 月次で staging 環境にリストアし検証 |

---

## 関連ドキュメント

- [database.guide.md](./database.guide.md) -- 設計背景・マイグレーション SQL・クエリパターン・バックアップ手順
- [tier-architecture](../../architecture/overview/tier-architecture.md) -- Tier アーキテクチャ・データベースアクセスルール
- [認証認可設計](../../architecture/auth/認証認可設計.md) -- OAuth 2.0 / OIDC・RBAC・Vault シークレット管理
- [config設計](../../cli/config/config設計.md) -- config.yaml スキーマ（database セクション）
- [テンプレート仕様-データベース](../../templates/data/データベース.md) -- マイグレーション命名規則・テンプレート
- [コンセプト](../../architecture/overview/コンセプト.md) -- 技術スタック（PostgreSQL 17・sqlx）
- [ディレクトリ構成図](../../architecture/overview/ディレクトリ構成図.md) -- データベース配置先ディレクトリ
- [docker-compose設計](../../infrastructure/docker/docker-compose設計.md) -- ローカル開発用 PostgreSQL
- [可観測性設計](../../architecture/observability/可観測性設計.md) -- OpenTelemetry トレース ID 連携
- [kubernetes設計](../../infrastructure/kubernetes/kubernetes設計.md) -- Namespace・PVC 設計
- [helm設計](../../infrastructure/kubernetes/helm設計.md) -- PostgreSQL Helm Chart・Vault Agent Injector
