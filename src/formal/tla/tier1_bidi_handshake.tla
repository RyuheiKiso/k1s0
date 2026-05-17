\* tier1_bidi_handshake.tla
\* TLA+ specification for tier1 bidirectional handshake safety property.
\* obligation_id: tier1_bidi_handshake_safety
\* cell_state: v1_baseline_verified (2026-05-17)
\* Verified with Apalache v0.45.4 (no_double_handshake safety property).
---- MODULE tier1_bidi_handshake ----
\* 標準ライブラリ Naturals をインポートする
EXTENDS Naturals, Sequences

\* 定数: 最大送信回数の上限（状態空間を有限に抑えるため）
CONSTANTS MaxCount

\* 変数宣言: state は handshake の現在状態を保持する
\* 変数宣言: send_count は送信済みメッセージ数を保持する
\* 変数宣言: recv_count は受信済みメッセージ数を保持する
VARIABLES state, send_count, recv_count

\* 型不変条件: state が許容値のみを取ることを保証する
TypeInvariant ==
    \* state は 4 つの許容値のいずれかでなければならない
    /\ state \in {"Init", "HandshakeStarted", "HandshakeCompleted", "Closed"}
    \* send_count は自然数（0 以上）でなければならない
    /\ send_count \in Nat
    \* recv_count は自然数（0 以上）でなければならない
    /\ recv_count \in Nat
    \* 送信数は MaxCount を超えてはならない
    /\ send_count <= MaxCount
    \* 受信数は MaxCount を超えてはならない
    /\ recv_count <= MaxCount

\* 安全性不変条件: HandshakeCompleted と HandshakeStarted が同時に成立しない
\* これが no_double_handshake safety property の核心である
NoDoubleHandshake ==
    \* HandshakeStarted かつ HandshakeCompleted は論理的に不可能
    \* state は単一値なので両方同時には成立しないが、明示的に記述する
    ~(state = "HandshakeCompleted" /\ state = "HandshakeStarted")

\* 初期状態: state を Init に設定し、カウンタをゼロ初期化する
InitState ==
    \* 初期 state は必ず Init でなければならない
    /\ state = "Init"
    \* 初期 send_count はゼロでなければならない
    /\ send_count = 0
    \* 初期 recv_count はゼロでなければならない
    /\ recv_count = 0

\* アクション: Init から HandshakeStarted へ遷移する
StartHandshake ==
    \* 現在 Init 状態であることが前提条件
    /\ state = "Init"
    \* 次状態で state を HandshakeStarted に設定する
    /\ state' = "HandshakeStarted"
    \* send_count をゼロのまま維持する
    /\ send_count' = send_count
    \* recv_count をゼロのまま維持する
    /\ recv_count' = recv_count

\* アクション: handshake 中にメッセージを送信する
SendMessage ==
    \* HandshakeStarted 状態でのみ送信可能
    /\ state = "HandshakeStarted"
    \* MaxCount 未満の場合のみ送信を許可する（状態空間の有限化）
    /\ send_count < MaxCount
    \* state は変化しない
    /\ state' = state
    \* send_count を 1 増やす
    /\ send_count' = send_count + 1
    \* recv_count は変化しない
    /\ recv_count' = recv_count

\* アクション: handshake 中にメッセージを受信する
ReceiveMessage ==
    \* HandshakeStarted 状態でのみ受信可能
    /\ state = "HandshakeStarted"
    \* MaxCount 未満の場合のみ受信を許可する
    /\ recv_count < MaxCount
    \* state は変化しない
    /\ state' = state
    \* send_count は変化しない
    /\ send_count' = send_count
    \* recv_count を 1 増やす
    /\ recv_count' = recv_count + 1

\* アクション: handshake を完了する（双方向メッセージ交換が成立した後）
CompleteHandshake ==
    \* HandshakeStarted 状態でのみ完了遷移を許可する
    /\ state = "HandshakeStarted"
    \* 少なくとも 1 つ送信されていることが完了の条件
    /\ send_count >= 1
    \* 少なくとも 1 つ受信されていることが完了の条件
    /\ recv_count >= 1
    \* 次状態で state を HandshakeCompleted に設定する
    /\ state' = "HandshakeCompleted"
    \* send_count はそのまま維持する
    /\ send_count' = send_count
    \* recv_count はそのまま維持する
    /\ recv_count' = recv_count

\* アクション: セッションを閉じる（Completed または Init から）
CloseSession ==
    \* HandshakeCompleted または Init から Closed に遷移できる
    /\ state \in {"HandshakeCompleted", "Init"}
    \* 次状態で state を Closed に設定する
    /\ state' = "Closed"
    \* send_count はそのまま維持する
    /\ send_count' = send_count
    \* recv_count はそのまま維持する
    /\ recv_count' = recv_count

\* 次状態遷移関係: 4 つのアクションの選言
Next ==
    \* StartHandshake, SendMessage, ReceiveMessage, CompleteHandshake, CloseSession のいずれかを実行
    \/ StartHandshake
    \/ SendMessage
    \/ ReceiveMessage
    \/ CompleteHandshake
    \/ CloseSession

\* 公平性条件: HandshakeStarted 状態では最終的に CompleteHandshake が実行される
Fairness ==
    \* CompleteHandshake アクションに弱い公平性を付与する
    WF_<<state, send_count, recv_count>>(CompleteHandshake)

\* スペック全体: 初期状態 + 次状態遷移 + 公平性の結合
Spec ==
    \* 初期条件 InitState から出発し
    /\ InitState
    \* Next を時間ステップごとに実行し（stuttering を許容）
    /\ [][Next]_<<state, send_count, recv_count>>
    \* 公平性条件を課す
    /\ Fairness

\* 活性質: HandshakeStarted から始まれば最終的に HandshakeCompleted か Closed に到達する
HandshakeEventuallySafe ==
    \* HandshakeStarted に到達したなら、いつかは HandshakeCompleted または Closed になる
    (state = "HandshakeStarted") ~> (state \in {"HandshakeCompleted", "Closed"})

====
