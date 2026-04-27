# 60. 観測性設計

本章は k1s0 の観測性（ADR-OBS-001 で選定した Grafana LGTM Stack + Pyroscope、ADR-OBS-002 で選定した OpenTelemetry Collector）の実装面を実装段階確定版として固定する。SLO / SLI 定義、エラーバジェット、Incident Taxonomy、Runbook 連携までを Google SRE Book の運用に揃え、稼働面の意思決定を数値で回す構造を規定する。ADR-0003（AGPL 分離）により Grafana / Loki / Tempo / Pyroscope は別プロセス分離を前提とする。

## 本章の位置付け

可用性とセキュリティを別軸で計測する運用は、現場で「どちらの SLO にも違反していないが、実害は発生している」という盲点を生む。k1s0 は Incident Taxonomy を可用性系とセキュリティ系の共通分類体系として統合する（ADR-OBS-003 として新規起票）。CVSS 9.0+ の脆弱性は 48 時間 SLO、7.0+ は 7 日 SLO としてエラーバジェットと連動する。

DORA 4 keys（Lead Time / Deploy Freq / MTTR / Change Failure Rate）は本章ではなく `95_DXメトリクス/` に置く。本章は稼働側の SLI に責務を絞り、開発者生産性側の SLI と混在させない。Runbook は本章で計測した SLI に紐付く運用手順を `ops/runbooks/` に配置し、本章はその対応関係を管理する（`04_概要設計/55_運用ライフサイクル方式設計/` の Runbook 目録 15 本と接続）。

![観測性スタック配置: LGTM + Pyroscope + OTel Agent/Gateway 2 段構成 + AGPL 分離](img/60_LGTM_Pyroscope_OTel配置.svg)

## OSS リリース時点での確定範囲

- リリース時点: OTel Collector 配置、tier1 API の SLO / SLI 初期定義、Incident Taxonomy、Runbook スケルトン、エラーバジェット運用ルール
- リリース時点: Pyroscope によるプロファイル統合、長期保存（Mimir）
- リリース時点: 合成監視（synthetic monitoring）と顧客影響可視化

## RACI

| 役割 | 責務 |
|---|---|
| SRE（主担当 / B） | SLO / SLI 定義、エラーバジェット、Runbook、Incident Taxonomy |
| DX（共担当 / C） | アプリ開発者向けの観測性 SDK 整備 |
| Security（共担当 / D） | セキュリティ SLI（CVSS 連動）、Incident 分類の一致 |

## 節構成予定

```text
60_観測性設計/
├── README.md
├── 00_方針/                # Google SRE に準拠した運用と AGPL 分離
├── 10_OTel_Collector配置/
├── 20_LGTM_Stack/          # Loki / Grafana / Tempo / Mimir
├── 30_Pyroscope/
├── 40_SLO_SLI定義/
├── 50_ErrorBudget運用/
├── 60_Incident_Taxonomy/   # 可用性 + セキュリティ統合
├── 70_Runbook連携/
└── 90_対応IMP-OBS索引/
```

## IMP ID 予約

本章で採番する実装 ID は `IMP-OBS-*`（予約範囲: IMP-OBS-001 〜 IMP-OBS-099）。

## 対応 ADR / 概要設計 ID / NFR

- ADR: [ADR-OBS-001](../../02_構想設計/adr/ADR-OBS-001-grafana-lgtm.md)（Grafana LGTM）/ [ADR-OBS-002](../../02_構想設計/adr/ADR-OBS-002-otel-collector.md)（OTel Collector）/ [ADR-0003](../../02_構想設計/adr/ADR-0003-agpl-isolation-architecture.md)（AGPL 分離）/ 本章初版策定時に ADR-OBS-003（Incident Taxonomy 統合）を起票予定
- DS-SW-COMP: DS-SW-COMP-124（観測性サイドカー統合）
- NFR: NFR-A-CONT-001（SLA 99%）/ NFR-I-SLO-001（内部 SLO 99.9%）/ NFR-I-SLI-001（Availability SLI）/ NFR-B-PERF-001（p99 < 500ms）/ NFR-C-NOP-001（監視スタック）/ NFR-E-MON-001（特権監査）

## 関連章

- `70_リリース設計/` — エラーバジェット連動の deploy 自動停止
- `85_Identity設計/` — 認証系 SLI
- `95_DXメトリクス/` — DORA 4 keys との役割分離
