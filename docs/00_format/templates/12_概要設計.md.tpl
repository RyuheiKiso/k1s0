---
id: arch.<axis>.<slug>
axis: <axis>
phase: architecture
kind: architecture
status: draft
depends_on: []
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes: []
---

# <タイトル>

## 一文方針
- <この概要設計ドキュメントの核心を 1 文で>。

## 設計原則
- <原則 1>
- <原則 2>
- <原則 3>

## 構成

### <構成要素 A>
- 役割: <役割>
- インタフェース: <入出力 / API>
- 物理境界: <process / network / module>

### <構成要素 B>
- 役割: <役割>
- インタフェース: <入出力 / API>
- 物理境界: <process / network / module>

## 採用しない選択肢
- <選択肢>: 採用しない。理由: <理由>

## 5 層 defense-in-depth との bind
- 層 A: <該当する compile-time 規律>
- 層 B: <該当する lint / policy 規律>
- 層 C: <該当する integration test 規律>
- 層 D: <該当する runtime 規律>
- 層 E: <該当する物理規律>
- 層 F: <該当する数学的規律 / formal 関連時のみ>

## 強制機構との bind
- 本設計は [<axis> 強制機構](../../04_詳細設計/02_強制機構/<axis>強制機構.md) の以下経路で enforce:
    - <経路 1>
    - <経路 2>

## 適合仕様との bind
- 本設計の動的不変量は [<関連適合仕様>](../../04_詳細設計/01_適合仕様/<相対パス>.md) で機械可読化。

## 関連参照
- [<要件定義側の対応 doc>](../../02_要件定義/<相対パス>.md)
- [<詳細設計側の対応 doc>](../../04_詳細設計/<相対パス>.md)
- [<同フェーズ内の隣接 doc>](<相対パス>.md)
