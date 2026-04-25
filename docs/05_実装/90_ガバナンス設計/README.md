# 90. ガバナンス設計

本章は k1s0 のガバナンス（ADR-CICD-003 で選定した Kyverno ポリシー・ADR プロセス・Technology Radar・脅威モデル）を実装段階確定版として固定する。採用検討通過後の「意思決定の可監査性」を 10 年維持するため、誰が・いつ・どの根拠で採用したかを ID 付きで残す運用を規定する。ADR-0002（drawio 図解レイヤ規約）と ADR-0003（AGPL 分離アーキテクチャ）はガバナンス上の基準として本章に組み込む。

## 本章の位置付け

採用側組織のガバナンスは、実装者と承認者が異なるという前提で機能する。本章は Kyverno の二分所有モデルを採用する（ADR-POL-001 として新規起票）。すなわち、技術側（D / B）が validate ポリシーの **提案** を行い、統制側（D）が **承認** する。この分離により、実装ペースとガバナンスペースが緊張関係を保ったまま共存する。

Technology Radar（Thoughtworks 方式）は半期ごとに Adopt / Trial / Assess / Hold の 4 区分で管理し、採用判定の根拠を ADR に結合する。脅威モデル（STRIDE）は tier1 公開 11 API および外部連携面ごとに作成し、`80_サプライチェーン設計/` と `85_Identity設計/` に影響を与えるため、本章の改訂は両章の見直しをトリガーとする。

![Kyverno 二分所有モデル / ADR プロセス / Technology Radar / STRIDE 脅威モデル の関係](img/90_Kyverno二分所有_ADR_Radar.svg)

## OSS リリース時点での確定範囲

- リリース時点: Kyverno ポリシー初期セット、ADR プロセスの運用確立、Technology Radar 初版
- リリース時点: 脅威モデル（STRIDE）の tier1 公開 11 API 分、監査ログ基盤
- リリース時点: ISO 27001 / SOC2 相当の統制対応（認証取得は別プロジェクト）

## RACI

| 役割 | 責務 |
|---|---|
| Security（主担当 / D） | Kyverno 承認、脅威モデル、ガバナンス統制 |
| SRE（共担当 / B） | ポリシー違反の検出と対応フロー |
| DX（共担当 / C） | ADR プロセスの開発者体験、Radar の周知 |

## 節構成予定

```
90_ガバナンス設計/
├── README.md
├── 00_方針/                # 二分所有モデルと ADR 運用
├── 10_Kyverno_Policy/      # validate / mutate / generate
├── 20_ADR_プロセス/
├── 30_Technology_Radar/
├── 40_脅威モデル_STRIDE/
├── 50_監査ログ/
└── 90_対応IMP-POL索引/
```

## IMP ID 予約

本章で採番する実装 ID は `IMP-POL-*`（予約範囲: IMP-POL-001 〜 IMP-POL-099）。

## 対応 ADR / 概要設計 ID / NFR

- ADR: [ADR-CICD-003](../../02_構想設計/adr/ADR-CICD-003-kyverno.md)（Kyverno）/ [ADR-0002](../../02_構想設計/adr/ADR-0002-diagram-layer-convention.md)（図解レイヤ規約）/ [ADR-0003](../../02_構想設計/adr/ADR-0003-agpl-isolation-architecture.md)（AGPL 分離）/ 本章初版策定時に ADR-POL-001（Kyverno 二分所有モデル）を起票予定
- DS-SW-COMP: 全体横断（特定 ID なし）
- NFR: NFR-E-SIR-001（Runbook）/ NFR-E-SIR-002（漏洩報告）/ NFR-H-COMP-\*（コンプライアンス）/ NFR-C-MGMT-001（設定 Git 管理）/ NFR-C-MGMT-002（Flag/Decision 管理）

## 関連章

- `40_依存管理設計/` — ライセンス判定
- `80_サプライチェーン設計/` — 署名検証ポリシー
- `85_Identity設計/` — mTLS 強制
