---
id: req.tech.tech_index
axis: overview
phase: requirement
kind: index
status: draft
depends_on:
  - req.overview.provided_scope
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 技術選定 index

## 一文方針
- 本ディレクトリは 19 軸の OSS 採用 / ライセンス規律 / OSS ライフサイクル / 言語スタック / 自製 in-house 方針 を集約する。L1+ 単一深耕 + 移行コミットメント、AGPL / SSPL 不採用、4 言語（Rust / C# / Go / TypeScript）+ Companion 言語（.NET Framework）。

## 5 ドキュメント
- [01_OSS 採用一覧](01_OSS採用一覧.md) — 19 軸の v1_l1plus_primary OSS 一覧
- [02_OSS ライセンス規律](02_OSSライセンス規律.md) — (a)〜(e) 階層、AGPL / SSPL / BSL 不採用
- [03_OSS ライフサイクル](03_OSSライフサイクル.md) — 6 lifecycle_class、8 signal、移行 toolchain
- [04_言語スタック](04_言語スタック.md) — Rust / C# / Go / TypeScript、.NET Framework Companion
- [05_自製 in-house 方針](05_自製in-house方針.md) — `v1_inhouse_authoritative` 区画

## 関連参照
- [提供スコープ](../01_スコープ/01_提供スコープ.md)
- [OSS ライフサイクル適合仕様](../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md)
- [OSS 公開戦略](../../01_企画/06_OSS公開戦略/README.md)
- [法務確認](../../01_企画/04_法務確認/README.md)
