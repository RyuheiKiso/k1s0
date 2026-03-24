# tier_access クレームのクライアント別動的マッピング設計

## 概要

Keycloak の `tier-access-mapper` は JWT の `tier_access` クレームを生成するプロトコルマッパーである。本文書は現在の実装状況・セキュリティ上の課題・今後の移行方針を定義する。

## 現在の状態（ローカル開発環境）

```json
{
  "name": "tier-access-mapper",
  "protocolMapper": "oidc-hardcoded-claim-mapper",
  "config": {
    "claim.name": "tier_access",
    "claim.value": "[\"system\", \"business\", \"service\"]"
  }
}
```

### 問題点（H-10 指摘）

`oidc-hardcoded-claim-mapper` はすべてのユーザーに対して同一の値を返す。これにより：

- `sys_admin`、`biz_operator`、一般 `user` ロールを問わず、全ユーザーが `system`・`business`・`service` の 3 tier すべてにアクセス可能な JWT を受け取る
- RBAC ロール（`realm_access.roles`）が正しく設定されていても、`tier_access` クレームの検証で全 tier が通過してしまう
- 多層防御の一層が形骸化する

## クライアント別 tier アクセス方針

各クライアントの役割に応じて `tier_access` の値を制限する。

| クライアント | 説明 | tier_access |
| --- | --- | --- |
| `bff-proxy` | BFF Proxy（エンドユーザー代理） | `["business", "service"]`（system tier に直接アクセスしない） |
| `service-client` | サービス間通信 | `["system", "business", "service"]`（全 tier 必要） |
| `cli-client` | CLI ツール | `["system", "business", "service"]`（管理操作のため） |

## 移行方針

### フェーズ 1（現状維持 — ローカル開発）

`oidc-hardcoded-claim-mapper` を維持する。ローカル開発環境では全 tier へのアクセスが開発効率に寄与する。

**セキュリティ補完**: Istio `AuthorizationPolicy` と各サービスのミドルウェアによる `realm_access.roles` チェックで RBAC を確保する（二重検証）。

### フェーズ 2（クライアント別ハードコード — 短期）

各クライアントスコープで `oidc-hardcoded-claim-mapper` の値をクライアント別に設定する。

```json
// bff-proxy クライアントのプロトコルマッパー
{
  "name": "tier-access-mapper",
  "protocolMapper": "oidc-hardcoded-claim-mapper",
  "config": {
    "claim.value": "[\"business\", \"service\"]"
  }
}
```

### フェーズ 3（ロールベース動的マッピング — 長期）

Keycloak の Script Mapper（`oidc-script-based-protocol-mapper`）または `oidc-usermodel-attribute-mapper` を使用して、ユーザーのレルムロールに基づいて `tier_access` を動的に生成する。

ADR-0030 で詳細を決定する。

## Istio AuthorizationPolicy との関係

`tier_access` は 2 層で検証される:

1. **Istio AuthorizationPolicy**: `request.auth.claims[tier_access]` で Mesh レベルの検証
2. **アプリケーションミドルウェア**: `tier_access` 配列にサービスの Tier が含まれるかを二重検証

どちらの層も `tier_access` のすべての値を許可している場合、多層防御の意味が薄れる。フェーズ 2 以降の対応でクライアント別の制限を適用することで、最小権限原則を徹底する。

## 関連ドキュメント

- [RBAC設計.md](RBAC設計.md) — `tier_access` Claim の検証ロジック
- [JWT設計.md](JWT設計.md) — JWT クレーム設計
- [ADR-0030: tier_access 動的マッピング移行](../adr/0030-tier-access-dynamic-mapping.md)
