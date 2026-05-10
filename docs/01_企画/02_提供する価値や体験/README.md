---
id: plan.value_experience
axis: overview
phase: plan
kind: index
status: draft
depends_on:
  - plan.background_purpose
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 提供する価値や体験

## 一文方針
- 業務エンジニア（業務不変条件 / 整合性境界 / セキュリティ制御を意識せずに業務 UX に集中できる）/ 運用エンジニア（cluster / OSS / 鍵 / audit を文章運用ではなく物理機構で運用できる）/ プラットフォーム所有者（業界拡張 / OSS 移行 / 規制対応 / 形式検証を建設的に進められる）の 3 ペルソナに対し、それぞれの価値を構造で提供する。

## 業務エンジニア（tier3 ジュニア級）への価値
- **業務不変条件を踏み抜けない構造**: tier1 / OSS 直接 import 禁止 + tier2 リポジトリ抽象 + tier3 13 層強制機構で物理拒否
- **業務語彙のみで開発できる**: tier2 が業界横断 / 業界共通 / 業界固有の業務 API を提供、tier3 は override 拡張点で業務固有差分のみ表現
- **エラーハンドリングが標準化**: BusinessConflict subtype 4 種（stale_write / lost_update / supersede / concurrent_edit）の UI 分岐を 1:1 固定
- **オフライン業務が透過**: 4 layer state（Server Truth / Optimistic Local / Pending Queue / Draft）で業務コード再設計なしで吸収
- **Backstage Software Template で onboarding 完結**: 強制機構の組み込みは template 生成時点で自動

## 運用エンジニア（SRE / プラットフォーム運営）への価値
- **文章運用ゼロ**: SLO / 監査 / 認可 / 鍵管理 / OSS lifecycle はすべて build artifact + Kyverno admission policy + Cosign 署名で物理 enforce
- **L1+ 移行コミットメント**: OSS ライフサイクルイベント（ライセンス変更 / 改廃）は dry-run green 維持の移行 toolchain で対応
- **ops-edge cluster**: target cluster outage 時にも escalation 物理発火、SPOF 対策
- **8 secret class lifecycle**: 自動 rotation + HSM zeroize + 外部公証 attestation
- **5 incident class × 6 phase playbook**: 全 incident（severity = notify 以上）に postmortem PR、blameless 原則

## プラットフォーム所有者への価値
- **業界 pack 並立構造**: 1.0.0 で製造業のみ ship でも、業界横断層 ↔ 業界 pack の単方向依存 + 第二業界 stub conformance で日後の業界拡張に堪える
- **OSS 移行コミットメント**: L1+ 4 primary pair の dry-run green を年次で維持、ライセンス変更時に即移行可能
- **5 build_provenance_class**: 全 production artifact が cosign signed + SBOM + SLSA L3+ + Witness multi-attestation chain
- **5 proof_class（formal）**: temporal_safety / liveness / refinement / program_correctness / runtime_modelcheck の数学的 proof で 19 軸の不変条件を形式化
- **tier2 / tier3 の互換性**: SemVer + 並行バージョン提供 + 移行コミットメントで tier3 を破壊せずに進化

## 体験設計の原則
- **「同じ業務処理を異なる操作シナリオで表現する場」**: tier3 は表現に集中、業務処理は tier2 集約
- **a11y 最低基準は WCAG 2.1 AA**: axe-core で merge 阻止
- **Web Vitals 閾値守る**: Lighthouse CI で merge 阻止
- **i18n は day-1 から**: 1.0.0 で日本語 / 英語 ship

## 関連参照
- [背景と目的](../01_背景と目的/README.md)
- [ターゲットと利用シナリオ](../05_ターゲットと利用シナリオ/README.md)
- [tier3 設計方針](../../03_概要設計/04_tier3設計方針/README.md)
