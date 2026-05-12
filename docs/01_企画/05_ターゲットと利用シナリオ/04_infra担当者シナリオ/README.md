---
id: plan.infra.scenario_index
axis: infra
phase: plan
kind: index
status: draft
depends_on:
  - arch.infra.infra_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# infra 担当者シナリオ index

## 一文方針

infra 担当者（シニア級）が日常的に踏む 14 シナリオを 1 ファイル 1 シナリオで列挙する。cluster 構築 / 5 topology_class / 5 clock_integrity_class / 25+ Kyverno admission policy の維持と、全 19 軸の defense-in-depth 層 E（13 cross-cutting 適合仕様を含む）（物理 enforcement）の最終 safety net 提供が主責務。

## 担当者プロフィール

- 級: シニア
- 想定人数: 3-5 名
- 必須スキル: Kubernetes / Argo CD / Istio / Calico BGP / Envoy / OpenBao / Cosign / Litmus
- 責務: cluster 構築 / 5 topology_class / 5 clock_integrity_class / 25+ Kyverno admission policy

詳細は [層別エンジニア要件](../../../02_要件定義/05_開発体制要件/01_層別エンジニア要件.md) を参照。

> **注記**: 「全 19 軸の defense-in-depth 層 E（13 cross-cutting 適合仕様を含む）」の「13」は `docs/04_詳細設計/03_クロスカッティング適合仕様/` 配下の適合仕様**ファイル数**です。tier3 の「13 層強制機構」（tier3 固有の enforcement 13 層）とは別概念です。

## infra 軸の主要分類

### 5 topology_class（failover drill の対象）
| class | drill cadence |
|---|---|
| v1_local_only | 180 日 |
| v1_zone_replicated | 90 日 |
| v1_cluster_replicated | 60 日 |
| v1_cross_region_replicated | 30 日 |
| v1_global_replicated | 14 日 |

詳細: [クラスタ位相適合仕様](../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md)

### 5 clock_integrity_class
PTP（高精度）/ NTP（標準）/ HLC（Hybrid Logical Clock）/ leap second 戦略（smear）/ monotonic guard の 5 class。
詳細: [時刻整合適合仕様](../../../04_詳細設計/01_適合仕様/13_時刻整合適合仕様.md)

### 25+ Kyverno admission policy
cluster 変更を制御する Kubernetes admission webhook 群。cosign 署名確認 / image pullPolicy 強制 / resource limit 強制 / network policy 確認 / hostPath 禁止 等が含まれる。
詳細: [infra 強制機構](../../../04_詳細設計/02_強制機構/04_infra強制機構.md)

## シナリオ一覧

| # | シナリオ名 | trigger | 想定頻度 | 主たる関連適合仕様 | 種別 |
|---|-----------|---------|---------|-----------------|------|
| 01 | [cluster upgrade](01_cluster_upgrade.md) | k8s minor version の skew が 1 に達した or upgrade window（≤ 30 日）到来時 | 四半期（30 日以内の upgrade window） | クラスタ位相適合仕様 / infra強制機構 | [周期] |
| 02 | [topology drill failover](02_topology_drill_failover.md) | drill cadence（5 topology_class の定期 drill 間隔）到来時、または DR 演習の指示があった時 | 定期（class 別 14〜180 日） | クラスタ位相適合仕様 | [周期] |
| 03 | [clock integrity 変更](03_clock_integrity変更.md) | データセンター / クラウドプロバイダの PTP 対応変更、または leap second 戦略の見直しが必要になった時 | 不定期（DC 変更時） | 時刻整合適合仕様 | [計画] |
| 04 | [Kyverno admission policy 追加](04_Kyverno_admission_policy追加.md) | 新しい security 要件 / OSS 追加 / 脅威モデル更新に伴い Kyverno admission policy を追加する必要が生じた時 | 月次〜四半期 | infra強制機構 | [計画] |
| 05 | [GitOps 配信 progressive delivery](05_GitOps配信_progressive_delivery.md) | 新サービス / 新 Library version のデプロイを GitOps 経由で progressive に配信する必要が生じた時 | 週次〜月次 | infra強制機構 | [計画] |
| 06 | [Chaos drill 実行](06_Chaos_drill実行.md) | drill cadence（topology_class / preservation_class / clock_integrity_class 別）到来時 | 定期（class 別 14〜180 日） | クラスタ位相適合仕様 / 時刻整合適合仕様 | [周期] |
| 07 | [network 変更](07_network変更.md) | ネットワーク構成変更（Calico BGP ECMP / Istio mTLS policy / Envoy Gateway filter / HTTP/3 QUIC 有効化 / WebTransport）が必要になった時 | 四半期〜年次 | クラスタ位相適合仕様 / HTTP2_enforcement | [計画] |
| 08 | [secret rotation](08_secret_rotation.md) | KEK shamir 鍵の定期ローテーション / cosign 署名鍵のローテーション / OpenBao シークレット TTL 切れ / 漏洩疑い | 定期 + イベント駆動 | 鍵管理適合仕様 / KEK_shamir | [周期]+[緊急] |
| 09 | [node_lifecycle](09_node_lifecycle.md) | node pool 容量上限到達または node 故障時 | 月次〜四半期 + イベント駆動 | クラスタ位相適合仕様 | [計画]+[緊急] |
| 10 | [platform_upgrade](10_platform_upgrade.md) | 運用基盤 OSS の minor/major release 公開または CVE 対応時 | 四半期〜年次 | クラスタ位相適合仕様 | [計画]+[緊急] |
| 11 | [storage 拡張 Longhorn / Ceph](11_storage拡張_Longhorn_Ceph.md) | PVC 容量逼迫アラートまたは storage class 変更が必要になった時 | 月次〜四半期 | クラスタ位相適合仕様 | [計画]+[緊急] |
| 12 | [KEDA / ClusterAutoscaler 調整](12_KEDA_ClusterAutoscaler調整.md) | ワークロードのスケーリング挙動が SLO を満たさなくなった時 | 月次〜四半期 | クラスタ位相適合仕様 | [計画] |
| 13 | [OpenTofu cluster replay 訓練](13_OpenTofu_cluster_replay訓練.md) | 年次 replay 訓練の cadence 到来時 | 年次 | クラスタ位相適合仕様 | [周期] |
| 14 | [canary / progressive delivery 観察と切戻し](14_canary_progressive_観察切戻し.md) | Argo Rollouts の AnalysisRun fail または SLI 劣化検出時 | デプロイ頻度に比例（月次〜四半期に切戻し）| クラスタ位相適合仕様 | [計画]+[緊急] |

## 新規参画者向けオンボーディング

シニア級エンジニアとして着任した際の推奨学習順序:

- **Day 1-3**: README 全体読了 → 05（GitOps 配信）→ 04（Kyverno policy）を通読（日常デプロイの基本操作）
- **Day 4-7**: 02（topology drill）→ 06（Chaos drill）の drill 観察（次回 drill の観察役として参加）
- **Week 2**: 01（cluster upgrade）の staging 先行実施をペアで実施
- **Week 3-4**: 08（secret rotation）→ 09（node lifecycle）の実作業を担当
- **Month 2 以降**: 03（clock integrity）→ 07（network 変更）→ 10（platform upgrade）は発生時に担当

## シナリオ間の依存関係

- **10（platform upgrade）と 01（cluster upgrade）**: 独立して実施可能。ただし Argo CD 等の platform OSS が特定 k8s バージョンに依存する場合は 01 を先行させる。
- **09（node lifecycle）→ 02（topology drill）**: node 追加後の次回 topology drill cadence で、新 node 構成でも drill green であることを確認。
- **08（secret rotation）→ 10（platform upgrade）**: OpenBao 自体を upgrade する場合（10）は、upgrade 前に 08 の rotation タイミングを確認し競合を避ける。
- **04（Kyverno policy）→ 05（GitOps 配信）**: 新 Kyverno policy 追加後、次回デプロイ（05）で policy の admission 動作を確認。

## 関連参照

- infra 設計方針: [../../../03_概要設計/05_infra設計方針/README.md](../../../03_概要設計/05_infra設計方針/README.md)
- ターゲットと利用シナリオ index: [../README.md](../README.md)
