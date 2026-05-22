\* k1s0-proof: PROOF-tier1-tsafe-003 -> IMPL-tier1-0005
\* tier1_key_mgmt_temporal_safety.tla
\* tier1 鍵管理 temporal safety: KEK ローテーションと暗号消去の安全性保証
\* obligation_id: tier1_key_mgmt_tsp
\* cell_state: stub（Apalache で検証後に v1_baseline_verified に更新する）
\* property: NoRawBytesInFlight — raw bytes が in-flight の間に destroyed 状態に遷移しない
---- MODULE tier1_key_mgmt_temporal_safety ----
\* 標準ライブラリ Naturals, Sequences と TLC をインポートする（整数演算・列操作・モデル検査）
EXTENDS Naturals, Sequences, TLC

\* 鍵の状態集合を定義する: active（稼働中）/ rotating（ローテーション中）/ destroyed（消去済み）
KeyState == {"active", "rotating", "destroyed"}

\* 変数宣言: kek_state は KEK の現在状態を保持する（Apalache type annotation: Str）
VARIABLE
    \* @type: Str;
    kek_state

\* 変数宣言: raw_bytes_in_flight は暗号化前 raw bytes が転送中かどうかを示す（Apalache type: Bool）
VARIABLE
    \* @type: Bool;
    raw_bytes_in_flight

\* 変数宣言: rotation_requested はローテーション要求フラグを保持する（Apalache type: Bool）
VARIABLE
    \* @type: Bool;
    rotation_requested

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* kek_state は KeyState の集合内の値でなければならない
    /\ kek_state \in KeyState
    \* raw_bytes_in_flight は真偽値でなければならない
    /\ raw_bytes_in_flight \in BOOLEAN
    \* rotation_requested は真偽値でなければならない
    /\ rotation_requested \in BOOLEAN

\* 初期状態: KEK が稼働中で raw bytes 転送なし・ローテーション要求なしから開始する
Init ==
    \* KEK は初期状態で active（稼働中）に設定する
    /\ kek_state = "active"
    \* raw bytes は初期状態で in-flight ではない（転送中でない）
    /\ raw_bytes_in_flight = FALSE
    \* ローテーション要求は初期状態でなし
    /\ rotation_requested = FALSE

\* アクション: ローテーション要求を発生させる（運用者による KEK ローテーション指示）
RequestRotation ==
    \* 現在 active 状態の場合のみローテーション要求を発生させる
    /\ kek_state = "active"
    \* 既にローテーション要求がない場合のみ新規要求を発生させる
    /\ rotation_requested = FALSE
    \* ローテーション要求フラグを TRUE に変更する
    /\ rotation_requested' = TRUE
    \* kek_state は変化しない
    /\ UNCHANGED kek_state
    \* raw_bytes_in_flight は変化しない
    /\ UNCHANGED raw_bytes_in_flight

\* アクション: raw bytes の転送開始（暗号化前データを active KEK で暗号化する前に送信）
StartRawBytesTransfer ==
    \* active 状態の KEK のみ raw bytes 転送を開始できる
    /\ kek_state = "active"
    \* ローテーション要求中は raw bytes 転送を禁止する（安全性のため）
    /\ rotation_requested = FALSE
    \* raw_bytes_in_flight フラグを TRUE に変更する
    /\ raw_bytes_in_flight' = TRUE
    \* kek_state は変化しない
    /\ UNCHANGED kek_state
    \* rotation_requested は変化しない
    /\ UNCHANGED rotation_requested

\* アクション: raw bytes の転送完了（暗号化完了後に in-flight フラグをクリアする）
FinishRawBytesTransfer ==
    \* raw bytes が in-flight である場合のみ転送完了アクションを実行する
    /\ raw_bytes_in_flight = TRUE
    \* raw_bytes_in_flight フラグをクリアする（転送完了を示す）
    /\ raw_bytes_in_flight' = FALSE
    \* kek_state は変化しない
    /\ UNCHANGED kek_state
    \* rotation_requested は変化しない
    /\ UNCHANGED rotation_requested

\* アクション: ローテーション実行（active → rotating 遷移）
ExecuteRotation ==
    \* active 状態でかつローテーション要求がある場合のみ実行する
    /\ kek_state = "active"
    \* ローテーション要求フラグが立っていることを前提とする
    /\ rotation_requested = TRUE
    \* raw bytes が in-flight でないことを確認する（転送中に ローテーション禁止）
    /\ raw_bytes_in_flight = FALSE
    \* kek_state を rotating に遷移させる
    /\ kek_state' = "rotating"
    \* raw_bytes_in_flight は FALSE のまま維持する
    /\ raw_bytes_in_flight' = FALSE
    \* ローテーション要求フラグをクリアする
    /\ rotation_requested' = FALSE

\* アクション: 暗号消去実行（rotating → destroyed 遷移）
CryptoShred ==
    \* rotating 状態の場合のみ暗号消去を実行する
    /\ kek_state = "rotating"
    \* raw bytes が in-flight でないことを確認する（転送中に消去禁止）
    /\ raw_bytes_in_flight = FALSE
    \* kek_state を destroyed に遷移させる（不可逆な消去）
    /\ kek_state' = "destroyed"
    \* raw_bytes_in_flight は変化しない
    /\ UNCHANGED raw_bytes_in_flight
    \* rotation_requested は変化しない
    /\ UNCHANGED rotation_requested

\* 全遷移の定義: 5 つのアクションのいずれかを実行する
Next ==
    \* RequestRotation アクションを選択する
    \/ RequestRotation
    \* StartRawBytesTransfer アクションを選択する
    \/ StartRawBytesTransfer
    \* FinishRawBytesTransfer アクションを選択する
    \/ FinishRawBytesTransfer
    \* ExecuteRotation アクションを選択する
    \/ ExecuteRotation
    \* CryptoShred アクションを選択する
    \/ CryptoShred

\* safety property: raw bytes が in-flight の間に destroyed 状態にはならない
\* 05_鍵管理適合仕様の no_raw_bytes_in_flight を保証する
NoRawBytesInFlight ==
    \* raw_bytes_in_flight が TRUE かつ kek_state が "destroyed" の組合せは存在しない
    ~(raw_bytes_in_flight = TRUE /\ kek_state = "destroyed")

\* spec 定義: 初期状態 + 次状態遷移の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_<<kek_state, raw_bytes_in_flight, rotation_requested>>

\* 定理: Spec が成立すれば NoRawBytesInFlight が常に成立する
THEOREM Spec => []NoRawBytesInFlight
====
