---
id: detail.infra.ops_dx
axis: infra
phase: detail
kind: ops_dx
status: draft
depends_on:
  - arch.infra.infra_index
  - arch.infra.cluster_composition_policy
  - arch.infra.gitops_delivery_policy
  - detail.infra.infra_enforcement
covered_by:
  defense_in_depth_layers: [D, E]
  proof_classes: []
trace:
  fr_ids:
  - FR-infra-004

---

# infra 運用 UI と開発者体験

## 一文方針
- 運用 UI は Headlamp / Kubeshark で読み取り、開発者 portal は Backstage、ad-hoc 探索は Apache Superset。cluster 状態を変更する操作は全て GitOps 経路。

## 至高路線における立ち位置
- Octant 採用しない（Headlamp L1+ 単一深耕、archive 状態でもあるので考慮しない）
- Lens 採用しない（商用 / 利用規約上 OSS 方針と整合しないため）
- Grafana 採用しない（AGPL 回避）
- Kibana 採用しない（観測可能性 SoR は ClickHouse + Superset、ELK stack は採用しない）
- skaffold v2 候補

## Headlamp（cluster Web UI）
- L1+ primary: Headlamp（CNCF Sandbox、Apache 2.0）
- 用途: cluster 状態の閲覧、Pod log 確認、Pod exec、event 閲覧
- 認証: Backstage と同じ OIDC token（Keycloak）
- 権限: default は read-only、書き込み操作は break-glass role（OpenBao response wrapping 必須）
- audit: 全操作を 09 観測可能性 SoR に bulk insert（`infra.headlamp.action.event`）

## Kubeshark（API トラフィックビューア）
- 用途: cluster 内 API トラフィックの ad-hoc 監視、デバッグ
- scope: staging cluster default、production cluster は runbook での明示的 enable のみ
- PII: Kubeshark で観測されるトラフィックは 09 観測可能性 SoR の `redaction.yaml` と同等の redaction を必須化
- 採用しない用途: production の long-term 監視（Istio access log + ClickHouse が SoR、Kubeshark は ad-hoc 用途のみ）

## Backstage（開発者ポータル）
- L1+ primary: Backstage（Apache 2.0）
- 機能:
    - service catalog: tier1 / tier2 / tier3 の component / API / system を catalog 化
    - software template scaffolding: tier2 Service / tier3 SPA / tier1 Library 新規作成 template
    - kubeconfig 発行: 短期 OIDC token（最大 8 時間）
    - runbook: region failover / restore drill / break-glass 等の runbook を Backstage プラグインで実行
    - tech docs: MkDocs ベースの技術ドキュメント
- 認証: Keycloak OIDC、tenant_id を JWT claim から取得

## Apache Superset（ad-hoc 探索）
- L1+ primary: Apache Superset（Apache 2.0）
- 用途: ClickHouse データソースに対する ad-hoc SQL 探索
- 権限: tier3 業務担当者にも限定的に公開可能、tenant_id フィルタを必須化（32 RLS と同等の論理 filter）

## Perses（メトリクス可視化）
- L1+ primary: Perses（CNCF Sandbox、Apache 2.0）
- 用途: Prometheus データソースの dashboard
- Grafana の AGPL 回避方針を維持

## Jaeger v2 UI（トレース探索）
- L1+ primary: Jaeger v2（v1.50+、jaegertracing/jaeger、first-party、Apache 2.0）の内蔵 ClickHouse storage
- 用途: 分散トレース探索、trace_id 検索
- first-party 実装のため community_plugin_stagnation trigger 対象から除外

## 開発者 inner loop
- Tilt: local Kubernetes（kind / k3d）で iterative development
- 本番 manifest と divergence しない overlay 構造（[IaC 方針](../../03_概要設計/05_infra設計方針/08_IaC方針.md)）
- skaffold は v2 候補
- dev container: Tilt + dev container（VS Code Remote Containers）で onboarding 短縮

## ローカル開発環境の再現性
- kind / k3d で本番 cluster と同 OSS バージョン（`cluster_inventory.lock.yaml` の version 値を local にも適用）
- Testcontainers で integration test 環境を本番 OSS と同バージョンで起動
- [IaC 方針](../../03_概要設計/05_infra設計方針/08_IaC方針.md) の Helm chart Harbor mirror から local も pull

## docs 規約
- 全 docs は MkDocs Material で codify、Backstage tech docs plugin で publish
- 図は drawio（drawio-authoring skill 規約に従う、白背景 / 矢印重なり禁止 / 1 図 1 ページ等）
- レイヤ別の図（アプリ層 / ネットワーク層 / インフラ層 / データ層）は figure-layer-convention skill 規約に従う

## on-call / incident
- on-call shift: Backstage で管理
- incident response: Argo Events で alert → Mattermost / 自製 escalation engine / runbook trigger
- postmortem: Backstage tech docs に template 化、blameless postmortem 文化

## 採用しない選択肢
- Octant: 採用しない
- Lens: 採用しない
- Grafana: 採用しない（AGPL 回避）
- Kibana: 採用しない

## 関連参照
- [infra 設計方針 index](../../03_概要設計/05_infra設計方針/README.md)
- [クラスタ位相適合仕様](../01_適合仕様/12_クラスタ位相適合仕様.md)
- [infra 強制機構](../02_強制機構/04_infra強制機構.md)
