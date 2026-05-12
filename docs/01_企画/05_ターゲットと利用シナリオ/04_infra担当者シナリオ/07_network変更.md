---
id: plan.infra.scenario_network_change
axis: infra
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.infra.infra_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# network 変更

## 一文方針

L3（Calico BGP）/ L4-L7（Istio service mesh）/ Edge（Envoy Gateway）の変更を IaC 宣言 → staging 先行検証 → chaos drill → GitOps 本番適用の手順で安全に実施し、既存 HTTP/2 経路と mutual TLS を維持する。

> 朝 9 時、本社 IT 室の infra 担当者（シニア級）が Argo CD UI で staging cluster の ApplicationSet を確認中に、「Envoy Gateway HTTP/3 QUIC stable サポート公開」の upstream release note を Mattermost `#infra-ops` で共有されていることに気付く。手元には現行 Calico / Istio の IaC 設定と Kubeshark トレース画面、Mattermost 越しに ops 担当者と dual reviewer がいる。

## Trigger（発火条件）

- Calico BGP ECMP 設定変更が必要になった時
- Istio mTLS policy の更新が必要になった時
- Envoy Gateway filter の追加 / 変更が必要になった時
- HTTP/3 QUIC 有効化が求められた時
- WebTransport 対応が必要になった時

## 想定頻度 / 典型きっかけ

想定頻度: 四半期〜年次。典型きっかけ: 「Envoy Gateway が HTTP/3 QUIC を stable サポートし、本番 cluster での有効化を評価することになった」

## 主役 / 関与者

- 主役: infra 担当者（シニア級）
- 関与: ops 担当者（SLO 監視）、dual reviewer

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（infra）| シニア | 本社 IT 室 | Argo CD UI / Istio dashboard | L3/L4-L7/Edge 変更の IaC 宣言 / Kubeshark トレース確認 / chaos drill 実施 |
| 関与（ops）| シニア | 本社 IT 室 / リモート | Perses dashboard（SLO パネル）| staging / 本番適用中の SLO 監視 / トラフィック異常 alert |
| 承認（dual reviewer）| シニア | 本社 IT 室 / リモート | Mattermost `#infra-ops` | IaC PR レビュー / Kubeshark 確認結果 sign-off |

## 前提

- Calico / Istio / Envoy Gateway が GitOps（Argo CD）で管理されていること
- Kubeshark が staging cluster に導入されていること（本番では runbook 明示的 enable が必要）

## 流れ

1. 変更対象を分類: L3（Calico BGP）/ L4-L7（Istio service mesh）/ Edge（Envoy Gateway）
2. IaC（OpenTofu / Kustomize）で変更を宣言し staging に先行適用
3. Kubeshark で変更後の API トラフィックを ad-hoc 確認（staging のみ。本番は runbook 明示的 enable）
4. HTTP/3 QUIC 有効化: HTTP/2 enforcement spec との整合確認（既存 HTTP/2 経路を壊さない）
5. Istio mTLS policy 変更: 全 service 間通信が mutual TLS であることを Istio dashboard で確認
6. chaos drill（Network partition 実験）で変更後も failover が機能することを確認
7. 本番適用（Argo CD GitOps）/ dual reviewer sign-off

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（infra は全業務の k8s cluster / network / storage の基盤を担うため）。特に影響度が高い 2 業務:

- **SCADA（IoT 通信）**: センサー → cluster 間の大量 IoT パケットは L3 Calico BGP ECMP ルートの変更で経路が変わるため、変更後の Kubeshark トレースで SCADA namespace のパケット損失がゼロであることを最優先に確認する。
- **設備操作（remote）**: 工場フロアからのリモート設備操作は Istio mTLS に依存しており、mTLS policy 変更後に全操作 API の mutual TLS が維持されていることを Istio dashboard で確認してから本番適用する。

## 関連適合仕様 / 関連 OSS

- クラスタ位相適合仕様: [../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md](../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md)
- HTTP2_enforcement: [../../../04_詳細設計/03_クロスカッティング適合仕様/01_HTTP2_enforcement.md](../../../04_詳細設計/03_クロスカッティング適合仕様/01_HTTP2_enforcement.md)
- 関連 OSS: Calico / Istio / Envoy Gateway / Kubeshark / OpenTofu / Kustomize / Argo CD / Litmus

## 期待結果 / 観測指標

- 変更後も既存 HTTP/2 経路が壊れていないこと
- 全 service 間通信が mutual TLS を維持していること
- chaos drill（Network partition）後の failover が成功すること
- dual reviewer sign-off が記録されていること

## 失敗時の挙動 / escalation

- staging での Kubeshark 確認で異常トラフィック検出 → 変更 rollback + 原因究明 + PR 差し戻し
- HTTP/2 enforcement spec 違反 → apply 中止 + lint 修正 + 再レビュー
- Istio mTLS 検証失敗 → 変更 rollback + security 担当者 escalation
- chaos drill fail → 本番適用中止 + **postmortem 期限: 2 営業日以内**（Backstage runbook `network-change-postmortem`）

**escalate 先**: ops 担当者 / Mattermost `#infra-incident`（トラフィック異常・drill fail 時）、security 担当者（mTLS 検証失敗時）
**SLA**: 本番影響発生時は 30 分以内に rollback 完了
**runbook**: Backstage runbook `network-change-rollback` を参照

## 関連参照

- [README.md](README.md) — infra 担当者シナリオ index
- [06_Chaos_drill実行.md](06_Chaos_drill実行.md) — Chaos drill 実行シナリオ
- [../../../03_概要設計/05_infra設計方針/README.md](../../../03_概要設計/05_infra設計方針/README.md) — infra 設計方針
