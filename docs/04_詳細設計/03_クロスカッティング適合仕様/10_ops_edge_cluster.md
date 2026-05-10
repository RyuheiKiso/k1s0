---
id: detail.ops.ops_edge_cluster
axis: ops
phase: cross_cutting
kind: cross_cut_spec
status: draft
depends_on:
  - arch.ops.ops_index
  - arch.ops.oncall_policy
  - detail.ops.ops_loop_conformance
  - detail.ops.ops_enforcement
  - detail.security.audit_ingest_gap_monitor
covered_by:
  defense_in_depth_layers: [B, D, E]
  proof_classes: []
---

# ops-edge escalation cluster（v1）

## 位置づけ
- target cluster（tier1 / tier2 / tier3 を運ぶ本番 Kubernetes クラスタ群）が outage / partition / control-plane down に陥った際にも、on-call escalation を物理的に発火・継続するための独立クラスタ
- [オンコール方針](../../03_概要設計/08_ops設計方針/03_オンコール方針.md) / [運用ループ適合仕様](../01_適合仕様/17_運用ループ適合仕様.md) と双方向 lock
- target cluster 内の Argo Workflows / Tekton / Mattermost のみで escalation を組むと、target 障害時に escalation 経路ごと停止する自己観測 invariant 矛盾が発生するため、本クラスタを独立配置する

## 不可避性
- 「自分自身を観測対象とする escalation engine」は target cluster apiserver / etcd / Argo controller の障害時に suspend timer を発火できない
- on-call invariant の「page は 5 分以内に必ず届く」は、target cluster 内では技術的に成立しない時間帯が常に存在する
- target cluster 外の独立 K8s + 独立 etcd + 独立 IdP + 独立 network path で escalation engine を運用することによってのみ「target が落ちても page は届く」を実現する

## 配置原則
- 物理 region は target cluster と異なる region を最低 1
- control plane は target cluster とは別 etcd / 別 apiserver（managed service の利用も可、AGPL/SSPL/Commercial 排除原則の例外として明記）
- IdP（Keycloak）は target IdP と分離。escalation 用途 Keycloak の users は SRE roster のみ、tier3 業務 users は持たない
- network path は target cluster と独立した carrier 経路で出口を確保（VPN / 直結回線 二系統）
- storage は ops-edge 専用 PostgreSQL（escalation log / rotation roster / acknowledged_at）

## 構成

### 役割 OE-A: escalation engine
- Argo Workflows（ops-edge 上）が rotation roster を参照し、ack window 5 min strict / secondary 5 min / domain-expert 5 min の 3 段 escalation を実行
- target cluster からの page 受信は webhook（HTTPS）+ Kafka MirrorMaker（target → ops-edge）の二経路冗長

### 役割 OE-B: multi-channel notification
- Asterisk（GPL-2.0、process_boundary linkage）+ 自社契約 SIP trunk 二系統（carrier A / carrier B）で voice call
- SMS は SIP trunk の SMS service + 国内 SMS aggregator の二系統
- Mattermost mobile push は APNs（Apple Push）+ FCM（Firebase Cloud Messaging）。商用 SaaS 排除原則の例外として「escalation 用途に限り許容」を本ファイルで明文化（[OSS 採用一覧](../../02_要件定義/04_技術選定/01_OSS採用一覧.md) の例外規律と整合）
- 全 channel は seq parallel（一斉発火）+ ack 受信時に他 channel 取消の semantics で運用

### 役割 OE-C: auto-routing fallback
- `rotation_gap_zero` 不能時間（全 SRE unreachable）に備えて、predefined runbook を ops-edge 上 ServiceAccount が auto-execute する
- runbook は `runbook/auto_route/*.yaml` で事前定義、blast radius は `blast_radius.lock.yaml` で承認済み

### 役割 OE-D: dead-man monitoring
- ops-edge 自体の障害を検出するため、第三クラスタ（別 region / 別 vendor）から ops-edge へ毎分 heartbeat
- heartbeat 喪失時は第三クラスタが直接 SMS / voice で page（dead-man for the dead-man）

## APNs / FCM 例外の根拠
- escalation の last-mile delivery（mobile lock 画面表示）は APNs / FCM が事実上 OS native の唯一経路
- Apple / Google プラットフォームの設計上、third-party push channel は deliverability が著しく劣る
- 「商用 SaaS 排除」原則の趣旨は「業務継続性を商用 SaaS の SLA に依存しない」であり、escalation の last-mile に限定して例外許容することは原則の趣旨と整合
- APNs cert / FCM service account の rotation は cert-manager + Backstage workflow で自動化、年次運用 toil として 12 に組み込む

## SIP trunk 二系統契約の派生規律
- carrier A / carrier B は法人契約。両者の支払い停止 / 契約失効を防ぐため、契約満了の 60 日前に Backstage が auto-PR で更新作業 issue を起票
- SIP trunk の health check は ops-edge から 5 min ごと silent call、応答失敗時は反対 carrier に自動 reroute + SRE への notification

## 採用しない設計
- target cluster 内のみで escalation engine を完結させる: 禁止（自己観測 invariant 矛盾）
- 商用 PagerDuty / VictorOps SaaS 依存: 採用しない（escalation は Apache 2.0 / MIT 経路 + APNs/FCM 例外で完結）
- single carrier の SIP trunk: 禁止、二系統必須
- ops-edge 自体の dead-man なし運用: 禁止、第三クラスタが必須
- escalation engine の手動 escalation rule 編集: 禁止、`escalation_policy.lock.yaml` build artifact 経由のみ

## 整合
- 整合 1: [オンコール方針](../../03_概要設計/08_ops設計方針/03_オンコール方針.md) の「Argo Workflows + Mattermost のみ」記述を本クラスタ運用に置換。自製 escalation engine は ops-edge 上で運用
- 整合 2: [運用ループ適合仕様](../01_適合仕様/17_運用ループ適合仕様.md) の `rotation_gap_zero` invariant は `escalation_continuity_invariant` に置換され、auto-routing fallback が page 継続を担保
- 整合 3: [OSS 採用一覧](../../02_要件定義/04_技術選定/01_OSS採用一覧.md) / [ops 採用 OSS](../../03_概要設計/08_ops設計方針/) の「商用 SaaS 排除」原則の例外（APNs / FCM）を本ファイルが定義
- 整合 4: [audit_ingest_gap_monitor](09_audit_ingest_gap_monitor.md) の SPOF 対策は ops-edge 経路（target cluster 障害時にも audit gap alert を ops-edge から page）で担保
- 整合 5: [検証規律適合仕様](../01_適合仕様/19_検証規律適合仕様.md) の Chaos drill 定義に「target cluster kill → ops-edge から page 到達」シナリオを追加（SRE drill 必須）

## 関連参照
- [オンコール方針](../../03_概要設計/08_ops設計方針/03_オンコール方針.md)
- [運用ループ適合仕様](../01_適合仕様/17_運用ループ適合仕様.md)
- [ops 強制機構](../02_強制機構/07_ops強制機構.md)
- [audit_ingest_gap_monitor](09_audit_ingest_gap_monitor.md)
