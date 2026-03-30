# ADR-0057: Kong JWT プラグインにおけるユーザー認証とサービス間認証の分離

## ステータス

承認済み

## コンテキスト

Kong API ゲートウェイでは、グローバル JWT プラグインを `key_claim_name: iss` で設定し、Keycloak が発行したユーザートークンを検証している。

一方、`saga-v1` と `dlq-manager-v1` はサービス間通信（service-to-service）専用のエンドポイントであり、これらのサービスには `key_claim_name: sub` を使用したサービスアカウント JWT 検証が設定されていた。

この構成では以下の問題が生じていた:

1. **グローバル JWT との衝突**: グローバル JWT プラグインが `key_claim_name: iss` でユーザートークンを検証するのに対し、saga/dlq-manager のサービスレベル JWT が `key_claim_name: sub` で検証するため、どちらのプラグインが優先されるかが不明確だった
2. **Consumer の未定義**: サービス間認証用の Consumer が定義されておらず、Kong が JWT を照合できる Consumer が存在しなかった
3. **役割の混在**: ユーザー向けエンドポイントとサービス間通信エンドポイントが同一の Consumer 設定を共有していた

## 決定

以下の変更を行い、ユーザー認証とサービス間認証を明確に分離する。

### 1. サービス間認証専用 Consumer の追加

```yaml
consumers:
  # 既存: Keycloak ユーザー認証用 Consumer
  - username: keycloak
    jwt_secrets:
      - algorithm: RS256
        key: "${KONG_KEYCLOAK_ISSUER}"

  # 新規: サービス間認証専用 Consumer
  - username: service-to-service
    jwt_secrets:
      - algorithm: HS256
        key: "service-to-service"
        secret: "${KONG_S2S_JWT_SECRET}"
```

### 2. saga-v1 / dlq-manager-v1 のサービスレベル JWT プラグイン維持

`key_claim_name: sub` を維持し、Consumer `service-to-service` と組み合わせてサービスアカウント JWT を検証する。

## 理由

1. **役割の明確化**: Keycloak ユーザートークン（iss ベース）とサービスアカウント JWT（sub ベース）を Consumer レベルで分離することで、意図しない認証バイパスを防止できる
2. **最小権限の原則**: saga/dlq-manager は内部サービスのみが呼び出すエンドポイントであり、外部ユーザーによるアクセスを意図的に拒否する設計が明確になる
3. **監査トレーサビリティ**: Consumer を分離することで、Kong のアクセスログにおいてユーザーアクセスとサービス間通信を区別できる

## 影響

**ポジティブな影響**:

- JWT 検証フローが明確になり、意図しない認証バイパスリスクが低減する
- 監査ログでのアクセス種別の区別が容易になる
- グローバル JWT プラグインとサービスレベル JWT プラグインの衝突が解消される

**ネガティブな影響・トレードオフ**:

- `KONG_S2S_JWT_SECRET` 環境変数の管理が必要（Kubernetes Secret として管理すること）
- サービス間通信を行う内部クライアントは `service-to-service` Consumer に対応した JWT を生成する必要がある

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| Istio AuthorizationPolicy のみで制御 | Kong の JWT プラグインを使用せず、Istio の mTLS + AuthorizationPolicy でサービス間認証を実現する | Istio への完全移行は別フェーズで検討中。現時点では Kong のプラグイン機構を維持する方が移行コストが低い |
| グローバル JWT に統合 | saga/dlq-manager のサービスレベル JWT を削除し、グローバル JWT のみで制御 | サービス間通信と外部ユーザーアクセスを区別できなくなる。ゼロトラスト設計に反する |
| Kong の ACL プラグインで制御 | Consumer グループによる ACL で saga/dlq-manager へのアクセスを制限 | JWT 検証と ACL の二段階設定が複雑になる。Consumer 分離の方がシンプル |

## 参考

- [ADR-0044: Kong API ゲートウェイ設計](./0044-kong-api-gateway-design.md)
- Kong JWT プラグインドキュメント: https://docs.konghq.com/hub/kong-inc/jwt/
- Istio AuthorizationPolicy（将来検討）: https://istio.io/latest/docs/reference/config/security/authorization-policy/
- `infra/kong/kong.yaml` — Consumers セクション

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-30 | 初版作成（H-01 監査対応） | - |
