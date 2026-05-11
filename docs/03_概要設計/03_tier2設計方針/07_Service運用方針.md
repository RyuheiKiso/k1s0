---
id: arch.tier2.service_operation_policy
axis: tier2
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.tier2.tier2_index
  - arch.tier2.multi_tenant_policy
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_program_correctness_proof
---

# tier2 Service 運用方針

## 一文方針
- tier2 機能カテゴリは Library 形態（in-process）または Service 形態（独立 Pod）で配布する。同一機能を両形態で提供する場合は共通 conformance test suite で両方 green を merge 条件とし、Service 形態は共有 Pod / 共有 DB のマルチテナント運用を default に保護四層 + 自動昇格 trigger で SLO 違反波及を抑制する。

## 配布形態の選択
- **Library 形態**: tier3 業務 process に in-process linkage（言語別 package）
  - 利点: latency 最小、tier1 Library と同 process 内で context propagation
  - 欠点: tier3 process の resource を共有
- **Service 形態**: 独立 Pod として動作（Helm / Operator manifest）
  - 利点: tier3 と独立した resource 管理、tier3 不在環境でも動作
  - 欠点: network latency、context propagation オーバーヘッド
- 両形態提供カテゴリは [Library Service 等価性](../../04_詳細設計/02_強制機構/02_tier2強制機構.md) 層 7 を必須

## 共有 Pod / 共有 DB のマルチテナント運用
- 既定: 複数テナントが同居する共有 Pod / 共有 DB
- 専用 Pod / 専用 DB への昇格条件は [マルチテナント方針](05_マルチテナント方針.md) を参照
- 昇格時にも業務コードは無変更（リポジトリ抽象が接続先を切替）

## SLO 違反波及の抑制と自動昇格
- 「テナント間 SLO 違反波及確率を error-budget の 5% 以下に抑える」を SLO 化
- 抑制超過時は自動昇格 trigger が 5 min 以内に専用 Pod / 専用 DB へ promote
- 詳細は [SLO protection layers](../../04_詳細設計/03_クロスカッティング適合仕様/05_SLO_protection_layers.md) を参照

## 採用しない設計
- per-tenant rate limiter で「SLO 違反波及なし担保」と表現する文言（「保証」「担保」表現は禁止）
- 共有運用での hysteresis なき自動昇格（flapping 抑止が必須）
- 手動 Provision での専用 Pod 切替（Backstage + ArgoCD 自動化前提）

## 関連参照
- [tier2 設計方針 index](README.md)
- [マルチテナント方針](05_マルチテナント方針.md)
- [SLO protection layers](../../04_詳細設計/03_クロスカッティング適合仕様/05_SLO_protection_layers.md)
- [tier2 強制機構](../../04_詳細設計/02_強制機構/02_tier2強制機構.md)
