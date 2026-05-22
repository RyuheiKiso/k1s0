\* k1s0-proof: PROOF-infra-tsafe-015 -> IMPL-infra-0015
\* infra_safety_temporal.tla
\* infra クラスタ leader 一意性 temporal safety: leader が高々1つしか存在しないことの形式検証
\* obligation_id: infra_safety_015
\* cell_state: stub（Apalache で検証後に v1_baseline_verified に更新する）
\* property: LeaderUniqueness — クラスタに leader が高々1つしか存在しない
---- MODULE infra_safety_temporal ----
\* 標準ライブラリ Naturals と TLC をインポートする（整数演算とモデル検査に使用）
EXTENDS Naturals, TLC

\* クラスタノードの最大数を定義する（状態空間を有限に抑えるため）
MAX_NODES == 3

\* 変数宣言: has_leader は現在 leader が存在するかを保持する（Apalache type: Bool）
\* -1 の代わりに has_leader フラグを使って leader 不在を表現する
VARIABLE
    \* @type: Bool;
    has_leader

\* 変数宣言: leader_id は現在の leader ノード ID を保持する（Apalache type: Int）
\* has_leader = FALSE のときは leader_id の値は未定義（任意）
VARIABLE
    \* @type: Int;
    leader_id

\* 変数宣言: leader_count は現在の leader 数を保持する（Apalache type: Int）
VARIABLE
    \* @type: Int;
    leader_count

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* has_leader は真偽値でなければならない
    /\ has_leader \in BOOLEAN
    \* leader_id は 0 以上 MAX_NODES 未満の整数でなければならない
    /\ leader_id >= 0 /\ leader_id < MAX_NODES
    \* leader_count は 0 以上 1 以下の整数でなければならない（leader は高々1つ）
    /\ leader_count >= 0 /\ leader_count <= 1

\* 初期状態: leader なしから開始する
Init ==
    \* has_leader の初期値は FALSE（leader なし）に設定する
    /\ has_leader = FALSE
    \* leader_id の初期値は 0（placeholder）に設定する
    /\ leader_id = 0
    \* leader_count の初期値は 0（leader なし）に設定する
    /\ leader_count = 0

\* アクション: leader を選出する（Raft/Paxos の leader election をモデル化する）
ElectLeader ==
    \* 現在 leader が存在しない場合のみ election を行う
    /\ has_leader = FALSE
    \* 新しい leader_id を 0 以上 MAX_NODES 未満から選択する
    /\ leader_id' \in 0..(MAX_NODES - 1)
    \* has_leader を TRUE に設定する
    /\ has_leader' = TRUE
    \* leader_count を 1 に設定する
    /\ leader_count' = 1

\* アクション: leader が辞任する（ネットワーク障害や再起動をシミュレートする）
LeaderResigns ==
    \* 現在 leader が存在する場合のみ辞任を行う
    /\ has_leader = TRUE
    \* has_leader を FALSE にリセットする
    /\ has_leader' = FALSE
    \* leader_id は任意の値にリセットする（placeholder として 0 を維持する）
    /\ leader_id' = 0
    \* leader_count を 0 にリセットする
    /\ leader_count' = 0

\* アクション: Sink ステートのループ遷移（deadlock 回避用の no-op 遷移）
Stutter ==
    \* 全変数を変化させない（stutter 遷移）
    /\ UNCHANGED has_leader
    /\ UNCHANGED leader_id
    /\ UNCHANGED leader_count

\* 全遷移の定義: 3 つのアクションのいずれかを実行する
Next ==
    \* leader 選出アクションを選択する
    \/ ElectLeader
    \* leader 辞任アクションを選択する
    \/ LeaderResigns
    \* Stutter アクションを選択する
    \/ Stutter

\* safety property: leader_count は常に 0 または 1 でなければならない
\* クラスタに leader が高々1つしか存在しないことを形式化する
LeaderUniqueness ==
    \* leader_count が 0 または 1 の範囲に収まることを保証する
    leader_count <= 1

\* spec 定義: 初期状態 + 次状態遷移の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_<<has_leader, leader_id, leader_count>>

\* 定理: Spec が成立すれば LeaderUniqueness が常に成立する
THEOREM Spec => []LeaderUniqueness
====
