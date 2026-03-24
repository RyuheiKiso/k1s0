# ADR-0030: tier_access クレームのロールベース動的マッピングへの移行

## ステータス

承認済み（2026-03-24）

## 承認理由

外部監査 H-04 の指摘（全ユーザーが全 Tier へのアクセス権を持つ）の重大度を踏まえ、本 ADR の優先度を引き上げ承認とする。
RBAC が実質的に機能しない状態は最小権限の原則に違反しており、早急な対応が必要。

## 実施タイムライン

| フェーズ | 内容 | 目標期間 |
|---------|------|---------|
| Phase 1 | クライアント別ハードコード: `bff-proxy` クライアントは `["system", "business", "service"]`、`service-client` は `["business", "service"]`、`cli-client` は `["service"]` に変更 | 2026-04 |
| Phase 2 | Script Mapper 実装: Keycloak のカスタム `ScriptMapper` を実装し、ユーザーロール（`sys_admin`, `biz_operator`, `user`）に基づいて `tier_access` クレームを動的にマッピング | 2026-05 |

## コンテキスト

外部技術評価報告書 H-10 において、Keycloak の `tier-access-mapper` がすべてのユーザーに対して `["system", "business", "service"]` を固定値で返す実装（`oidc-hardcoded-claim-mapper`）の問題が指摘された。

### 現状の問題

`k1s0-realm.json` の各クライアント（bff-proxy, service-client, cli-client 等）では以下のような設定がされている:

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

この設定では:
- ロール（`sys_admin`, `biz_operator`, `user`）に関わらず、すべてのユーザーが全 tier へのアクセスを持つ JWT を受け取る
- `tier_access` による Tier 間アクセス制御が事実上機能しない
- Istio AuthorizationPolicy での `tier_access` 検証が形骸化する

### 既存の補完措置

現在は以下の二重検証で RBAC を補完している:
1. `realm_access.roles` による RBAC ロールチェック（各サービスのミドルウェア）
2. Istio AuthorizationPolicy による HTTP レベルの制御

`tier_access` の形骸化は現時点では RBAC で補完されているが、最小権限原則の観点から改善が必要である。

## 決定

以下の 2 段階で移行する。

### フェーズ 1: クライアント別ハードコード（短期）

`oidc-hardcoded-claim-mapper` を維持しつつ、クライアントごとに適切な tier 範囲に制限する。

| クライアント | 現状 | 変更後 |
| --- | --- | --- |
| `bff-proxy` | `["system", "business", "service"]` | `["business", "service"]` |
| `service-client` | `["system", "business", "service"]` | 維持（全 tier が必要） |
| `cli-client` | `["system", "business", "service"]` | 維持（管理操作のため） |

BFF Proxy は system tier（auth, config, saga 等）に直接アクセスする必要がないため、`system` を削除する。

### フェーズ 2: ロールベース動的マッピング（長期）

Keycloak の `oidc-usermodel-realm-role-mapper` をベースに、ユーザーのレルムロールから `tier_access` を動的に生成する Script Mapper に移行する。

```json
// 移行後のプロトコルマッパー設定
{
  "name": "tier-access-mapper",
  "protocolMapper": "oidc-script-based-protocol-mapper",
  "config": {
    "claim.name": "tier_access",
    "script": "// ユーザーロールに基づいて tier_access を動的生成\nvar roles = user.getRoleMappings().getRealmMappings();\nvar tierAccess = [];\nif (roles.contains('sys_admin') || roles.contains('sys_operator')) {\n  tierAccess = ['system', 'business', 'service'];\n} else if (roles.contains('biz_admin') || roles.contains('biz_operator')) {\n  tierAccess = ['business', 'service'];\n} else {\n  tierAccess = ['service'];\n}\ntoken.setOtherClaims('tier_access', tierAccess);"
  }
}
```

> **注意**: Keycloak Script Mapper は Keycloak 18+ でデフォルト無効化されており、有効化には `--features=scripts` フラグが必要。

## 理由

1. **最小権限原則の徹底**: ユーザーロールに応じた最小限の tier アクセスのみを付与することで、不要な権限拡大を防ぐ。

2. **多層防御の実効化**: `tier_access` クレームによる Tier 間制御を RBAC の補完ではなく独立した防御層として機能させる。

3. **段階的移行による安全性**: フェーズ 1 でクライアント別制限を適用してリスクを低減し、フェーズ 2 で動的マッピングに移行することで破壊的変更のリスクを最小化する。

## 影響

**ポジティブな影響**:
- `tier_access` クレームが実際のアクセス制御として機能する
- Istio AuthorizationPolicy の `tier_access` 検証が有意義なセキュリティ境界になる
- 最小権限原則に準拠した JWT が発行される

**ネガティブな影響・トレードオフ**:
- フェーズ 1: realm JSON の変更が必要。クライアント別の設定が増加する
- フェーズ 2: Script Mapper の有効化が必要（Keycloak 設定変更）。スクリプトのテストが必要
- BFF Proxy クライアントが `system` tier を失うと、BFF Proxy から system サービスへの直接呼び出しが制限される（想定通りの動作）

## 代替案

| 案 | 概要 | 採用しなかった理由 |
| --- | --- | --- |
| 案 A: 現状維持 | ハードコードマッパーを継続 | 最小権限原則違反・H-10 要件不満足 |
| 案 B: カスタム SPI | Keycloak SPI でカスタムマッパーを実装 | 開発・メンテナンスコストが高い。Script Mapper で要件を満たせる |
| 案 C: Kong でのクレーム変換 | Kong post-function で `tier_access` をロール別に変換 | JWT 改ざんになるため不適切 |

## 参考

- [tier_access クレームのクライアント別動的マッピング設計](../../architecture/auth/tier-access-mapping.md)
- [RBAC設計.md](../../architecture/auth/RBAC設計.md)
- 外部技術評価報告書 H-10: tier-access-mapper のハードコード問題
- [Keycloak Script-Based Protocol Mapper](https://www.keycloak.org/docs/latest/server_admin/#_script_mapper)
