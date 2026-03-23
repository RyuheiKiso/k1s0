# ADR-0029: PostgreSQL 高可用性（HA）戦略

## ステータス

提案

## コンテキスト

現在、k1s0 プロジェクトでは Bitnami `postgresql` Helm chart を使用して PostgreSQL を運用している。
この構成の現状と課題は以下の通りである。

- **現構成**: Bitnami `postgresql` chart（非 HA）、リードレプリカ 2 台、プライマリ自動フェイルオーバーなし
- **課題**:
  - プライマリノードで障害が発生した場合、手動介入なしには自動フェイルオーバーが行われない
  - リードレプリカはスタンバイとして機能しているが、プライマリ昇格のオーケストレーションが欠如している
  - SLA 要件（99.9% 以上の可用性）を満たすには自動フェイルオーバー機能が必要
- **影響範囲**: system, business, service 各層のサーバーが PostgreSQL に依存しており、DB 停止はサービス全体に影響する

本 ADR では、PostgreSQL の高可用性を実現するための戦略を選択・記録する。

## 決定

**CloudNativePG（CNPG）を採用する。**

現在の Bitnami `postgresql` chart から CloudNativePG への移行を計画する。
移行は段階的に実施し、本番環境への影響を最小化する。

## 理由

CloudNativePG は以下の観点から最適な選択である。

1. **Kubernetes ネイティブ設計**: Custom Resource Definition（CRD）として PostgreSQL クラスターを定義するため、Kubernetes のエコシステム（RBAC、ネットワークポリシー、PVC 管理）と自然に統合される
2. **CNCF プロジェクト**: Cloud Native Computing Foundation のサンドボックスプロジェクトであり、コミュニティとドキュメントが充実している
3. **自動フェイルオーバー**: プライマリ障害時に自動的にスタンバイを昇格させる機能を標準で提供する
4. **宣言的管理**: `Cluster` リソースで副本数・リソース・バックアップ設定を一元管理でき、GitOps ワークフローと親和性が高い
5. **WAL アーカイブと PITR**: WAL（Write-Ahead Log）アーカイブと Point-in-Time Recovery を標準サポートしており、RPO/RTO の要件を満たしやすい

## 影響

**ポジティブな影響**:

- プライマリ障害時の自動フェイルオーバーにより可用性が向上する
- Kubernetes ネイティブな宣言的管理でオペレーション負荷が軽減される
- WAL アーカイブによりデータ損失リスクが低減される
- PITR により誤操作からの回復が容易になる

**ネガティブな影響・トレードオフ**:

- 既存の Bitnami `postgresql` chart から CloudNativePG への移行にはデータマイグレーション計画が必要
- CloudNativePG の CRD・オペレーター概念の学習コストが発生する
- Helm chart の変更に伴い、関連する Terraform・CI/CD パイプラインの修正が必要
- 移行期間中はサービス停止ウィンドウの確保が必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| Bitnami `postgresql-ha` | Patroni ベースの HA 構成。Bitnami エコシステムに留まるため移行コストが低い | Patroni は Kubernetes ネイティブではなく、設定が複雑になりやすい。Bitnami 依存が継続するため将来の柔軟性が低い |
| 現状維持（非 HA） | 追加コストなし。既存構成を維持する | SLA 要件（99.9% 以上）を満たせない。プライマリ障害時に手動介入が必要でダウンタイムが発生する |
| Crunchy Data PGO | CloudNativePG と同様の Kubernetes ネイティブ Operator | CloudNativePG と比較してコミュニティ規模・ドキュメント量が少ない。CNCF プロジェクトではない |

## 参考

- [CloudNativePG 公式ドキュメント](https://cloudnative-pg.io/documentation/)
- [CNCF プロジェクト一覧](https://www.cncf.io/projects/)
- [Bitnami PostgreSQL HA chart](https://github.com/bitnami/charts/tree/main/bitnami/postgresql-ha)
- [ADR-0025: Terraform State S3](./0025-terraform-state-s3.md)
- [ADR-0026: Service Tier DB Integration](./0026-service-tier-db-integration.md)
- [ADR-0027: DB App User Role Separation](./0027-db-app-user-role-separation.md)

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-24 | 初版作成（外部監査 M-07 対応） | k1s0 team |
