# 認証・認可レイヤー設計

## 概要
k1s0プラットフォームの認証・認可は多層防御アーキテクチャを採用する。
Kong API Gateway、Istio サービスメッシュ、JWKS トークン検証、RBAC の4層が協調して動作する。

## 信頼境界とコンポーネント

```
インターネット
     |
[Kong API Gateway]         ← 外部リクエストの入口。OAuth2/OIDC認証、レート制限
     |
[Istio mTLS]               ← サービス間通信の認証。証明書ベースの相互認証
     |
[k1s0 Auth Server]         ← Keycloak + k1s0カスタム認証ロジック
     |
[JWKSVerifier (各サービス)] ← JWT検証。各サービスがローカルでトークンを検証
     |
[RBAC Check (各サービス)]   ← ロールベースアクセス制御。リソース操作の認可
```

## 各レイヤーの責務

### Kong API Gateway
- **位置**: クラスター外部との境界
- **責務**:
  - OAuth2/OIDC認証フロー（Authorization Code Flow, Client Credentials Flow）
  - JWT検証（Kong JWT/OIDC プラグイン）
  - レート制限・DDoS防御
  - リクエストルーティング
- **通過するリクエスト**: 認証済みJWTを持つもの、または公開エンドポイント
- **JWT クレーム検証必須項目**（`infra/kong/kong.yaml` の `claims_to_verify`）:
  - `exp`: トークン有効期限
  - `iss`: 発行元（Keycloak レルム URL と一致すること）
  - `aud`: audience（このサービス向けトークンであることを確認する。OIDC 仕様上必須。別サービス向けトークンの流用を防止する）

### Istio サービスメッシュ
- **位置**: クラスター内のサービス間
- **責務**:
  - mTLS（相互TLS）による通信暗号化・認証
  - サービスアカウントベースのAuthorizationPolicy
  - ゼロトラストネットワーク: デフォルト全拒否、明示的な許可のみ
- **設定**: `infra/kubernetes/istio/` 参照

### JWKS トークン検証
- **位置**: 各サービス（`k1s0_auth::JwksVerifier`）
- **責務**:
  - JWTの署名検証（RS256アルゴリズム固定）
  - トークン有効期限チェック
  - Issuer/Audience 検証
  - JWKSキャッシュ（TTL設定可能、デフォルト60秒）
- **アルゴリズム**: RS256のみ許可（alg confusion攻撃防御済み）
- **設定例**:
  ```rust
  // k1s0_auth::JwksVerifier::new() でRS256固定設定
  JwksVerifier::new(&jwks_url, &issuer, &audience, cache_ttl)
  ```

### RBAC（ロールベースアクセス制御）
- **位置**: 各サービスのビジネスロジック層
- **責務**:
  - ロール確認（Claims.roles）
  - リソース操作（read/write/admin）の認可
  - Tier別ロールプレフィックス（sys_, biz_, svc_）
- **ロール体系**:
  | ロール | 権限 |
  |--------|------|
  | sys_admin | system層全リソースの読み書き |
  | sys_operator | system層リソースの読み書き（管理操作除く）|
  | sys_auditor | system層リソースの読み取りのみ |
  | biz_admin | business層リソースの全操作 |
  | svc_admin | service層リソースの全操作 |

## 認証フロー

### ユーザー認証（ブラウザ）
```
1. ユーザー → Kong → Keycloak (Authorization Code Flow)
2. Keycloak → JWT (access_token + refresh_token) → ユーザー
3. ユーザー → Kong (Authorization: Bearer <token>) → サービス
4. サービス → JwksVerifier.verify(token) → Claims
5. サービス → RBAC.check(claims.roles, resource, action)
```

### サービス間認証（gRPC）
```
1. サービスA → Istio mTLS → サービスB
2. サービスB → Istio AuthorizationPolicy で送信元を検証
3. 必要な場合: サービスA → gRPC Metadata (Authorization) → サービスB
4. サービスB → JwksVerifier.verify() → 認可チェック
```

### Machine-to-Machine（バッチ処理）
```
1. バッチジョブ → Client Credentials Flow → Keycloak → JWT
2. JWT → サービスAPI → JwksVerifier + RBAC
```

## セキュリティ上の注意事項

### 内部通信の認証要件
- **gRPC サーバー**: 全ハンドラーで Claims チェック必須（actor_from_claims はデフォルトで "anonymous" を返すため、明示的なNoneチェックが必要）
- **REST ハンドラー**: ActorExtractor middleware が Claims を抽出し、全ハンドラーで `actor` を使用すること

### トークンの取り扱い
- JWTをログに出力しない
- JWTをURLパラメータに含めない（Authorizationヘッダーを使用）
- refresh_tokenはセキュアクッキーに保存（BFF経由）

### Keycloak realm-k1s0.json の環境分離（M-17/M-18 監査対応）

`infra/keycloak/realm-k1s0.json` は開発・ステージング・本番の **共通ベース設定** として管理しているが、
本番デプロイ前に以下の点を必ず確認・修正すること。

#### M-17: webOrigins ワイルドカードの制限

現在の設定:
```json
"webOrigins": [
  "https://*.k1s0.internal.example.com",
  "http://localhost:3000",
  "http://localhost:5173"
]
```

- `*.k1s0.internal.example.com` のワイルドカードは内部ドメイン限定だが、サブドメインを乗っ取られた場合の CORS バイパスリスクがある
- **本番環境では** ワイルドカードを削除し、明示的なドメインリストに変更すること（例: `https://app.k1s0.internal.example.com`）
- ステージング環境でも同様にサブドメインを特定すること
- 開発環境のみ `http://localhost:3000`、`http://localhost:5173` を許可する

#### M-18: localhost URI の本番混入防止

現在の設定に含まれる localhost URI は **開発環境専用** である:
```json
"redirectUris": [
  "https://app.k1s0.internal.example.com/callback",
  "http://localhost:3000/callback",   ← 開発環境のみ
  "http://localhost:5173/callback"    ← 開発環境のみ
]
```

- 本番 Keycloak インスタンスには localhost URI を含めてはならない
- 本番デプロイ時は `infra/keycloak/realm-k1s0.json` をそのままインポートせず、
  以下いずれかの方法で環境分離を行うこと:
  1. **推奨**: 本番専用のオーバーライドスクリプト（`infra/keycloak/scripts/patch-realm-prod.sh` 等）で
     localhost エントリを削除してからインポートする
  2. Terraform/Helm の環境別 values から `KEYCLOAK_REDIRECT_URIS` 変数として注入し、realm.json をテンプレート化する
- `_comment_redirectUris` フィールドに記載の通り、HIGH-SEC-04 監査対応として本番環境では explicit ドメインのみを使用すること

## BFF-Proxy の役割
`regions/system/server/go/bff-proxy/` がフロントエンドとバックエンドの橋渡し:
- フロントエンドはHTTPOnly Cookieでセッション管理
- BFF-Proxyがセッションからトークンを取得してバックエンドに転送
- `ALLOW_REDIS_SKIP` は development 環境のみ有効（production/staging では無効）

## 関連ドキュメント
- `docs/servers/system/auth/` — Auth Server設計
- `docs/servers/system/bff-proxy/` — BFF-Proxy設計
- `infra/kubernetes/istio/` — Istio設定
- `infra/docker/vault/` — Vault設定

## 更新履歴

| 日付 | 変更内容 |
|------|---------|
| 2026-03-21 | 初版作成（技術品質監査対応 P2-31） |
