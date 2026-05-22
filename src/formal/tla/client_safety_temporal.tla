\* k1s0-proof: PROOF-client-tsafe-034 -> IMPL-client-0034
\* client_safety_temporal.tla
\* client SDK 配布 temporal safety: 配布 artifact が cosign signed でなければならないことの形式検証
\* obligation_id: client_safety_034
\* cell_state: stub（Apalache で検証後に v1_baseline_verified に更新する）
\* property: SDKProvenance — 配布された SDK artifact は cosign signed でなければならない
---- MODULE client_safety_temporal ----
\* 標準ライブラリ Naturals と TLC をインポートする（整数演算とモデル検査に使用）
EXTENDS Naturals, TLC

\* artifact 状態を整数でエンコードする（Apalache は Str 型の選択肢を限定的にしか扱えないため）
\* STATE_UNSIGNED=0: 未署名状態、STATE_SIGNED=1: 署名済み状態、STATE_DISTRIBUTED=2: 配布済み状態
STATE_UNSIGNED == 0
STATE_SIGNED == 1
STATE_DISTRIBUTED == 2

\* 変数宣言: artifact_state は現在の artifact 状態を保持する（Apalache type: Int）
VARIABLE
    \* @type: Int;
    artifact_state

\* 変数宣言: signature_present は cosign 署名が存在するかを保持する（Apalache type: Bool）
VARIABLE
    \* @type: Bool;
    signature_present

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* artifact_state は 0 以上 2 以下の整数でなければならない（STATE_UNSIGNED/SIGNED/DISTRIBUTED）
    /\ artifact_state >= 0 /\ artifact_state <= 2
    \* signature_present は真偽値でなければならない
    /\ signature_present \in BOOLEAN

\* 初期状態: artifact は unsigned 状態、署名なしから開始する
Init ==
    \* artifact_state の初期値は STATE_UNSIGNED（未署名）に設定する
    /\ artifact_state = STATE_UNSIGNED
    \* signature_present の初期値は FALSE（署名なし）に設定する
    /\ signature_present = FALSE

\* アクション: artifact に cosign 署名を付与する（cosign sign をモデル化する）
SignArtifact ==
    \* unsigned 状態の artifact のみ署名できる
    /\ artifact_state = STATE_UNSIGNED
    \* artifact_state を signed に更新する
    /\ artifact_state' = STATE_SIGNED
    \* signature_present を TRUE に設定する
    /\ signature_present' = TRUE

\* アクション: 署名済み artifact を配布する（Harbor/OCI registry への push をモデル化する）
DistributeArtifact ==
    \* signed 状態の artifact のみ配布できる（署名なしの配布は許可しない）
    /\ artifact_state = STATE_SIGNED
    \* signature_present が TRUE の場合のみ配布できる
    /\ signature_present = TRUE
    \* artifact_state を distributed に更新する
    /\ artifact_state' = STATE_DISTRIBUTED
    \* signature_present は TRUE のまま維持する
    /\ UNCHANGED signature_present

\* アクション: Sink ステートのループ遷移（deadlock 回避用の no-op 遷移）
Stutter ==
    \* 全変数を変化させない（stutter 遷移）
    /\ UNCHANGED artifact_state
    /\ UNCHANGED signature_present

\* 全遷移の定義: 3 つのアクションのいずれかを実行する
Next ==
    \* artifact 署名アクションを選択する
    \/ SignArtifact
    \* artifact 配布アクションを選択する
    \/ DistributeArtifact
    \* Stutter アクションを選択する
    \/ Stutter

\* safety property: artifact_state = STATE_DISTRIBUTED のとき signature_present = TRUE でなければならない
\* 配布された SDK artifact が cosign signed であることを形式化する
SDKProvenance ==
    \* distributed 状態の場合は署名が存在することを保証する
    artifact_state = STATE_DISTRIBUTED => signature_present = TRUE

\* spec 定義: 初期状態 + 次状態遷移の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_<<artifact_state, signature_present>>

\* 定理: Spec が成立すれば SDKProvenance が常に成立する
THEOREM Spec => []SDKProvenance
====
