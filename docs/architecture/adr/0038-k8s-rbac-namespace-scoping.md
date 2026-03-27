# ADR-0038: Kubernetes RBAC 権限のNamespaceスコープ化

## ステータス

承認済み

## コンテキスト

外部技術監査（2026-03-26）で以下の問題が指摘された（H-6）:

1. **k1s0-admin の RBAC 操作権限**: `k1s0-admin` ClusterRole が `clusterroles`、`clusterrolebindings` への CRUD 権限を持ち、ClusterRoleBinding で全クラスタに適用されていた。これは権限エスカレーション（admin 自身の権限拡張）のリスクになる。

2. **ClusterRoleBinding の過剰スコープ**: `k1s0-admin` と `k1s0-operator` が ClusterRoleBinding で全クラスタに適用されており、k1s0 アプリケーション外の Namespace にも影響を与える可能性があった。

k1s0 アプリケーションが利用する Namespace は `k1s0-system`、`k1s0-business`、`k1s0-service` の3つに限定されている。これらの Namespace に権限を絞るべきである。

## 決定

1. **RBAC 操作権限を分離**: `k1s0-admin` から `clusterroles`/`clusterrolebindings` への CRUD 権限を除外し、新規 `k1s0-security-admin` ClusterRole に分離する。
2. **ClusterRoleBinding → RoleBinding**: `k1s0-admin-binding` と `k1s0-operator-binding` を ClusterRoleBinding から k1s0 関連 Namespace（k1s0-system, k1s0-business, k1s0-service）への RoleBinding に変更する。
3. **developer と readonly も RoleBinding に変更（H-15 監査対応）**: 2026-03-27 の外部監査（H-15）で指摘を受け、`k1s0-developer-binding` と `readonly-binding` を ClusterRoleBinding から各 k1s0 Namespace の RoleBinding に変更する。参照のみの権限であっても、クラスタ全体の機密 Namespace（kube-system 等）へのアクセスは不要であり、最小権限の原則に従い Namespace スコープに限定する。

## 理由

- **最小権限の原則（PoLP）**: アプリケーション管理者は自分たちのアプリケーション Namespace のみ操作できれば十分。クラスタ全体への書き込み権限は不要。
- **権限エスカレーション防止**: RBAC リソース自体の操作権限と一般的な管理権限を分離することで、意図しない権限昇格を防ぐ。
- **Namespace 分離**: k1s0 の3Tier（system/business/service）は明確に分離されており、各 Namespace に RoleBinding を適用することでより細粒度な制御が可能。
- **参照権限も Namespace 限定（H-15 対応）**: developer や readonly であっても kube-system 等の機密 Namespace を参照できる必要はなく、k1s0 アプリケーション Namespace のみへのアクセスで十分。

## 影響

**ポジティブな影響**:

- 権限エスカレーションのリスクを軽減できる
- k1s0 Namespace 外のリソースへの誤操作を防ぐことができる
- RBAC 操作権限は `k1s0-security-admin` にのみ付与され、セキュリティチームが明示的に管理できる

**ネガティブな影響・トレードオフ**:

- `k1s0-admin` グループのユーザーは、k1s0 Namespace 外での管理操作ができなくなる
- 追加 Namespace が必要な場合は、各 Namespace に個別の RoleBinding を追加する必要がある

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A | ClusterRoleBinding を維持しつつ RBAC 権限のみ削除 | Namespace スコープの問題が残る |
| 案 B | Namespace ごとに完全に独立した Role を定義 | 管理コストが高く、コード重複が増える |

## 参考

- [Kubernetes RBAC ドキュメント](https://kubernetes.io/docs/reference/access-authn-authz/rbac/)
- [infra/kubernetes/rbac/cluster-roles.yaml](../../../infra/kubernetes/rbac/cluster-roles.yaml)
- [infra/kubernetes/rbac/role-bindings.yaml](../../../infra/kubernetes/rbac/role-bindings.yaml)
- ADR-0031: etcd 保存データの暗号化設定

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|-------|
| 2026-03-26 | 初版作成（外部監査 H-6 対応） | k1s0 team |
| 2026-03-27 | developer/readonly も RoleBinding に変更（外部監査 H-15 対応） | k1s0 team |
