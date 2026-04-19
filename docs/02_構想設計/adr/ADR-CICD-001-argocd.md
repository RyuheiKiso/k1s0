# ADR-CICD-001: GitOps に Argo CD を採用

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / 運用チーム / tier1 開発チーム

## コンテキスト

k1s0 の Kubernetes マニフェスト（Deployment、Service、Argo Rollouts の Rollout、Kafka Topic、PostgreSQL Cluster 等）は、手動 kubectl apply では運用不可能な量になる。さらに、OR-ENV-002 で GitOps 運用が要件化されており、「Git に書いてあるものが正」「クラスタと Git が乖離した場合は Git が正」の原則を技術的に保証する必要がある。

制約条件は以下の通り。

- **オンプレミス完結**
- **マルチクラスタ対応**（Phase 3 で複数クラスタ、DR 構成を想定）
- **Kubernetes 以外のリソース（PostgreSQL Cluster、Kafka Topic）も管理対象**
- **運用チーム 2 名で回せる運用密度**
- **Kustomize / Helm の両方をサポート**

候補は Argo CD、Flux、Rancher Fleet、Spinnaker など。

## 決定

**GitOps ツールは Argo CD（CNCF Graduated、Apache 2.0）を採用する。**

- Argo CD 2.12+
- HA 構成（argocd-server x2、argocd-repo-server x2、argocd-application-controller、Redis HA）
- ApplicationSet で環境別・クラスタ別・テナント別の Application を自動生成
- Sync Policy は Manual（本番）/ Automated（dev/staging）を環境ごとに設定
- Sync Wave で依存リソースの順序制御（CRD → Deployment → Service 等）
- SSO は Keycloak（ADR-SEC-001）経由
- PR 時の Render Preview（argocd-diff-preview）で変更影響の可視化

## 検討した選択肢

### 選択肢 A: Argo CD（採用）

- 概要: CNCF Graduated、Intuit 発、GitOps の業界デファクト
- メリット:
  - GA 済み、大規模運用実績多数
  - UI が洗練、Sync 状態・Diff 確認が直感的
  - ApplicationSet でマルチクラスタ / マルチ環境対応
  - Kustomize / Helm / Jsonnet / Plain YAML すべてサポート
  - SSO、RBAC、Audit Log 標準搭載
- デメリット:
  - argocd-application-controller の CPU 消費が多い（大規模時に注意）
  - マニフェスト以外（Imperative な処理）には不向き

### 選択肢 B: Flux CD

- 概要: CNCF Graduated、Weaveworks 発
- メリット: 軽量、Kustomize 連携が洗練
- デメリット:
  - UI がない（argocd の強みがない）
  - 運用中心の使い方で、開発者セルフサービス性が Argo CD より低い

### 選択肢 C: Rancher Fleet

- 概要: Rancher 傘下、マルチクラスタ特化
- メリット: マルチクラスタ管理が本命
- デメリット:
  - Rancher エコシステム依存が強い
  - 単クラスタでの恩恵が薄い

### 選択肢 D: Spinnaker

- 概要: Netflix 発、CD プラットフォーム
- メリット: Blue/Green、Canary 等のデプロイ戦略豊富
- デメリット:
  - GitOps 純粋思想ではない
  - 運用コンポーネント多数で 2 名チームには重い

## 帰結

### ポジティブな帰結

- Git が single source of truth となり、変更証跡と監査対応が容易
- PR → Merge → Sync の流れで、変更の Audit Log が自動で残る
- UI/CLI で Sync 状態を運用チームが視覚的に確認可能、障害対応の短縮
- ApplicationSet で Phase 3 のマルチクラスタ展開も同じツールで対応可能

### ネガティブな帰結

- argocd-application-controller の負荷監視が必要、大規模化で sharding 検討
- 秘密情報は Git に置けないため、External Secrets Operator + OpenBao との統合が必要
- GitOps 外の Imperative 操作（大規模 migration 等）は別途運用手順が必要

## 実装タスク

- Argo CD HA の Helm Chart バージョン固定、Argo CD 自体も GitOps で自己管理（bootstrap）
- ApplicationSet テンプレートを dev/staging/prod 向けに整備
- Sync Wave 設計（CRD → Operator → Application）
- External Secrets Operator + OpenBao の統合で秘密情報は Git 管理外
- PR プレビュー（argocd-diff-preview）を GitHub Actions に組込み
- SSO を Keycloak に統合、RBAC ポリシー（テナント管理者、運用者、開発者ロール）を定義

## 参考文献

- Argo CD 公式: argo-cd.readthedocs.io
- CNCF Graduated Projects
- Weaveworks GitOps Principles
- ApplicationSet: argoproj.github.io/applicationset
