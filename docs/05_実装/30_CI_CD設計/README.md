# 30. CI_CD 設計

本章は k1s0 の継続的統合と継続的配信のワークフロー配置を実装フェーズ確定版として固定する。構想設計 [`02_構想設計/04_CICDと配信/00_CICDパイプライン.md`](../../02_構想設計/04_CICDと配信/00_CICDパイプライン.md) で確定した 7 段ステージ（fetch → lint → unit-test → build → scan → push → GitOps 更新）と Harbor 門番（Trivy CVE Critical 拒否）を、GitHub Actions self-hosted runner + reusable workflow の物理配置に落とし込む。

## 本章の位置付け

tier1 / tier2 / tier3 はそれぞれ異なるビルド単位を持つため、愚直な全ビルドは時間と費用の両面で持続不可能となる。本章では path-filter で変更影響範囲を判定し、影響下のみ CI を回す設計を確定する。並行して、すべての PR が通過すべき quality gate を一律適用するため reusable workflow で統制する。quality gate は MUST 扱いとし、例外は ADR 起票必須。

配信側は ADR-CICD-001 で選定した Argo CD を受け手とするため、CI は「コンテナイメージのビルド・署名・Harbor への push まで」を責務とし、クラスタ反映は `70_リリース設計/` に譲る。この境界で CI が配信を握らない GitOps 原則を維持する。CI 不可環境向けの Tekton フォールバックは構想設計で既に選定済み（Phase 2 以降）。

## Phase 確定範囲

- Phase 0: GitHub Actions reusable workflow、path-filter、quality gate、Harbor push（Trivy スキャン付）、cosign 署名連携（80 章で本体定義）
- Phase 1a: sccache / Go module cache / pnpm cache のリモート化、Renovate 自動マージ範囲拡大
- Phase 1b: マージキュー（merge queue）導入可否、Tekton フォールバック設計

## RACI

| 役割 | 責務 |
|---|---|
| Platform/Build（主担当 / A） | ワークフロー設計、キャッシュ、path-filter、Harbor 統合 |
| Security（共担当 / D） | quality gate の必須項目、署名連携点、branch protection、Trivy 閾値 |
| SRE（共担当 / B） | CI 失敗率 SLI、パイプライン MTTR |
| DX（共担当 / C） | CI ログの可読性、PR 反映時間 |

## 節構成予定

```
30_CI_CD設計/
├── README.md
├── 00_方針/                # CI と CD の境界、reusable workflow 原則
├── 10_reusable_workflow/
├── 20_path_filter選択ビルド/
├── 30_quality_gate/        # fmt / lint / unit / coverage
├── 40_Harbor_Trivy_push/   # 構想設計 7 段ステージの物理化
├── 50_branch_protection/
└── 90_対応IMP-CI索引/
```

## IMP ID 予約

本章で採番する実装 ID は `IMP-CI-*`（予約範囲: IMP-CI-001 〜 IMP-CI-099）。

## 対応 ADR / 概要設計 ID / NFR

- ADR: [ADR-CICD-001](../../02_構想設計/adr/ADR-CICD-001-argocd.md)（Argo CD）/ [ADR-CICD-002](../../02_構想設計/adr/ADR-CICD-002-argo-rollouts.md)（Argo Rollouts）/ [ADR-CICD-003](../../02_構想設計/adr/ADR-CICD-003-kyverno.md)（Kyverno）/ [ADR-DIR-003](../../02_構想設計/adr/ADR-DIR-003-sparse-checkout-cone-mode.md)（sparse checkout と CI 整合）
- DS-SW-COMP: DS-SW-COMP-135（配信系）
- NFR: NFR-C-NOP-004（ビルド所要時間）/ NFR-H-INT-001（Cosign 署名）/ NFR-H-INT-002（SBOM 添付）/ NFR-E-MON-004（Flag/Decision 変更監査）

## 関連章

- `10_ビルド設計/` — ビルド機構を CI 上で呼び出す
- `70_リリース設計/` — CI の成果物（署名済イメージ）を CD が受ける
- `80_サプライチェーン設計/` — 署名・SBOM の組込
- `40_依存管理設計/` — Renovate の CI 統合
