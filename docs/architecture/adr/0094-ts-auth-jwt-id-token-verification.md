# ADR-0094: TypeScript Auth ライブラリ JWT id_token 署名検証の追加

## ステータス

承認済み

## コンテキスト

`regions/system/library/typescript/auth/src/auth-client.ts` の `handleCallback()` は、Keycloak のトークンエンドポイントから受け取った `id_token` をそのまま `TokenStore` に格納していた。id_token の署名検証を行わないため、中間者攻撃や IdP からの改ざんされたトークンを検知できない状態だった。

外部技術監査（C-005）にて「JWT 署名検証なし」として CRITICAL 指摘を受けた。runtime dependency がゼロであり、ライブラリとして OIDC 準拠の検証を行っていないことが問題とされた。

## 決定

`jose` ライブラリ（RFC 7517 準拠の Web Cryptography API ベース JOSE 実装）を runtime dependency に追加し、`handleCallback()` でトークン格納前に `id_token` の RS256 署名を JWKS エンドポイント経由で検証する `verifyIdToken` プライベートメソッドを実装する。

また、OIDC Discovery フェッチに指数バックオフリトライ（最大3回、500ms→1000ms）を追加し、ネットワーク瞬断時の即時失敗を防ぐ（M-016-ts 対応）。

## 理由

- **jose** は Web Cryptography API を使用するため Node.js / ブラウザ両対応で動作する。バンドルサイズが小さく、tree-shaking に対応している。
- `createRemoteJWKSet` + `jwtVerify` の組み合わせにより、JWKS エンドポイントからの動的鍵取得・issuer/audience 検証を簡潔に実装できる。
- `jsonwebtoken` 等の代替と比較して、モダンな ESM 対応かつブラウザ互換性が高い。

## 影響

**ポジティブな影響**:

- Keycloak から受け取った id_token の改ざんを検知できるようになる（CRITICAL 監査指摘の解消）
- OIDC の仕様（issuer・audience 検証）に準拠する
- ネットワーク瞬断時の OIDC Discovery フェッチ失敗率が低下する（M-016-ts 対応）

**ネガティブな影響・トレードオフ**:

- runtime dependency に `jose ^5.0.0` が追加される（バンドルサイズ微増）
- `handleCallback()` がネットワーク IO を追加で1回実施する（JWKS フェッチ、内部でキャッシュされる）
- 既存テストで `verifyIdToken` をスタブ/モックする必要が生じる

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| jsonwebtoken | 広く使われている JWT ライブラリ | ESM / ブラウザ非対応、jose の方が軽量 |
| 手動 JWKS フェッチ + SubtleCrypto | 外部依存なし | 実装複雑度が高く、メンテナンスコストが大きい |
| サーバーサイド検証のみ | クライアントで検証しない | クライアント側での早期検知ができず OIDC 仕様不準拠 |

## 参考

- [jose npm](https://github.com/panva/jose)
- [OIDC Core 仕様 §3.1.3.7 ID Token Validation](https://openid.net/specs/openid-connect-core-1_0.html#IDTokenValidation)
- [ADR-0091: JWT Token Introspection Hybrid](0091-jwt-token-introspection-hybrid.md)
- [ADR-0090: AES-GCM AAD Introduction](0090-aes-gcm-aad-introduction.md)

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-04 | 初版作成（C-005 / M-016-ts 監査対応） | @k1s0 |
