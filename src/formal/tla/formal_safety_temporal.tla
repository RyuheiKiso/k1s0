\* k1s0-proof: PROOF-formal-tsafe-044 -> IMPL-formal-0044
\* formal_safety_temporal.tla
\* formal proof monotonicity temporal safety: verified cell が unverified に戻れないことの形式検証
\* obligation_id: formal_safety_044
\* cell_state: stub（Apalache で検証後に v1_baseline_verified に更新する）
\* property: ProofMonotonicity — verified セルは unverified に戻れない（proof は不可逆）
---- MODULE formal_safety_temporal ----
\* 標準ライブラリ Naturals と TLC をインポートする（整数演算とモデル検査に使用）
EXTENDS Naturals, TLC

\* proof 状態を整数でエンコードする（Apalache は Str 型の選択肢を限定的にしか扱えないため）
\* STATE_UNVERIFIED=0: 未検証状態、STATE_VERIFIED=1: 検証済み状態
STATE_UNVERIFIED == 0
STATE_VERIFIED == 1

\* 変数宣言: proof_state は現在の proof セル状態を保持する（Apalache type: Int）
VARIABLE
    \* @type: Int;
    proof_state

\* 変数宣言: verification_locked は検証ロック状態（一度 verified になると locked）を保持する（Apalache type: Bool）
VARIABLE
    \* @type: Bool;
    verification_locked

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* proof_state は 0 または 1 の整数でなければならない（STATE_UNVERIFIED/VERIFIED）
    /\ proof_state >= 0 /\ proof_state <= 1
    \* verification_locked は真偽値でなければならない
    /\ verification_locked \in BOOLEAN

\* 初期状態: proof は unverified 状態、ロックなしから開始する
Init ==
    \* proof_state の初期値は STATE_UNVERIFIED（未検証）に設定する
    /\ proof_state = STATE_UNVERIFIED
    \* verification_locked の初期値は FALSE（ロックなし）に設定する
    /\ verification_locked = FALSE

\* アクション: proof を検証する（Apalache/Lean4 による proof の検証をモデル化する）
VerifyProof ==
    \* unverified 状態の proof のみ検証できる
    /\ proof_state = STATE_UNVERIFIED
    \* proof_state を verified に更新する
    /\ proof_state' = STATE_VERIFIED
    \* verification_locked を TRUE に設定する（不可逆なロック）
    /\ verification_locked' = TRUE

\* アクション: Sink ステートのループ遷移（verified から unverified への遷移は定義しない）
\* verified → unverified の遷移は ProofMonotonicity を破るため意図的に除外する
Stutter ==
    \* 全変数を変化させない（stutter 遷移）
    /\ UNCHANGED proof_state
    /\ UNCHANGED verification_locked

\* 全遷移の定義: 2 つのアクションのいずれかを実行する
Next ==
    \* proof 検証アクションを選択する
    \/ VerifyProof
    \* Stutter アクションを選択する
    \/ Stutter

\* safety property: verification_locked = TRUE のとき proof_state = STATE_VERIFIED でなければならない
\* verified セルが unverified に戻れないことを形式化する
ProofMonotonicity ==
    \* verification_locked が TRUE の場合、proof_state は verified であることを保証する
    verification_locked = TRUE => proof_state = STATE_VERIFIED

\* spec 定義: 初期状態 + 次状態遷移の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_<<proof_state, verification_locked>>

\* 定理: Spec が成立すれば ProofMonotonicity が常に成立する
THEOREM Spec => []ProofMonotonicity
====
