\* k1s0-proof: PROOF-cross_fsm-tsafe-064 -> IMPL-cross_fsm-0001
\* cross_fsm_safety_temporal.tla
\* cross_fsm クロスカット関心事 — FSM 遷移の定義済み状態集合外逸脱禁止 temporal safety
\* obligation_id: cross_fsm_safety_064
\* cell_state: v1_accepted_with_assumption（Apalache 検証後に v1_baseline_verified に更新する）
\* property: FSMStateSpace — FSM の遷移は定義済み状態集合の外に出ない
---- MODULE cross_fsm_safety_temporal ----
\* 標準ライブラリ Naturals と TLC をインポートする（整数演算とモデル検査に使用）
EXTENDS Naturals, TLC

\* 変数宣言: current_state は FSM の現在状態を保持する（Apalache type annotation: Str）
VARIABLE
    \* @type: Str;
    current_state

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* current_state は規定された 4 状態の文字列でなければならない
    /\ current_state \in {"Init", "Processing", "Complete", "Error"}

\* 初期状態: Init 状態から開始する
Init ==
    \* current_state の初期値は "Init"（初期状態）に設定する
    /\ current_state = "Init"

\* アクション: 処理を開始する（Init -> Processing）
StartProcessing ==
    \* Init 状態の場合のみ処理を開始できる
    /\ current_state = "Init"
    \* Processing 状態に遷移する
    /\ current_state' = "Processing"

\* アクション: 処理を完了する（Processing -> Complete）
CompleteProcessing ==
    \* Processing 状態の場合のみ完了できる
    /\ current_state = "Processing"
    \* Complete 状態に遷移する
    /\ current_state' = "Complete"

\* アクション: エラーが発生する（Processing -> Error）
EncounterError ==
    \* Processing 状態の場合のみエラー遷移できる
    /\ current_state = "Processing"
    \* Error 状態に遷移する
    /\ current_state' = "Error"

\* アクション: リトライする（Error -> Processing）
RetryProcessing ==
    \* Error 状態の場合のみリトライできる
    /\ current_state = "Error"
    \* Processing 状態に再遷移する
    /\ current_state' = "Processing"

\* アクション: 初期状態にリセットする（Complete/Error -> Init）
ResetToInit ==
    \* Complete または Error 状態の場合のみリセットできる
    /\ current_state \in {"Complete", "Error"}
    \* Init 状態に遷移する
    /\ current_state' = "Init"

\* 全遷移の定義: 各アクションのいずれかを選択する
Next ==
    \* 処理開始アクションを選択する
    \/ StartProcessing
    \* 処理完了アクションを選択する
    \/ CompleteProcessing
    \* エラー発生アクションを選択する
    \/ EncounterError
    \* リトライアクションを選択する
    \/ RetryProcessing
    \* リセットアクションを選択する
    \/ ResetToInit

\* safety invariant: FSM は常に定義済み状態集合の中に存在する
\* cross_fsm 適合仕様の FSMStateSpace 規定の形式化
FSMStateSpace ==
    \* current_state は常に定義済み状態集合 {"Init","Processing","Complete","Error"} に属する
    current_state \in {"Init", "Processing", "Complete", "Error"}

\* 変数タプル演算子: Apalache 型推論のための明示的 tuple 宣言（単一 Str 変数の曖昧性回避）
\* @type: <<Str>>;
vars == <<current_state>>

\* spec 定義: 初期状態 + 次状態遷移の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_vars

\* 定理: Spec が成立すれば FSMStateSpace が常に成立する
THEOREM Spec => []FSMStateSpace
====
