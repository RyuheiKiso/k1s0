---
id: arch.tier3.override_implementation
axis: tier3
phase: architecture
kind: policy
status: published
version: 1.0.0
depends_on:
  - arch.tier3.tier3_index
  - arch.tier2.business_asset_ownership
  - detail.tier3.client_state_conformance
covered_by:
  defense_in_depth_layers: [A, B, C]
  proof_classes: []
---

# tier3 override 拡張点実装規約

## 一文方針
- tier3 は tier2 が宣言した拡張点（`extension_points.yaml`）に対してのみ override する。各言語で DI / 注入経路を統一し、override の単体テスト + Schema 拡張 + Workflow 拡張 + 決定表拡張 + invariants 抵触検出を CI で必須実行。

## 言語別 override 実装パターン
- **Rust**: trait + 拡張点 trait の impl で override
- **C# (.NET 8+)**: interface + DI container（`Microsoft.Extensions.DependencyInjection`）で override 注入
- **TypeScript**: hook + Context Provider で override
- **Go**: interface + functional option pattern

各言語で同じ「拡張点 → override impl → 注入」の構造を統一。

## override 範囲（拡張点）
- tier2 が宣言した拡張点のみ override 可能
- 拡張点は YAML 宣言（`extension_points.yaml`）に signature / lv（業界固有 / テナント固有）/ invariants を明記
- 拡張点以外への override（業界横断 / 業界共通 Lv の不変条件）は禁止

## 不変条件への抵触検出
- 各拡張点の invariants は YAML 宣言
- override 後の挙動が invariants を満たすことを contract test で機械検査
- 違反は CI fail

## DI / 注入経路
- 各言語で統一された DI / 注入経路:
  - Rust: trait object + Box<dyn Trait>
  - C#: ServiceCollection.AddTransient / AddScoped
  - TypeScript: React Context Provider + custom hook
  - Go: interface argument

DI container 自体の差し替え禁止（言語別の standard を採用）。

## override の単体テスト
- 各 override impl に対し:
  - 拡張点 signature 一致テスト
  - invariants 満足テスト
  - tier2 提供 contract test の継承

## override の Schema 拡張
- proto / Avro / DDL 拡張は purely additive のみ可能
- 既存 field の意味変更 / 削除は禁止
- 拡張 field は tier2 schema registry に登録（テナント別 sub-namespace）

## override の Workflow 拡張
- Workflow / Activity の override は extension hook（pre / post）のみ
- 基本 Workflow flow は tier2 が所有
- 業界固有 / テナント固有 Lv のみ拡張可

## override の決定表拡張
- 決定表 override は decision table override layer で吸収
- 既定決定表は tier2 が所有
- テナント差分は override layer で表現（[マルチテナント方針](../03_tier2設計方針/05_マルチテナント方針.md)）

## override 後の互換性責務
- override 実装の互換性責務は tier3 / テナントが所有
- tier2 minor version up で override が破綻しないことを CI で検査（contract test）
- tier2 major version up での override 修正は migration codemod が支援

## 採用しない設計
- 業界横断 / 業界共通 Lv 不変条件の override
- 拡張点宣言外の override（tier2 が所有する範囲への侵入）
- override 単体テスト省略
- contract test の skip / 改変
- DI container 自体の差し替え（言語標準のみ）

## 関連参照
- [tier3 設計方針 index](README.md)
- [責務](01_責務.md)
- [tier2 業務資産所有権](../03_tier2設計方針/06_業務資産所有権.md)
- [tier2 マルチテナント方針](../03_tier2設計方針/05_マルチテナント方針.md)
- [tier3 強制機構](../../04_詳細設計/02_強制機構/03_tier3強制機構.md)
