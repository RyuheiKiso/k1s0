# ADR-0024: Kubernetes ServiceAccount のサービス単位分離

## ステータス

承認済み

## コンテキスト

外部監査 H-05 にて、`infra/kubernetes/rbac/service-accounts.yaml` が tier 単位の
ServiceAccount（`k1s0-system-sa`, `k1s0-business-sa`, `k1s0-service-sa`）のみを定義
しており、同一 tier 内の全サービスが同じ SA を共有していることが指摘された。

この構成では以下のリスクがある。

- 1つのサービスが侵害（コンテナブレイクアウト、依存ライブラリの脆弱性等）された場合、
  攻撃者はその SA に紐づく全権限を利用できる。
- 同 tier 内の他サービスが持つ Kubernetes リソース（Secret, ConfigMap, Pod 等）に
  アクセスされる危険性がある。
- 最小権限の原則（Principle of Least Privilege）を満たしておらず、インシデント発生時の
  爆発半径（blast radius）が tier 全体に及ぶ。

k1s0 プロジェクトは system / business / service の 3 tier で構成されており、
それぞれ多数のマイクロサービスを収容している。

| tier     | サービス数 | 主なサービス                                    |
|----------|-----------|------------------------------------------------|
| system   | 29        | auth, bff-proxy, graphql-gateway, vault 等     |
| business | 1         | project-master                                  |
| service  | 3         | task, board, activity                           |

## 決定

tier 単位 SA は後方互換のために残しつつ、主要サービスごとに個別の ServiceAccount を
`infra/kubernetes/rbac/service-accounts.yaml` に追加する。

命名規則は `{サービス名}-sa`（例: `auth-sa`, `bff-proxy-sa`）とし、
`app.kubernetes.io/component` ラベルでサービスを識別できるようにする。

移行は段階的に行い、新規デプロイからサービス単位 SA を参照するよう Helm chart を更新する。

## 理由

### サービス単位 SA 分離を採用した理由

1. **爆発半径の最小化**: 各サービスが専用 SA を持つことで、1サービスの侵害が
   同 tier 内の他サービスへ波及しない。

2. **監査トレーサビリティの向上**: Kubernetes 監査ログで SA 名をフィルタリングすることで、
   どのサービスが何のリソースにアクセスしたかを明確に追跡できる。

3. **将来の Role/RoleBinding 細分化の基盤**: 現時点では SA のみ分離するが、
   将来的に各 SA に対して最小限の Role を付与する際の土台となる。

4. **段階移行の容易さ**: 既存の tier SA を残すことで破壊的変更なく段階的に移行できる。

### tier 単位 SA を即時廃止しなかった理由

- 既存の Deployment / Helm chart が tier SA を参照しているため、
  一斉切り替えは停止リスクが高い。
- 段階移行によりサービスごとに検証しながら安全に移行できる。

## 影響

**ポジティブな影響**:

- 最小権限の原則に準拠し、外部監査 H-05 の指摘事項を解消する。
- 1サービスの侵害がクラスター全体やtier全体に波及するリスクを大幅に低減する。
- サービスごとの監査ログ追跡が可能になり、インシデント対応が迅速化する。
- 将来的に各 SA へ細粒度の RBAC ロールを付与するための基盤が整う。

**ネガティブな影響・トレードオフ**:

- ServiceAccount リソースが tier 単位 3件からサービス単位 33件へ増加し、
  管理コストが上がる。
- 各サービスの Helm chart を順次更新して `serviceAccountName` を切り替える
  作業が必要になる。
- tier SA と個別 SA が並存する移行期間中は、どの SA が有効かを把握する管理上の
  オーバーヘッドが生じる。

## 段階移行方針

| フェーズ | 内容                                                                        |
|---------|----------------------------------------------------------------------------|
| Phase 1 | `service-accounts.yaml` にサービス単位 SA を追加（本 ADR が対象）             |
| Phase 2 | 各サービスの Helm chart の `serviceAccountName` をサービス単位 SA に更新      |
| Phase 3 | 全サービスの移行完了後、tier 単位 SA を Deprecated としてマークし廃止を予告     |
| Phase 4 | 全 Deployment が個別 SA を参照していることを確認後、tier 単位 SA を削除        |

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A: 現状維持（tier 単位 SA のみ） | 変更なし | 外部監査 H-05 の指摘事項を解消できず、爆発半径のリスクが残る |
| 案 B: Namespace のさらなる細分化 | サービスごとに Namespace を作成する | Namespace 増加によるネットワークポリシー・クォータ管理の複雑化が過大。マイクロサービス数が多く現実的でない |
| 案 C: PodSecurityContext でのユーザー分離のみ | SA は tier 単位のまま OS ユーザーを分離 | Kubernetes RBAC レベルのリソースアクセス制御ができず、根本対応にならない |

## 参考

- [外部監査報告書 H-05: Kubernetes ServiceAccount のサービス単位分離]
- [Kubernetes ドキュメント: ServiceAccount](https://kubernetes.io/docs/concepts/security/service-accounts/)
- [Kubernetes ドキュメント: Least Privilege (RBAC)](https://kubernetes.io/docs/reference/access-authn-authz/rbac/#privilege-escalation-prevention-and-bootstrapping)
- [ADR-0011: RBAC 管理者権限分離](./0011-rbac-admin-privilege-separation.md)
- [infra/kubernetes/rbac/service-accounts.yaml](../../../infra/kubernetes/rbac/service-accounts.yaml)
