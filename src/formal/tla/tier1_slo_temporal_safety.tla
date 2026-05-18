\* tier1_slo_temporal_safety.tla
\* tier1 SLO temporal safety: burn-rate アラートによるエラーバジェット凍結の形式検証
\* obligation_id: tier1_slo_tsp
\* cell_state: stub（Apalache で検証後に v1_baseline_verified に更新する）
\* property: FreezeOnBurn — burn_rate > 2 の場合にエラーバジェットを必ず凍結する
---- MODULE tier1_slo_temporal_safety ----
\* 標準ライブラリ Naturals と TLC をインポートする（整数演算とモデル検査に使用）
EXTENDS Naturals, TLC

\* 定数: burn_rate の最大値を定義する（状態空間を有限に抑えるため）
MAX_BURN_RATE == 10

\* 変数宣言: burn_rate は現在の burn rate 値を保持する（Apalache type: Int）
VARIABLE
    \* @type: Int;
    burn_rate

\* 変数宣言: budget_frozen はエラーバジェット凍結フラグを保持する（Apalache type: Bool）
VARIABLE
    \* @type: Bool;
    budget_frozen

\* 変数宣言: alert_fired はアラート発火フラグを保持する（Apalache type: Bool）
VARIABLE
    \* @type: Bool;
    alert_fired

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* burn_rate は 0 以上 MAX_BURN_RATE 以下の整数でなければならない
    /\ burn_rate >= 0 /\ burn_rate <= MAX_BURN_RATE
    \* budget_frozen は真偽値でなければならない
    /\ budget_frozen \in BOOLEAN
    \* alert_fired は真偽値でなければならない
    /\ alert_fired \in BOOLEAN

\* 初期状態: burn_rate=0, 凍結なし, アラート未発火から開始する
Init ==
    \* burn_rate の初期値は 0（正常状態）に設定する
    /\ burn_rate = 0
    \* エラーバジェットは初期状態で凍結されていない
    /\ budget_frozen = FALSE
    \* アラートは初期状態で発火していない
    /\ alert_fired = FALSE

\* アクション: burn_rate を増加させる（エラー率の上昇をシミュレートする）
IncreaseBurnRate ==
    \* 最大値未満の場合のみ burn_rate を増加させる
    /\ burn_rate < MAX_BURN_RATE
    \* burn_rate を 1 増やす
    /\ burn_rate' = burn_rate + 1
    \* budget_frozen は変化しない
    /\ UNCHANGED budget_frozen
    \* alert_fired は変化しない
    /\ UNCHANGED alert_fired

\* アクション: burn_rate を減少させる（エラー率の回復をシミュレートする）
DecreaseBurnRate ==
    \* 最小値超過の場合のみ burn_rate を減少させる
    /\ burn_rate > 0
    \* burn_rate を 1 減らす
    /\ burn_rate' = burn_rate - 1
    \* budget_frozen は変化しない
    /\ UNCHANGED budget_frozen
    \* alert_fired は変化しない
    /\ UNCHANGED alert_fired

\* アクション: アラート発火（burn_rate > 2 で budget を凍結する）
FireAlert ==
    \* burn_rate が閾値 2 を超えた場合のみアラートを発火させる
    /\ burn_rate > 2
    \* まだアラートが発火していない場合のみ実行する
    /\ alert_fired = FALSE
    \* アラート発火フラグを TRUE に設定する
    /\ alert_fired' = TRUE
    \* エラーバジェットを凍結する（07_SLO 適合仕様の freeze_on_burn 規定）
    /\ budget_frozen' = TRUE
    \* burn_rate は変化しない
    /\ UNCHANGED burn_rate

\* アクション: 凍結解除（burn_rate が正常範囲に戻った場合に凍結を解除する）
Unfreeze ==
    \* 凍結中であることを前提とする
    /\ budget_frozen = TRUE
    \* burn_rate が 1 以下（正常域）に回復した場合のみ解除する
    /\ burn_rate <= 1
    \* エラーバジェット凍結を解除する
    /\ budget_frozen' = FALSE
    \* アラートフラグをクリアする
    /\ alert_fired' = FALSE
    \* burn_rate は変化しない
    /\ UNCHANGED burn_rate

\* 全遷移の定義: 4 つのアクションのいずれかを実行する
Next ==
    \* IncreaseBurnRate アクションを選択する
    \/ IncreaseBurnRate
    \* DecreaseBurnRate アクションを選択する
    \/ DecreaseBurnRate
    \* FireAlert アクションを選択する
    \/ FireAlert
    \* Unfreeze アクションを選択する
    \/ Unfreeze

\* safety property: burn_rate > 2 の場合、budget_frozen は必ず TRUE でなければならない
\* 07_SLO 適合仕様の freeze_on_burn 規約の形式化
FreezeOnBurn ==
    \* burn_rate > 2 ならば budget_frozen = TRUE が成立することを主張する
    burn_rate > 2 => budget_frozen = TRUE

\* spec 定義: 初期状態 + 次状態遷移の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_<<burn_rate, budget_frozen, alert_fired>>

\* 定理: Spec が成立すれば FreezeOnBurn が常に成立する
THEOREM Spec => []FreezeOnBurn
====
