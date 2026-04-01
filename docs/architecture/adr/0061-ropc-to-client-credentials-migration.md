# ADR-0061: Keycloak 管理認証を ROPC から Client Credentials Grant に移行する

## ステータス

承認済み（STATIC-MEDIUM-003 監査対応: ROPC は実装されていないことを確認。Client Credentials Grant のみを使用）

## コンテキスト

`auth-rust` サービスの `keycloak_client.rs` は Keycloak Admin API トークンの取得に
Resource Owner Password Credentials（ROPC / password grant）を使用している。
ROPC は `admin_username` と `admin_password` をアプリケーションが直接受け取るフローであり、
以下の問題を抱えている。

- **OAuth 2.1 廃止予定**: OAuth 2.1 草案（RFC 9126 参照）において ROPC は廃止対象として明示されている
- **フィッシングリスク**: ユーザー認証情報をアプリケーション経由で送信するため、
  認証情報が漏洩した場合の影響範囲が広い
- **長期的な互換性の低下**: IdP がOAuth 2.1 に準拠した場合、ROPC エンドポイントが
  無効化されるリスクがある

現状のコードでは `admin_username` / `admin_password` が未設定の場合に
Client Credentials Grant にフォールバックする分岐（`else` ブランチ）が実装済みである。
つまり、ROPC を廃止して Client Credentials Grant に統一するためのコード基盤はすでに整っている。

なお、ADR-0050 は `pg_try_advisory_lock` による DB マイグレーション排他制御に関する決定であり、
本 ADR（ROPC 移行）とは別テーマである。

## 決定

Keycloak に `auth-rust` 専用の Service Account を作成し、
Client Credentials Grant（client_id + client_secret）のみを使用する形に統一する。
ROPC を使用する分岐（`admin_username` / `admin_password` を参照するコードパス）を削除する。

## 理由

- Client Credentials Grant はサービス間通信（M2M）の標準フローであり、
  ユーザー認証情報を扱わないため認証情報漏洩リスクが低い
- Keycloak の Service Account 機能により、Admin API へのアクセス権限を
  最小権限原則に従って付与できる
- 既存の `else` ブランチで Client Credentials Grant が実装済みのため、
  移行コストが低い
- OAuth 2.1 準拠により、将来の IdP バージョンアップへの対応コストを削減できる

## 移行手順

1. **Keycloak Service Account の作成**
   - Keycloak 管理コンソールで `auth-rust` 専用のクライアント（例: `auth-rust-admin`）を作成する
   - "Service Accounts Enabled" を有効にする
   - 必要な Admin API スコープ（`realm-management` ロール）を Service Account に付与する

2. **ROPC 分岐の削除**
   - `keycloak_client.rs` から `uses_admin_password_grant()` メソッドを削除する
   - `admin_token_url()` の分岐を削除し、`realm` を常に使用するよう統一する
   - `admin_token_form()` の `if` 分岐を削除し、Client Credentials Grant の実装のみ残す
   - `KeycloakConfig` から `admin_username`・`admin_password`・`admin_realm`・
     `admin_client_id` フィールドを削除する

3. **設定の統一**
   - `docker-compose.dev.yaml` の Keycloak 設定から `ADMIN_USERNAME` / `ADMIN_PASSWORD`
     環境変数を削除する
   - `.env.dev` の対応するエントリを削除する
   - `auth-rust` の `config.yaml` を Client Credentials 設定（`client_id` / `client_secret`）
     に統一する

4. **シークレット管理の更新**
   - Vault の `auth-rust` 専用ロール（ADR-0045 参照）に新しいクライアントシークレットを登録する
   - Kubernetes Secret を更新して新しい認証情報を注入する

## 影響

**ポジティブな影響**:

- OAuth 2.1 準拠によりセキュリティ態勢が向上する
- ROPC 廃止による認証情報漏洩リスクの低減
- `KeycloakConfig` の構造が簡素化され、設定ミスの余地が減る
- Service Account により Keycloak 側でのアクセス権限監査が容易になる

**ネガティブな影響・トレードオフ**:

- `docker-compose.dev.yaml` と `.env.dev` の更新が必要であり、
  既存の開発環境を再構築するコストが発生する
- Keycloak の Service Account 作成・設定作業が一度必要になる
- 既存の `admin_username` / `admin_password` を利用している環境では
  移行作業が完了するまで並行稼働できないため、計画的なリリースが必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| ROPC 継続使用 | 現状維持。`admin_username`/`admin_password` を引き続き使用する | OAuth 2.1 で廃止予定であり、長期的な技術負債になる |
| Device Authorization Grant | ユーザー操作が必要なフロー | サービス間通信（M2M）には不適切 |
| API Key 認証 | Keycloak の API Key で管理 API を呼び出す | Keycloak の標準フローから外れ、将来の互換性が低い |

## 参考

- [OAuth 2.0 Password Grant（廃止予定）](https://oauth.net/2/grant-types/password/)
- [OAuth 2.1 Draft (RFC 9126)](https://oauth.net/2.1/)
- [Keycloak Service Accounts](https://www.keycloak.org/docs/latest/server_admin/#service-accounts)
- [ADR-0045](./0045-vault-per-service-roles.md) — Vault サービス個別 Kubernetes Auth ロール実装計画
- [ADR-0050](./0050-advisory-lock-timeout-strategy.md) — pg_try_advisory_lock + リトライによる DB マイグレーション排他制御（本 ADR とは別テーマ）

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-31 | 初版作成（HIGH-04 + LOW-03 監査対応） | @system |
