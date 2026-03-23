# データベース認証情報管理: スキーマ別ロールへの移行計画

## 現状と課題

現在の k1s0 データベース接続は全サービスが共通の管理者権限ユーザーを使用している。
これはサービス間の権限分離ができず、セキュリティリスクとなっている。

### 現状の問題点
- 全サービスが同一のDB認証情報を共有
- 一つのサービスの認証情報漏洩が全データに影響
- 最小権限の原則に違反

## 目標アーキテクチャ

### スキーマ別ロール分離

各サービスは専用のDBユーザー/ロールを持ち、自サービスのスキーマのみにアクセス可能とする。

```
PostgreSQL
├── schema: auth        → role: k1s0_auth_rw      (auth-server のみ)
├── schema: saga        → role: k1s0_saga_rw      (saga-server のみ)
├── schema: config      → role: k1s0_config_rw    (config-server のみ)
├── schema: tenant      → role: k1s0_tenant_rw    (tenant-server のみ)
└── schema: workflow    → role: k1s0_workflow_rw  (workflow-server のみ)
```

### Vault Database エンジンによる動的認証情報

HashiCorp Vault の Database エンジンを使用して、TTL付きの動的DB認証情報を発行する。

```
サービス → Vault (database/creds/k1s0-{service}-role)
Vault → PostgreSQL (一時ユーザー作成)
Vault → サービス (username/password, TTL: 1h)
サービス → PostgreSQL (一時認証情報で接続)
```

## 移行計画

### フェーズ 1: スキーマ分離（完了）
- `infra/terraform/modules/database/roles.tf` にサービス別ロール定義を追加済み
- 各スキーマへの GRANT 設定（SELECT, INSERT, UPDATE, DELETE のみ。DDL操作はmigrationユーザーのみ）
- migration用ユーザー（`k1s0_migration`）は全スキーマのDDL権限を保持
- 開発環境用スクリプト: `infra/docker/init-db/16-roles.sql`
- ロール関連 Terraform 変数: `infra/terraform/modules/database/variables.tf`

### フェーズ 2: Vault Database エンジン設定
- `infra/terraform/vault/database.tf` にDatabase エンジン設定を追加
- 各サービス用のVaultロール定義
- シークレット TTL: 1時間（起動時に取得、期限前に自動更新）

### フェーズ 3: サービス側の対応
- 各サービスの `startup.rs` / `main.go` を Vault SDK 経由での認証情報取得に変更
- `k1s0_vault_client` ライブラリで認証情報の自動更新を実装
- `DATABASE_URL` 環境変数から Vault Secret Path 環境変数への移行

### フェーズ 4: 静的認証情報の廃止
- 全サービスのVault移行確認後、静的DB認証情報を無効化
- Vault の Audit Log で全DB接続を監査可能な状態に

## ロール権限マトリクス

| ロール | SELECT | INSERT | UPDATE | DELETE | DDL | 対象スキーマ |
|--------|--------|--------|--------|--------|-----|-------------|
| k1s0_{svc}_rw | ✓ | ✓ | ✓ | ✓ | ✗ | {svc}のみ |
| k1s0_migration | ✓ | ✓ | ✓ | ✓ | ✓ | 全スキーマ |
| k1s0_readonly | ✓ | ✗ | ✗ | ✗ | ✗ | 全スキーマ（監査用）|

## セキュリティ上の考慮事項
- 静的パスワードは使用しない（Vault動的認証情報のみ）
- 認証情報はコンテナの環境変数に直接渡さない（Vault Agent Sidecar 経由）
- DB接続はPrivateネットワーク内のみ（PublicIPからのアクセス禁止）

## 参照
- `infra/terraform/modules/database/roles.tf` — サービス別ロール・GRANT定義（Terraform）
- `infra/terraform/modules/database/variables.tf` — ロール関連変数定義
- `infra/docker/init-db/16-roles.sql` — 開発環境用ロール作成スクリプト
- `infra/terraform/vault/` — Vault Terraform設定
- `docs/infrastructure/security/Vault設計.md` — Vault全体設計

## 更新履歴

| 日付 | 変更内容 |
|------|---------|
| 2026-03-21 | 初版作成（技術品質監査対応 P2-32） |
| 2026-03-24 | フェーズ 1 完了: roles.tf 新規作成、variables.tf 更新、開発環境用 SQL スクリプト追加（外部監査 C-02 対応） |
