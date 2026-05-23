---
id: req.requirement_index
axis: overview
phase: requirement
kind: index
status: draft
depends_on:
  - plan.plan_index
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 02_要件定義

## 一文方針
- 本フェーズは要件定義レベルの 6 ディレクトリで構成。スコープ / 機能要件 / 非機能要件 / 技術選定 / 開発体制要件 / 制約と前提 を 19 軸 cross-product で一覧化する。1.0.0 完璧主義 + 機能削減なし + 段階的 release 禁止 を要件として宣言。

## 6 ディレクトリ

### [01_スコープ](01_スコープ/)
- [01_提供スコープ](01_スコープ/01_提供スコープ.md) — 19 軸の提供機能 / artifact / SLO
- [02_非提供スコープ](01_スコープ/02_非提供スコープ.md) — 採用しない設計（OSS / tooling / 運用 / 規律）+ 委譲する責務
- [03_業界 pack スコープ](01_スコープ/03_業界packスコープ.md) — 1.0.0 製造業 pack のスコープ

### [02_機能要件](02_機能要件/)
- 各 tier / client / 運用 UI の機能カテゴリ別要件

### [03_非機能要件](03_非機能要件/)
- SLO / 可用性 / セキュリティ / 性能容量 / 互換性 / 観測可能性 / 監査コンプライアンス / アクセシビリティ国際化 / 検証規律

### [04_技術選定](04_技術選定/)
- [01_OSS 採用一覧](04_技術選定/01_OSS採用一覧.md) — 19 軸の v1_l1plus_primary OSS リスト
- [02_OSS ライセンス規律](04_技術選定/02_OSSライセンス規律.md) — (a)〜(e) 階層、AGPL / SSPL 不採用
- [03_OSS ライフサイクル](04_技術選定/03_OSSライフサイクル.md) — major version migration 演習
- [04_言語スタック](04_技術選定/04_言語スタック.md) — Rust / C# / Go / TypeScript + .NET Framework
- [05_自製 in-house 方針](04_技術選定/05_自製in-house方針.md) — `v1_inhouse_authoritative` 区画

### [05_開発体制要件](05_開発体制要件/)
- [01_層別エンジニア要件](05_開発体制要件/01_層別エンジニア要件.md) — 5 階層エンジニア + 4 横断軸専任

### [06_制約と前提](06_制約と前提/)
- [01_OS 固有依存例外](06_制約と前提/01_OS固有依存例外.md) — APNs / FCM 例外
- [02_レガシー資産統合制約](06_制約と前提/02_レガシー資産統合制約.md) — .NET Framework Companion
- [03_クラウド非依存制約](06_制約と前提/03_クラウド非依存制約.md) — vendor lock-in 回避

## 1.0.0 ship blocker（要件）
- 全 19 軸の release_gate.lock.yaml cell が green
- 4 primary pair の dry_run.lock.yaml の last_green_at が 365 日以内
- formal proof 95 cell（proof_matrix 基底）verified or accepted_with_assumption
- 製造業 pack 9 stress test 全 green
- cosign signed tag が物理 prerequisite

## 関連参照
- [01_企画](../01_企画/README.md)
- [03_概要設計](../03_概要設計/README.md)
- [04_詳細設計](../04_詳細設計/README.md)
