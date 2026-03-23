# ADR-0028: マルチテナントID取得方式

## ステータス

承認済み

## コンテキスト

task・board・activity サービスの HTTP ハンドラーが JWT の `iss`（issuer）クレームをテナントIDとして使用していた。Keycloak の issuer は `https://keycloak.example.com/realms/k1s0` のような URL であり、これはテナント識別子ではなく認証プロバイダーのエンドポイントである。全ユーザーが同一 issuer を共有するため、全ユーザーが同一テナントに所属するという誤った動作を引き起こしていた。

また gRPC ハンドラーではテナントIDが `"system"` にハードコードされており、マルチテナント対応が全く機能しない状態だった。

正しいマルチテナント分離のためには、JWT に専用のテナントIDフィールドが必要である。

## 決定

### Phase 1: カスタムクレームによるテナントID取得（互換性維持）

- `regions/system/library/rust/auth/src/claims.rs` の `Claims` struct に `tenant_id: Option<String>` フィールドを追加
- Keycloak に Protocol Mapper を設定し、JWT に `tenant_id` カスタムクレームを付与する
- HTTP ハンドラー: `claims.tenant_id` を優先して使用し、存在しない場合のフォールバックとして `iss` を使用（互換性維持）
- gRPC ハンドラー: メタデータ `x-tenant-id` ヘッダーから取得し、存在しない場合のフォールバックとして `"system"` を使用

### Phase 2: フォールバック廃止（Keycloak Mapper 設定完了後）

- Keycloak Protocol Mapper の設定が完了し、全クライアントが `tenant_id` クレームを含む JWT を発行できることを確認後、HTTP ハンドラーの `iss` フォールバックを廃止する
- gRPC ハンドラーの `"system"` ハードコードを廃止し、`x-tenant-id` を必須化する

## 理由

JWT の `iss`（issuer）クレームは RFC 7519 において認証プロバイダーの識別子として定義されており、マルチテナントのテナント識別子として使用することは仕様上不適切である。テナントIDには専用のカスタムクレームを使用することで、テナント間の完全な論理分離が可能になる。

Phase 1 と Phase 2 に分けることで、Keycloak 設定変更前後の互換性を維持しながら段階的に移行できる。

## 影響

**ポジティブな影響**:

- マルチテナント分離が正しく機能するようになる
- JWT 標準仕様に準拠したクレームの使用
- Phase 1 で既存クライアントとの後方互換性を維持
- テナントIDが明示的なフィールドとなり、コードの可読性が向上

**ネガティブな影響・トレードオフ**:

- Keycloak への Protocol Mapper 設定が必要（インフラ設定変更）
- 全サービスの再ビルドが必要
- Phase 2 への移行タイミングを管理する必要がある
- gRPC 呼び出し元が `x-tenant-id` メタデータを付与する必要がある

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A: iss を継続使用 | JWT の `iss` をテナントIDとして使用し続ける | 全ユーザーが同一テナントに所属する誤動作。テナント分離が機能しない |
| 案 B: DB ルックアップ | user_id から DB でテナントIDを都度検索 | レイテンシ増加、DB への余分な負荷、JWT の自己完結性が失われる |
| 案 C: リクエストヘッダーのみ | JWT クレームを使わず HTTP ヘッダー `x-tenant-id` のみに統一 | ヘッダー偽装のリスクがあり、JWT の署名検証による信頼性を活用できない |

## 参考

- [RFC 7519: JSON Web Token (JWT)](https://datatracker.ietf.org/doc/html/rfc7519)
- [Keycloak Protocol Mapper documentation](https://www.keycloak.org/docs/latest/server_admin/#_protocol-mappers)
- [ADR-0012: system-tier-rls-scope](./0012-system-tier-rls-scope.md)
- `regions/system/library/rust/auth/src/claims.rs`

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-23 | 初版作成 | @k1s0-team |
