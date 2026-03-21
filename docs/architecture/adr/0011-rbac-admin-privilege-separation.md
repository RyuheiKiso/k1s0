# ADR-0011: RBAC admin ロール権限分離（sys_admin と admin の明確な区別）

## ステータス

承認済み

## コンテキスト

`CheckPermission` 関数（`regions/system/library/go/auth/rbac.go`）において、`realm_access.roles` に `admin` ロールを持つユーザーが全リソース・全アクションへのアクセスを許可されていた。

```go
// 問題のあった実装
if HasRole(claims, "admin") {
    return true  // リソースやアクションに関係なく全権限を付与
}
```

この実装は最小権限原則（Principle of Least Privilege）に違反しており、以下のリスクを持っていた。

- **権限昇格リスク**: `admin` ロールを持つユーザーが意図せず全リソースへのアクセスを得られる
- **意図しない権限付与**: `admin` は本来「通常の管理ロール」として設計されていたが、事実上 `sys_admin` と同等の全権限を持ってしまっていた
- **監査困難**: `admin` ロールによる全権限付与が暗黙的であり、権限マトリクスとの乖離が生じていた
- **RBAC設計との不整合**: `docs/architecture/auth/RBAC設計.md` では `sys_admin` のみが全権限スーパーユーザーとして定義されており、`admin` はリソース固有の管理者ロール（`svc_{service}_admin` 等）として定義されている

## 決定

`realm_access.roles` の `admin` ロールによる全権限付与ブロックを削除し、`sys_admin` のみが全リソース・全アクションへのアクセスを持つスーパーユーザーとして扱う。

`admin` ロールは `resource_access.{resource}.roles` に明示的に付与された場合のみ、そのリソースへの全アクションを許可する通常ロールとして扱う。

```go
// 修正後の実装
func CheckPermission(claims *Claims, resource, action string) bool {
    // sys_admin のみ全権限を付与する（スーパーユーザー）
    if HasRole(claims, "sys_admin") {
        return true
    }
    // resource_access のチェック（admin はリソース固有の全アクションのみ許可）
    if access, ok := claims.ResourceAccess[resource]; ok {
        for _, role := range access.Roles {
            if role == action || role == "admin" {
                return true
            }
        }
    }
    return false
}
```

## 理由

### 最小権限原則（Principle of Least Privilege）

各ユーザー・ロールが業務遂行に必要な最小限の権限のみを持つべきという原則。`admin` ロールに全権限を付与することは、この原則に明確に違反する。

### RBAC設計との整合

`docs/architecture/auth/RBAC設計.md` のパーミッション解決ルール第4項に「`sys_admin` は全 Tier の全リソースにアクセス可能（スーパーユーザー）」と明記されており、`admin` については全権限スーパーユーザーとしての定義がない。修正により設計書と実装が整合する。

### 権限昇格攻撃の防止

`admin` ロールへの昇格が即座に全リソース権限の取得に繋がる状況を排除することで、権限昇格攻撃（Privilege Escalation）のインパクトを限定する。

## 影響

**ポジティブな影響**:

- 最小権限原則に準拠したセキュアな権限制御が実現される
- `sys_admin` と `admin` の権限区分が明確になり、監査が容易になる
- RBAC設計書と実装が整合し、権限マトリクスが信頼できる参照となる
- 権限昇格攻撃のインパクトが限定される

**ネガティブな影響・トレードオフ**:

- `realm_access.roles` に `admin` を付与してシステム全体の管理権限を期待していた実装がある場合、権限拒否（403 Forbidden）が発生するようになる
- 該当ユーザーには `sys_admin` への昇格、または各リソースの `resource_access` への `admin` 付与が必要となる
- 既存の Keycloak ロール設定の見直しが必要になる場合がある

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A（現状維持） | `admin` ロールに引き続き全権限を付与する | 最小権限原則違反・権限昇格リスク・設計書との不整合が継続するため却下 |
| 案 B（リソース固有 admin のみ残す） | `resource_access` の `admin` はリソース固有の全アクションを許可し、`realm_access` の `admin` は削除 | 本 ADR で採用した案（realm_access の admin を通常ロールとして扱い、resource_access の admin はリソース固有の全アクションを維持） |
| 案 C（admin ロール廃止）| `admin` ロール自体を廃止し、`svc_{service}_admin` 等の具体的なロールのみで管理する | ロール設計の大規模変更が必要となり、段階的移行コストが高いため将来の検討事項とする |

## 参考

- [RBAC設計](../../architecture/auth/RBAC設計.md) — Tier 別ロール定義・パーミッションマトリクス
- [認証認可設計](../../architecture/auth/認証認可設計.md) — 全体の認証認可設計方針
- [authlib ライブラリ設計](../../libraries/auth-security/auth.md) — RBAC チェック関数の API 仕様
- 実装: `regions/system/library/go/auth/rbac.go`
