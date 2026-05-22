\* k1s0-proof: PROOF-meta-tsafe-089 -> IMPL-meta-0001
\* meta_safety_temporal.tla
\* meta 軸 — 軸 registry 軸数 monotone 増加（削除禁止）temporal safety
\* obligation_id: meta_safety_089
\* cell_state: v1_accepted_with_assumption（Apalache 検証後に v1_baseline_verified に更新する）
\* property: AxisRegistryMonotone — 登録済み軸は削除されない（軸数は monotone 増加のみ）
---- MODULE meta_safety_temporal ----
\* 標準ライブラリ Naturals と TLC をインポートする（整数演算とモデル検査に使用）
EXTENDS Naturals, TLC

\* 定数: 軸数の最大値を定義する（状態空間を有限に抑えるため・cap v1=20 に対応する）
MaxAxes == 20

\* 変数宣言: axis_count は現在の登録済み軸数を保持する（Apalache type annotation: Int）
VARIABLE
    \* @type: Int;
    axis_count

\* 変数宣言: prev_axis_count は前ステップの軸数を保持する（monotone 検証用）
VARIABLE
    \* @type: Int;
    prev_axis_count

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* axis_count は 0 以上 MaxAxes 以下の整数でなければならない
    /\ axis_count >= 0 /\ axis_count <= MaxAxes
    \* prev_axis_count は 0 以上 MaxAxes 以下の整数でなければならない
    /\ prev_axis_count >= 0 /\ prev_axis_count <= MaxAxes

\* 初期状態: 軸数 0 から開始する（登録前の初期状態）
Init ==
    \* axis_count の初期値は 0（軸未登録）に設定する
    /\ axis_count = 0
    \* prev_axis_count の初期値は 0（ベースライン）に設定する
    /\ prev_axis_count = 0

\* アクション: 新しい軸を登録する（軸数を 1 増加させる）
RegisterAxis ==
    \* cap 上限未満の場合のみ軸を追加できる（cap v1=20 制約）
    /\ axis_count < MaxAxes
    \* prev_axis_count を現在の axis_count に記録する（monotone 検証用）
    /\ prev_axis_count' = axis_count
    \* 軸数を 1 増加させる（登録のみ許可・削除は禁止する）
    /\ axis_count' = axis_count + 1

\* アクション: 軸数を維持する（no-op 遷移）
MaintainAxes ==
    \* 軸数を変化させない
    /\ axis_count' = axis_count
    \* prev_axis_count も変化させない
    /\ prev_axis_count' = prev_axis_count

\* 全遷移の定義: 軸登録または維持のいずれかを選択する
Next ==
    \* 軸登録アクションを選択する
    \/ RegisterAxis
    \* 軸数維持アクションを選択する
    \/ MaintainAxes

\* safety invariant: 軸数は前ステップ以上でなければならない（登録済み軸は削除されない）
\* meta 適合仕様の AxisRegistryMonotone 規定の形式化
AxisRegistryMonotone ==
    \* axis_count は prev_axis_count 以上でなければならない（monotone 増加のみ許可する）
    axis_count >= prev_axis_count

\* 変数タプル演算子: Apalache 型推論のための明示的 tuple 宣言（同型変数の曖昧性回避）
\* @type: <<Int, Int>>;
vars == <<axis_count, prev_axis_count>>

\* spec 定義: 初期状態 + 次状態遷移の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_vars

\* 定理: Spec が成立すれば AxisRegistryMonotone が常に成立する
THEOREM Spec => []AxisRegistryMonotone
====
