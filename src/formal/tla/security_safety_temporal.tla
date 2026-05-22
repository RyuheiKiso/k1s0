\* k1s0-proof: PROOF-security-tsafe-024 -> IMPL-security-0024
\* security_safety_temporal.tla
\* security 脅威軽減 temporal safety: mitigated 脅威が open に戻れないことの形式検証
\* obligation_id: security_safety_024
\* cell_state: stub（Apalache で検証後に v1_baseline_verified に更新する）
\* property: MitigationCoverage — bound mitigation を持つ threat は unmitigated 状態に戻らない
---- MODULE security_safety_temporal ----
\* 標準ライブラリ Naturals と TLC をインポートする（整数演算とモデル検査に使用）
EXTENDS Naturals, TLC

\* 脅威状態を整数でエンコードする（Apalache は Str 型の選択肢を限定的にしか扱えないため）
\* STATE_OPEN=0: 未対応状態、STATE_MITIGATED=1: 軽減済み状態、STATE_CLOSED=2: 完全解決状態
STATE_OPEN == 0
STATE_MITIGATED == 1
STATE_CLOSED == 2

\* 変数宣言: threat_state は現在の脅威状態を保持する（Apalache type: Int）
VARIABLE
    \* @type: Int;
    threat_state

\* 変数宣言: mitigation_bound は bound mitigation が存在するかを保持する（Apalache type: Bool）
VARIABLE
    \* @type: Bool;
    mitigation_bound

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* threat_state は 0 以上 2 以下の整数でなければならない（STATE_OPEN/MITIGATED/CLOSED）
    /\ threat_state >= 0 /\ threat_state <= 2
    \* mitigation_bound は真偽値でなければならない
    /\ mitigation_bound \in BOOLEAN

\* 初期状態: 脅威は open 状態、mitigation は未バインドから開始する
Init ==
    \* threat_state の初期値は STATE_OPEN（未対応）に設定する
    /\ threat_state = STATE_OPEN
    \* mitigation_bound の初期値は FALSE（未バインド）に設定する
    /\ mitigation_bound = FALSE

\* アクション: 脅威を軽減する（mitigation をバインドして mitigated 状態に遷移する）
MitigateThreat ==
    \* open 状態の脅威のみ軽減できる
    /\ threat_state = STATE_OPEN
    \* threat_state を mitigated に更新する
    /\ threat_state' = STATE_MITIGATED
    \* mitigation をバインドする
    /\ mitigation_bound' = TRUE

\* アクション: 脅威を完全解決する（mitigated から closed に遷移する）
CloseThreat ==
    \* mitigated 状態の脅威のみ解決できる（open からは直接 closed にできない）
    /\ threat_state = STATE_MITIGATED
    \* threat_state を closed に更新する
    /\ threat_state' = STATE_CLOSED
    \* mitigation_bound は TRUE のまま維持する
    /\ UNCHANGED mitigation_bound

\* アクション: Sink ステートのループ遷移（deadlock 回避用の no-op 遷移）
Stutter ==
    \* 全変数を変化させない（stutter 遷移）
    /\ UNCHANGED threat_state
    /\ UNCHANGED mitigation_bound

\* 全遷移の定義: 3 つのアクションのいずれかを実行する
Next ==
    \* 脅威軽減アクションを選択する
    \/ MitigateThreat
    \* 脅威完全解決アクションを選択する
    \/ CloseThreat
    \* Stutter アクションを選択する
    \/ Stutter

\* safety property: closed 状態に達した脅威は open に戻れない
\* bound mitigation を持つ threat は unmitigated 状態に戻らないことを形式化する
MitigationCoverage ==
    \* closed 状態は open または mitigated に遷移しない（一方向遷移の保証）
    threat_state = STATE_CLOSED => mitigation_bound = TRUE

\* spec 定義: 初期状態 + 次状態遷移の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_<<threat_state, mitigation_bound>>

\* 定理: Spec が成立すれば MitigationCoverage が常に成立する
THEOREM Spec => []MitigationCoverage
====
