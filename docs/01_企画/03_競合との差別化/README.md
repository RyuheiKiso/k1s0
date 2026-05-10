---
id: plan.competitive_differentiation
axis: overview
phase: plan
kind: index
status: draft
depends_on:
  - plan.background_purpose
  - plan.value_experience
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 競合との差別化

## 一文方針
- 商用 BaaS / SaaS（Salesforce Platform / ServiceNow / OutSystems / Mendix）と OSS-first low-code（Appsmith / ToolJet / Budibase）に対し、本企画は L1+ 単一深耕 + 移行コミットメント + 19 軸同型構造 + 物理 enforcement の 4 軸で差別化する。

## 商用 BaaS / SaaS との差別化

### vendor lock-in 回避
- 商用 BaaS: vendor 独自 wire / vendor 独自 OSS への lock-in
- 本企画: OSS L1+ 単一深耕 + 移行 toolchain（4 primary pair の dry-run green 維持）でロック回避

### audit / compliance の物理 enforcement
- 商用 SaaS: audit / compliance は SaaS 提供 ToS の文章保証に依存
- 本企画: audit hash chain + 外部公証 attestation + WORM Object Lock + KEK shamir M-of-N で物理 enforce、root 権限でも改竄不能

### オフライン業務 / レガシー資産統合
- 商用 SaaS: オフライン業務は限定的、レガシー .NET Framework 統合は SaaS 連携 API のみ
- 本企画: 4 layer state（25h オフライン best-effort）+ レガシー .NET Framework Companion + 8 transport adapter で統合

### 業界 pack の所有
- 商用 BaaS: 業界 pack（製造業 / 金融業 / 医療業）は vendor 提供、テナントが拡張 / 修正できない
- 本企画: 業界 pack を tier2 に分離、テナントが override 拡張点で業界固有差分を表現可能

## OSS-first low-code との差別化

### 業務不変条件の物理 enforcement
- OSS-first low-code（Appsmith / ToolJet / Budibase）: 業務不変条件は app builder で文章運用、業務エンジニアが踏み抜き可能
- 本企画: tier1 / tier2 / tier3 の 8 + 13 層強制機構 + 4 言語等価強度の compile-time 不正遷移防止 + atomic 三表書込で物理 enforce

### 観測 / 監査 / セキュリティ
- OSS-first low-code: 観測 / 監査 / セキュリティは限定的、別 OSS 統合が必要
- 本企画: 19 軸同型構造で OTel + ClickHouse + ABAC + KEK shamir + 脅威モデル 5⁴ cell を統合

### マルチテナント
- OSS-first low-code: マルチテナント分離は物理 cluster 分離 or app-level 実装、混在 risk 高
- 本企画: PostgreSQL RLS FORCE + atomic 三表書込 + cross-tenant integration test + pgaudit の 5 層 defense-in-depth

### formal 軸（数学的 proof）
- OSS-first low-code: 形式検証は提供せず
- 本企画: 5 proof_class（TLA+ / Apalache / Stainless / Dafny / Lean 4 / Kani / CBMC）で 19 軸の不変条件を数学的 proof

## L1+ 単一深耕との哲学的差別化
- 「複数 OSS 同時サポート」を諦める代わりに、単一 OSS の全機能 + 移行 toolchain を継続維持
- 競合（商用 / OSS-first low-code）は複数 backend 並走を売りにするが、本企画は「day-1 で並走しない、ライフサイクルイベント時に確実に移行できる」哲学

## 採用しない差別化
- 「機能数 / 統合 OSS 数」での競争（質を選ぶ）
- 「価格」での競争（運用コスト度外視の至高路線）
- 「即座に release」での競争（1.0.0 完璧主義）

## 関連参照
- [背景と目的](../01_背景と目的/README.md)
- [提供する価値や体験](../02_提供する価値や体験/README.md)
- [ターゲットと利用シナリオ](../05_ターゲットと利用シナリオ/README.md)
- [OSS 公開戦略](../06_OSS公開戦略/README.md)
