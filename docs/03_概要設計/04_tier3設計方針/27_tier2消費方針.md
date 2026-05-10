---
id: arch.tier3.tier2_consumption_policy
axis: tier3
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.tier3.tier3_index
  - arch.tier3.responsibility
  - arch.tier2.business_asset_ownership
covered_by:
  defense_in_depth_layers: [A, B, C]
  proof_classes: []
---

# tier3 tier2 消費方針

## 一文方針
- tier3 は tier2 を一次消費する。tier2 公開 API（Library / Service）+ override 拡張点経由のみ、tier1 Library / OSS 直接利用は CI で物理拒否（[tier3 強制機構](../../04_詳細設計/02_強制機構/03_tier3強制機構.md) 層 1）。tier2 minor version up 時に contract test 自動実行で互換性確認。

## tier2 消費経路
- tier2 generated stub（proto / Avro 由来）のみ使用
- tier2 SDK 経由で API 呼出（fetch / axios 等の生 HTTP 禁止）
- Domain Event subscription は tier2 SDK の subscribe API
- override 拡張点は tier2 が宣言した extension point のみ

## 公開 type の単一の真
- tier3 リポジトリ内に `.proto` ファイル配置禁止（tier2 が単一の真）
- 業務 entity / 業務 event は tier2 .proto 由来の生成型のみ
- 独自 type 宣言禁止

## tier2 minor version up 時の追従
- contract test の自動実行（Pact）
- 拡張点 signature の追加に対応（purely additive）
- 不変条件の強化検出は tier3 修正 trigger

## tier2 major version up 時の追従
- tier2 が提供する移行 codemod を適用
- 拡張点 signature 変更の影響範囲を contract test で検出
- 並行バージョン提供期間（180 日）内に新 MAJOR に移行

## 採用しない設計
- tier1 / OSS 直接 import
- 業務管理 API（admin） の業務 UI 取込
- 独自 type 宣言
- 独自 Domain Event emit（tier2 経由必須）
- contract test の skip / 改変

## 関連参照
- [tier3 設計方針 index](README.md)
- [責務](01_責務.md)
- [override 実装規約](15_override実装規約.md)
- [tier2 業務資産所有権](../03_tier2設計方針/06_業務資産所有権.md)
- [tier3 強制機構](../../04_詳細設計/02_強制機構/03_tier3強制機構.md)
