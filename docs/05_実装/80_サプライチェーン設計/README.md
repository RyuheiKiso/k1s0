# 80. サプライチェーン設計

本章は k1s0 のソフトウェアサプライチェーン（署名・SBOM・プロビナンス・Forensics）を実装段階確定版として固定する。SLSA v1.1 の段階的到達（リリース時点=L2 / 採用後の運用拡大時=L3）、sigstore/cosign による keyless 署名、CycloneDX SBOM、そしてインシデント時の Forensics Runbook を統合的に規定する。ADR-CICD-003 で選定した Kyverno が cosign 検証の実行主体となる。

## 本章の位置付け

サプライチェーン攻撃は 採用側組織にとって最も破壊的なインシデント類型であり、検知までの時間が短いほど影響範囲を封じ込めやすい。k1s0 は「image hash から tier1 公開 11 API のどの呼び出し経路が影響下か」を逆引きできる Forensics Runbook を リリース時点 の必須とする。この逆引きは署名・SBOM・プロビナンスを横断するインデックスに依存するため、本章で一体運用する。

SLSA レベルは リリース時点で L2（ビルド履歴の真正性と改ざん困難性）を満たし / L3（ハーメティックビルド）を目指す。リリース時点 からの L3 を主張する案もあったが、運用負荷と現実性から L2 先行が採択された（ADR-SUP-001 として新規起票）。

![SLSA L2→L3 段階到達 / cosign keyless / CycloneDX SBOM / Forensics Runbook の関係](img/80_SLSA段階到達_cosign_SBOM.svg)

## OSS リリース時点での確定範囲

- リリース時点: cosign keyless 署名、CycloneDX SBOM、SLSA L2 プロビナンス、Forensics Runbook スケルトン
- リリース時点: SBOM 差分監視、脆弱性通知の Runbook 連動
- リリース時点: SLSA L3 ハーメティックビルド、Policy Controller（Kyverno / sigstore-policy-controller）

## RACI

| 役割 | 責務 |
|---|---|
| Security（主担当 / D） | 署名・SBOM・プロビナンス、Forensics Runbook |
| Platform/Build（共担当 / A） | ビルドパイプラインへの署名組込、SBOM 生成の自動化 |
| SRE（共担当 / B） | Forensics Runbook の実行手順、インシデント連動 |

## 節構成予定

```
80_サプライチェーン設計/
├── README.md
├── 00_方針/                # SLSA 段階到達と Forensics 思想
├── 10_cosign署名/           # keyless 署名
├── 20_CycloneDX_SBOM/
├── 30_SLSA_プロビナンス/
├── 40_Forensics_Runbook/   # image hash → 影響範囲逆引き
├── 50_flag_定義署名検証/   # 70_リリース設計との境界
└── 90_対応IMP-SUP索引/
```

## IMP ID 予約

本章で採番する実装 ID は `IMP-SUP-*`（予約範囲: IMP-SUP-001 〜 IMP-SUP-099）。

## 対応 ADR / 概要設計 ID / NFR

- ADR: [ADR-CICD-003](../../02_構想設計/adr/ADR-CICD-003-kyverno.md)（Kyverno）/ [ADR-0003](../../02_構想設計/adr/ADR-0003-agpl-isolation-architecture.md)（AGPL 分離 / サプライチェーン監査エビデンス）/ 本章初版策定時に ADR-SUP-001（SLSA L2→L3 段階到達）を起票予定
- DS-SW-COMP: DS-SW-COMP-135（配信系と結合）
- NFR: NFR-H-INT-001（Cosign 署名）/ NFR-H-INT-002（SBOM 添付）/ NFR-H-INT-003（SLSA Provenance）/ NFR-H-KEY-001（鍵ライフサイクル）/ NFR-E-SIR-003（フォレンジック）

## 関連章

- `30_CI_CD設計/` — 署名・SBOM の生成点
- `40_依存管理設計/` — 依存の SBOM 差分
- `60_観測性設計/` — CVSS 連動の SLI
- `90_ガバナンス設計/` — 署名検証ポリシー
