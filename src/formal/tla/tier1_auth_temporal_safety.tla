\* tier1_auth_temporal_safety.tla
\* tier1 認証 temporal safety: auth_class × step-up FSM における無言格下げ禁止の形式検証
\* obligation_id: tier1_auth_tsp
\* cell_state: stub（Apalache で検証後に v1_baseline_verified に更新する）
\* property: NoSilentDemotion — 認証レベルが自発的に下がらないことを保証する
---- MODULE tier1_auth_temporal_safety ----
\* 標準ライブラリ Naturals と TLC をインポートする（整数演算とモデル検査に使用）
EXTENDS Naturals, TLC

\* 認証レベルの集合: 04_認証適合仕様の 5 auth_class (0-4) に対応する
AuthLevel == {0, 1, 2, 3, 4}

\* 変数宣言: auth_level は現在の認証レベルを保持する（Apalache type annotation: Int）
VARIABLE
    \* @type: Int;
    auth_level

\* 変数宣言: step_up_required は step-up 遷移の要求フラグを保持する（Apalache type annotation: Bool）
VARIABLE
    \* @type: Bool;
    step_up_required

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* auth_level は 0-4 の整数値でなければならない
    /\ auth_level \in AuthLevel
    \* step_up_required は真偽値でなければならない
    /\ step_up_required \in BOOLEAN

\* 初期状態: 最低認証レベルから開始し step-up 要求なしとする
Init ==
    \* auth_level の初期値は 0（最低レベル）に設定する
    /\ auth_level = 0
    \* step_up_required の初期値は FALSE（要求なし）に設定する
    /\ step_up_required = FALSE

\* アクション: step-up 要求を発生させる（外部トリガーによるレベル昇格要求）
RequestStepUp ==
    \* step-up 要求が既に出ていないことを前提とする
    /\ step_up_required = FALSE
    \* 現在が最大レベルでない場合のみ step-up 要求を発生させる
    /\ auth_level < 4
    \* step_up_required フラグを TRUE に変更する
    /\ step_up_required' = TRUE
    \* auth_level は変化しない
    /\ UNCHANGED auth_level

\* アクション: step-up を実行する（認証レベルを上昇させる）
StepUp ==
    \* step-up 要求フラグが立っていることを前提とする
    /\ step_up_required = TRUE
    \* 新しい auth_level は現在レベルより大きい値（上昇のみ許可）
    /\ auth_level' \in (auth_level+1..4)
    \* step_up_required フラグをクリアする
    /\ step_up_required' = FALSE

\* アクション: 同一レベルの維持遷移（stuttering 以外の実際の no-op 遷移）
MaintainLevel ==
    \* step-up 要求がない状態で現在レベルを維持する
    /\ step_up_required = FALSE
    \* auth_level は変化しない
    /\ UNCHANGED auth_level
    \* step_up_required も変化しない
    /\ UNCHANGED step_up_required

\* 全遷移の定義: RequestStepUp / StepUp / MaintainLevel のいずれかを実行する
Next ==
    \* RequestStepUp アクションを選択する
    \/ RequestStepUp
    \* StepUp アクションを選択する
    \/ StepUp
    \* MaintainLevel アクションを選択する
    \/ MaintainLevel

\* safety property: 認証レベルが自発的に下がることはない（no_silent_demotion）
\* 各遷移において auth_level' は auth_level 以上でなければならない
NoSilentDemotion == [][auth_level' >= auth_level]_<<auth_level, step_up_required>>

\* spec 定義: 初期状態 + 次状態遷移の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_<<auth_level, step_up_required>>

\* 定理: Spec が成立すれば NoSilentDemotion が常に成立する
THEOREM Spec => []NoSilentDemotion
====
