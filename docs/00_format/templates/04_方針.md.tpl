---
id: arch.<axis>.<slug>
axis: <axis>
phase: architecture
kind: policy
status: draft
depends_on: []
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes: []
---

# <axis> <方針名>

## 一文方針
- <方針の核心を 1 文で>。

## 至高路線における立ち位置
- <この方針が「至高路線（運用コスト度外視 / 1.0.0 で完璧 / 段階的 release 禁止 / 機能削減なし）」と整合する根拠>

## 設計原則
- <原則 1>
- <原則 2>
- <原則 3>

## 構成 / 分類 / カテゴリ

### <カテゴリ A>
- <内容>

### <カテゴリ B>
- <内容>

## 採用しない選択肢
- <選択肢>: 採用しない。理由: <理由>。例外: <なければ「なし」>

## 強制機構との bind
- 本方針は [<axis> 強制機構](../../04_詳細設計/02_強制機構/<axis>強制機構.md) の以下経路で物理 enforce される:
    - 層 A: <compile / type check 経路>
    - 層 B: <lint / policy check 経路>
    - 層 C: <integration test 経路>
    - 層 D: <runtime / admission webhook 経路>
    - 層 E: <物理 / cosign / Kyverno 経路>
    - 層 F: <数学的 / proof 経路>（formal 関連時のみ）

## 関連参照
- [<関連 spec>](<相対パス>.md)
