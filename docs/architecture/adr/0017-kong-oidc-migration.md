# ADR-0017: Kong JWT プラグインから OIDC プラグインへの移行検討

## ステータス

提案

## コンテキスト

現在、k1s0 の API Gateway（Kong）では標準の **JWT プラグイン**を使用して Keycloak 発行のアクセストークンを検証している。
`infra/kong/kong.yaml` の `consumers` セクションで `rsa_public_key` を手動管理し、
`KONG_KEYCLOAK_ISSUER` 環境変数で発行者（iss）を設定している。

Kong 標準 JWT プラグインには以下の制約がある：

1. **JWKS URL による動的公開鍵解決非サポート**: JWKS エンドポイント（Keycloak の `/protocol/openid-connect/certs`）から
   自動的に公開鍵を取得する機能がない。`rsa_public_key` を静的に設定する必要がある。
2. **鍵ローテーション対応不可**: Keycloak で RSA 鍵をローテーションする際、Kong の Consumer 設定を手動で更新する必要がある。
   ADR-0008（JWT 鍵ローテーション）で定めた 90 日ローテーション方針と相性が悪い。
3. **クレーム検証の限界**: `claims_to_verify` で `exp` と `iss` を検証できるが、
   `aud`（audience）や `azp`（authorized party）など OIDC 標準クレームの検証には対応していない。

`iss` クレーム検証（H-03）は JWT プラグインで対応済みだが、根本的な公開鍵管理の問題は解決されていない。

## 決定

Kong の JWT プラグインから `kong-oidc` プラグインまたは Kong Enterprise の
`openid-connect` プラグインへ移行する。

優先候補として **`kong-oidc`（OSS プラグイン）** を採用し、以下の設定で Keycloak と統合する：

```yaml
plugins:
  - name: openid-connect
    config:
      issuer: "${KONG_KEYCLOAK_ISSUER}/.well-known/openid-configuration"
      client_id: "k1s0-api-validator"
      verify_claims:
        - iss
        - exp
        - aud
      cache_ttl: 600  # JWKS キャッシュ TTL: 10 分
```

## 理由

- **JWKS 自動取得**: OIDC プラグインは Keycloak の JWKS エンドポイントを自動で参照し、
  公開鍵を動的に取得・キャッシュする。手動の `rsa_public_key` 管理が不要になる
- **鍵ローテーション自動対応**: ADR-0008 で定めた 90 日ローテーション計画において、
  Kong 側の手動更新作業が不要となり、運用負荷が大幅に削減される
- **OIDC 標準準拠**: `aud`、`azp`、`nbf` など OIDC 標準クレームの検証が可能となり、
  セキュリティ検証が強化される
- **Discovery Document 対応**: Keycloak の `.well-known/openid-configuration` エンドポイントを
  使用して設定を自動取得するため、Keycloak 設定変更への追従が容易になる

## 影響

**ポジティブな影響**:

- 鍵ローテーション時の手動オペレーション（Consumer 設定更新）が不要になる
- `aud` クレーム検証により、他のシステム向けトークンの誤用を防止できる
- Keycloak との統合設定が簡素化される
- `consumers` セクションの `jwt_secrets` 管理が不要となり、kong.yaml が簡潔になる

**ネガティブな影響・トレードオフ**:

- `kong.yaml` の JWT プラグイン設定を大幅に変更する必要がある（移行コスト）
- `kong-oidc` は OSS プラグインであり、Kong Enterprise の `openid-connect` と比較して
  サポートが限定的な場合がある
- JWKS キャッシュ TTL の設定によっては、鍵ローテーション後に一時的に認証エラーが発生する可能性がある
- `post-function` プラグインで行っている JWT クレーム転送ロジックの見直しが必要になる場合がある

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 現行の JWT プラグイン維持 | `rsa_public_key` の手動管理を継続 | 鍵ローテーションの運用負荷が高く、ADR-0008 の方針と矛盾する |
| Kong Enterprise の openid-connect プラグイン | Kong Enterprise ライセンスで提供される公式 OIDC プラグイン | ライセンスコストが高く、OSS 戦略と合わない。将来の選択肢として保留 |
| 各サービスで個別に JWKS 検証 | API Gateway での JWT 検証を廃止し、各サービスが直接 JWKS 検証 | 防御の多層化（API Gateway + サービス）の原則に反する。Kong での一元検証が望ましい |
| Istio RequestAuthentication との統合 | Istio の JWT 検証機能を使用 | Istio の JWT 検証は mTLS と組み合わせたサービスメッシュ内部向けであり、外部クライアント認証には適さない |

## 参考

- [kong-oidc: OpenID Connect plugin for Kong](https://github.com/nokia/kong-oidc)
- [Keycloak JWKS エンドポイント](https://www.keycloak.org/docs/latest/securing_apps/#_certificate_endpoint)
- [ADR-0008: JWT 鍵ローテーション](./0008-jwt-key-rotation.md)
- [docs/architecture/auth/JWT設計.md](../auth/JWT設計.md)
- [infra/kong/kong.yaml](../../../infra/kong/kong.yaml)
