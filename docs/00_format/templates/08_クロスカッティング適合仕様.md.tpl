---
id: detail.<primary_axis>.<slug>
axis: <primary_axis>
phase: cross_cutting
kind: cross_cut_spec
status: draft
depends_on: []
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes: []
---

# <主題> クロスカッティング適合仕様（v1）

## 位置づけ
- <bind する複数軸の spec を列挙し、双方向 lock 関係を明示>。
- <主たる動機 / 物理的不可避性 / 達成すべき性質>。

## 不可避性
- <この設計が唯一の物理的解である根拠>。
- <代替案を採用しない理由>。

## 役割

### 役割 <X-A>: <主機能 A>
- <仕様>
- <制約>
- <enforce 経路>

### 役割 <X-A'>: <代替経路 / fallback>
- <仕様>
- <制約>
- <enforce 経路>

### 役割 <X-B>: <主機能 B>
- <仕様>
- <制約>
- <enforce 経路>

### 役割 <X-C>: <fallback / capability_class>
- <仕様>
- <fallback 経路>
- <SLO 別系統登録>

## 採用しない設計
- <候補>: 採用しない。理由: <理由>。例外: <なければ「なし」>

## 軸間 bind
- [<bind 軸 1>](<相対パス>.md): <bind 内容>
- [<bind 軸 2>](<相対パス>.md): <bind 内容>
- [<bind 軸 3>](<相対パス>.md): <bind 内容>

## 5 層 defense-in-depth
- 層 A: compile（<軸名> の schema check）
- 層 B: lint（<軸名> の Conftest / Semgrep）
- 層 C: integration test（<軸名> の conformance test）
- 層 D: runtime（admission webhook + drill）
- 層 E: 物理（cosign + Object Lock + 物理隔離）

## 強制機構との bind
- 各 bind 軸の強制機構に経路宣言:
    - [<軸 1> 強制機構](../02_強制機構/<軸 1>強制機構.md): <経路>
    - [<軸 2> 強制機構](../02_強制機構/<軸 2>強制機構.md): <経路>
- release_gate.lock.yaml cell: `<slug>` → 1.0.0 ship blocker

## v1 / v2 ライフサイクル
- v1: <ship スコープ>
- v2 廃止候補 / 拡張候補: <該当時のみ>。`oss_lifecycle.lock.yaml` の deprecation_clock に登録。

## 関連参照
- <関連 spec / 関連方針 への link>
