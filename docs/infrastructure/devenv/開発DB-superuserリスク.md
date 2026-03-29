# 開発 DB の superuser リスクと管理方針（L-01 監査対応）

## 概要

ローカル開発環境（`docker-compose.dev.yaml`）では PostgreSQL に `dev/dev` の
superuser アカウントを使用している。本ドキュメントはそのリスクと管理方針を明記する。

## 現状

```yaml
# docker-compose.dev.yaml — 開発環境専用
postgres:
  environment:
    POSTGRES_USER: dev
    POSTGRES_PASSWORD: dev
    POSTGRES_DB: dev
```

開発環境の各サービスは `postgres://dev:dev@postgres:5432/<db>?sslmode=disable` で接続しており、
`dev` ユーザーは PostgreSQL の **superuser** 権限を持つ。

## superuser のリスク

| リスク | 内容 |
|--------|------|
| 全 DB へのアクセス | 他サービスのデータベースに自由にアクセス・変更が可能 |
| スキーマ変更 | `DROP TABLE` や `ALTER SYSTEM` 等の破壊的操作が可能 |
| ロール管理 | 他のユーザー・ロールの作成・削除・権限変更が可能 |
| 拡張機能のインストール | pg_exec 等の危険な拡張をインストール可能 |
| ログ汚染 | 誤ったクエリが全 DB に影響し、デバッグが困難になる |

## なぜ開発環境で superuser を使うのか

ローカル開発では以下の操作を頻繁に行うため、superuser が便利である：

- `cargo sqlx migrate run` によるマイグレーション（DDL 実行）
- `CREATE EXTENSION` による pgcrypto 等の拡張追加
- 複数 DB を横断したデバッグ・データ確認

開発効率を優先し、superuser による単純化を選択している。

## 本番環境での対応（ADR-0027 準拠）

本番環境では最小権限ロールを使用する（ADR-0027 参照）。

```sql
-- 本番環境のアプリケーションユーザー例
CREATE ROLE auth_app LOGIN PASSWORD '${VAULT_SECRET}';
GRANT CONNECT ON DATABASE auth_db TO auth_app;
GRANT USAGE ON SCHEMA auth TO auth_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA auth TO auth_app;
-- DDL 権限は付与しない（マイグレーションは別ロールで実施）
```

| 環境 | DB ユーザー | 権限 |
|------|-----------|------|
| ローカル開発 | `dev` (superuser) | 全権限（利便性優先） |
| CI | `dev` (superuser) | 全権限（短命コンテナ） |
| verify (k8s) | `dev` (superuser) | verify環境専用（外部非公開） |
| ステージング | アプリロール | 最小権限（スキーマ別 SELECT/INSERT/UPDATE/DELETE） |
| 本番 | アプリロール | 最小権限 + Vault 動的シークレット |

## 開発者向け注意事項

1. **dev 認証情報を本番 config.yaml や k8s Secret にコピーしないこと**
2. **`docker-compose.dev.yaml` を本番クラスターに apply しないこと**
3. **ローカル環境でデータを誤って消した場合は `just local-reset` でリセット可能**
4. **本番のマイグレーションは `migration` ロール（DDL のみ）で実施すること**

## 参考

- [ADR-0027: DB アプリユーザーロール分離](../../architecture/adr/0027-db-app-user-role-separation.md)
- [docker-compose.dev.yaml](../../../docker-compose.dev.yaml)
- 外部技術監査報告書 L-01: "開発環境の PostgreSQL superuser リスクの説明が不足"

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-28 | 初版作成（L-01 監査対応） | 監査対応チーム |
