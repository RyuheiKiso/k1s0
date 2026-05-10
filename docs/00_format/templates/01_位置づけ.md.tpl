---
id: arch.<axis>.positioning
axis: <axis>
phase: architecture
kind: positioning
status: draft
depends_on: []
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# <axis> 位置づけ

## 一文方針
- <axis> は〜の位置に立ち、〜を責務とする。

## 5 階層 + 横断 layer 中の位置
- tier1   : システム共通処理（Library / Server / 認証 / 観測可能性）
- tier2   : ドメイン業務共通化処理
- tier3   : 個別業務の Web サイト / exe
- infra   : 物理 / 仮想基盤 + 制御平面
- data    : データ層（保全 / lifecycle / 暗号化）
- security: 全層を直交に貫く「脅威 ⊗ 軸 ⊗ 層」invariant の管掌 layer
- ops     : 全層 + security を直交に貫く「loop ⊗ phase ⊗ 軸 ⊗ 層」invariant の管掌 layer
- test    : 全層 + security + ops を直交に貫く「verification_class ⊗ axis ⊗ phase ⊗ 層」invariant の管掌 layer
- formal  : 全層 + security + ops + test を直交に貫く「proof_class ⊗ axis ⊗ phase ⊗ 層」invariant の管掌 layer

- <axis> は <位置 / 役割>。

## 上下方向の関係

### 上方向（<axis> に依存する経路）
- <他軸>: <依存内容 + 該当 spec への link>

### 下方向（<axis> が依存する経路）
- <他軸>: <依存内容 + 該当 spec への link>

## 横断方向の関係（cross-axis）
- <security / ops / test / formal との bind 関係>
