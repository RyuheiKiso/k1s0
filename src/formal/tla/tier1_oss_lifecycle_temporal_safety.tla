\* k1s0-proof: PROOF-tier1-tsafe-004 -> IMPL-tier1-0008
\* tier1_oss_lifecycle_temporal_safety.tla
\* tier1 OSS ライフサイクル temporal safety: lifecycle signal が eventually アクションに解決される保証
\* obligation_id: tier1_oss_lifecycle_tsp
\* cell_state: stub（Apalache で検証後に v1_baseline_verified に更新する）
\* property: ActionRequiredForResolution — resolved 状態には必ずアクション実行が先行する
---- MODULE tier1_oss_lifecycle_temporal_safety ----
\* 標準ライブラリ Naturals と TLC をインポートする（整数演算とモデル検査に使用）
EXTENDS Naturals, TLC

\* lifecycle signal の状態集合: pending（未処理）/ processing（処理中）/ resolved（解決済み）
SignalState == {"pending", "processing", "resolved"}

\* 変数宣言: signal_state は OSS lifecycle signal の現在状態を保持する（Apalache type: Str）
VARIABLE
    \* @type: Str;
    signal_state

\* 変数宣言: action_taken は対応アクションが実行済みかを示すフラグ（Apalache type: Bool）
VARIABLE
    \* @type: Bool;
    action_taken

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* signal_state は SignalState の集合内の値でなければならない
    /\ signal_state \in SignalState
    \* action_taken は真偽値でなければならない
    /\ action_taken \in BOOLEAN

\* 初期状態: lifecycle signal が pending 状態でアクション未実行から開始する
Init ==
    \* signal_state の初期値は "pending"（未処理状態）に設定する
    /\ signal_state = "pending"
    \* action_taken の初期値は FALSE（アクション未実行）に設定する
    /\ action_taken = FALSE

\* アクション: signal の処理を開始する（pending → processing 遷移）
StartProcessing ==
    \* pending 状態の signal のみ処理を開始できる
    /\ signal_state = "pending"
    \* signal_state を "processing" に遷移させる
    /\ signal_state' = "processing"
    \* action_taken は変化しない（まだアクションを実行していない）
    /\ UNCHANGED action_taken

\* アクション: 対応アクションを実行する（processing 状態でアクションを完了させる）
TakeAction ==
    \* processing 状態の signal に対してのみアクションを実行できる
    /\ signal_state = "processing"
    \* アクションをまだ実行していない場合のみ実行する（冪等性のため）
    /\ action_taken = FALSE
    \* action_taken フラグを TRUE に設定する
    /\ action_taken' = TRUE
    \* signal_state は変化しない（resolve は別アクション）
    /\ UNCHANGED signal_state

\* アクション: signal を解決済みにする（processing かつアクション済みの場合に resolved へ）
ResolveSignal ==
    \* processing 状態であることを前提とする
    /\ signal_state = "processing"
    \* アクションが実行済みであることを前提とする（アクションなしに resolved 禁止）
    /\ action_taken = TRUE
    \* signal_state を "resolved" に遷移させる（不可逆）
    /\ signal_state' = "resolved"
    \* action_taken は変化しない（TRUE のまま維持）
    /\ UNCHANGED action_taken

\* 全遷移の定義: 3 つのアクションのいずれかを実行する
Next ==
    \* StartProcessing アクションを選択する
    \/ StartProcessing
    \* TakeAction アクションを選択する
    \/ TakeAction
    \* ResolveSignal アクションを選択する
    \/ ResolveSignal

\* safety property: resolved 状態には必ずアクション実行が先行する
\* 08_OSSライフサイクル適合仕様の action_required_for_resolution を形式化する
ActionRequiredForResolution ==
    \* signal が resolved 状態になった場合、action_taken は必ず TRUE でなければならない
    signal_state = "resolved" => action_taken = TRUE

\* spec 定義: 初期状態 + 次状態遷移 + 弱い公平性条件の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_<<signal_state, action_taken>>
    \* 弱い公平性: 有効なアクションが存在する場合、eventually 実行されることを保証する
    /\ WF_<<signal_state, action_taken>>(Next)

\* 定理: Spec が成立すれば ActionRequiredForResolution が常に成立する
THEOREM Spec => []ActionRequiredForResolution
====
