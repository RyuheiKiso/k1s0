---
id: req.<axis>.oss
axis: <axis>
phase: requirement
kind: oss
status: draft
depends_on:
  - req.overview.oss_catalog
  - req.overview.oss_license
  - detail.tier1.oss_lifecycle_conformance
covered_by:
  defense_in_depth_layers: [B, E]
  proof_classes: []
---

# <axis> 採用 OSS

## 一文方針
- <axis> の道具立ては <機能カテゴリ列挙> の N 機能カテゴリに対し L1+ 単一深耕、M OSS を v1_l1plus_primary として `oss_lifecycle.lock.yaml` に登録、横並び OSS を持たない。

## 機能カテゴリ × OSS（v1_l1plus_primary 単一深耕）

| 機能カテゴリ | v1_l1plus_primary | license | linkage_model |
|---|---|---|---|
| <カテゴリ 1> | <OSS 名> | <license> | <static / dynamic / process_boundary / network_service> |
| <カテゴリ 2> | <OSS 名> | <license> | <linkage> |

## 各 OSS の用途と class binding

### <OSS 名 1>
- 用途: <用途>
- 担当軸: <19 軸中の主担当>
- artifact: <生成物>
- 単一深耕: <横並び不採用の根拠>
- 例外: <なければ「なし」>

### <OSS 名 2>
- 用途: <用途>
- 担当軸: <19 軸中の主担当>
- artifact: <生成物>
- 単一深耕: <横並び不採用の根拠>
- 例外: <なければ「なし」>

## OSS 採用しない設計
- <候補 OSS>: 採用しない。理由: <理由>
- <候補 OSS>: 採用しない。理由: <理由>

## ライセンス階層との整合
- 採用 OSS のライセンス階層は [OSS ライセンス規律](../../02_要件定義/04_技術選定/02_OSSライセンス規律.md) の (a) primary 許容 / (b) LGPL / (c) GPL / (d) AGPL / (e) 例外 のいずれかに必ず帰属。
- (d) AGPL/SSPL 等は採用しない。
- (e) 例外（APNs / FCM 等）は <axis> の last-mile 経路に限定する場合のみ。

## OSS ライフサイクルへの登録
- 全採用 OSS は [OSS ライフサイクル適合仕様](../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md) の `oss_lifecycle.lock.yaml` に entry 登録、major version migration 演習が必須。
