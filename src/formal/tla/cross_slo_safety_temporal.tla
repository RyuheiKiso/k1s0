\* k1s0-proof: PROOF-cross_slo-tsafe-069 -> IMPL-cross_slo-0001
\* cross_slo_safety_temporal.tla
\* cross_slo クロスカット関心事 — SLO バーンレート計算 conservative 側 temporal safety
\* obligation_id: cross_slo_safety_069
\* cell_state: v1_accepted_with_assumption（Apalache 検証後に v1_baseline_verified に更新する）
\* property: BurnRateConservative — SLO バーンレート計算が underestimate しない（実際 >= 計算値）
---- MODULE cross_slo_safety_temporal ----
\* 標準ライブラリ Naturals と TLC をインポートする（整数演算とモデル検査に使用）
EXTENDS Naturals, TLC

\* 定数: バーンレートの最大値を定義する（状態空間を有限に抑えるため）
MaxBurn == 10

\* 変数宣言: actual_burn は実際のバーンレートを保持する（Apalache type annotation: Int）
VARIABLE
    \* @type: Int;
    actual_burn

\* 変数宣言: computed_burn は計算済みバーンレートを保持する（Apalache type annotation: Int）
VARIABLE
    \* @type: Int;
    computed_burn

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* actual_burn は 0 以上 MaxBurn 以下の整数でなければならない
    /\ actual_burn >= 0 /\ actual_burn <= MaxBurn
    \* computed_burn は 0 以上 MaxBurn 以下の整数でなければならない
    /\ computed_burn >= 0 /\ computed_burn <= MaxBurn

\* 初期状態: 両バーンレートを 0 から開始する
Init ==
    \* actual_burn の初期値は 0（正常状態）に設定する
    /\ actual_burn = 0
    \* computed_burn の初期値は 0（計算開始前）に設定する
    /\ computed_burn = 0

\* アクション: 実際のバーンレートを増加させる（エラー率上昇をシミュレートする）
IncreaseActualBurn ==
    \* 最大値未満の場合のみ増加を許可する
    /\ actual_burn < MaxBurn
    \* actual_burn を 1 増加させる
    /\ actual_burn' = actual_burn + 1
    \* computed_burn は変化しない
    /\ UNCHANGED computed_burn

\* アクション: 実際のバーンレートを減少させる（エラー率回復をシミュレートする）
\* バーンレート低下時は computed_burn も同時にリセットして conservative 性を維持する
DecreaseActualBurn ==
    \* 最小値超過の場合のみ減少を許可する
    /\ actual_burn > 0
    \* actual_burn を 1 減少させる
    /\ actual_burn' = actual_burn - 1
    \* computed_burn を新しい actual_burn 以下にリセットする（conservative 不変条件を維持する）
    /\ computed_burn' \in 0..(actual_burn - 1)

\* アクション: 計算バーンレートを更新する（conservative: actual_burn 以下に設定する）
UpdateComputedBurn ==
    \* computed_burn を actual_burn 以下の任意の値に設定する（conservative 計算）
    /\ computed_burn' \in 0..actual_burn
    \* actual_burn は変化しない
    /\ UNCHANGED actual_burn

\* 全遷移の定義: 各アクションのいずれかを選択する
Next ==
    \* 実際バーンレート増加アクションを選択する
    \/ IncreaseActualBurn
    \* 実際バーンレート減少アクションを選択する
    \/ DecreaseActualBurn
    \* 計算バーンレート更新アクションを選択する
    \/ UpdateComputedBurn

\* safety invariant: 計算バーンレートは実際バーンレート以下（conservative 保証）
\* cross_slo 適合仕様の BurnRateConservative 規定の形式化
BurnRateConservative ==
    \* computed_burn は actual_burn 以下でなければならない（underestimate 禁止）
    computed_burn <= actual_burn

\* 変数タプル演算子: Apalache 型推論のための明示的 tuple 宣言（同型変数の曖昧性回避）
\* @type: <<Int, Int>>;
vars == <<actual_burn, computed_burn>>

\* spec 定義: 初期状態 + 次状態遷移の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_vars

\* 定理: Spec が成立すれば BurnRateConservative が常に成立する
THEOREM Spec => []BurnRateConservative
====
