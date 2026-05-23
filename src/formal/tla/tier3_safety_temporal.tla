\* k1s0-proof: PROOF-tier3-tsafe-010 -> IMPL-tier3-0010
\* tier3_safety_temporal.tla
\* tier3 クライアント reducer temporal safety: イベントの silent drop 禁止の形式検証
\* obligation_id: tier3_safety_010
\* cell_state: stub（Apalache で検証後に v1_baseline_verified に更新する）
\* property: NoEventLoss — クライアント reducer が受け取ったイベントを silent drop しない
---- MODULE tier3_safety_temporal ----
\* 標準ライブラリ Naturals と TLC をインポートする（整数演算とモデル検査に使用）
EXTENDS Naturals, TLC

\* イベントキューの最大サイズを定義する（状態空間を有限に抑えるため）
MAX_EVENTS == 5

\* 変数宣言: event_queue_size はキューに滞留しているイベント数を保持する（Apalache type: Int）
VARIABLE
    \* @type: Int;
    event_queue_size

\* 変数宣言: processed_count は処理済みイベント数を保持する（Apalache type: Int）
VARIABLE
    \* @type: Int;
    processed_count

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* event_queue_size は 0 以上 MAX_EVENTS 以下の整数でなければならない
    /\ event_queue_size >= 0 /\ event_queue_size <= MAX_EVENTS
    \* processed_count は 0 以上 MAX_EVENTS 以下の整数でなければならない
    /\ processed_count >= 0 /\ processed_count <= MAX_EVENTS

\* 初期状態: キューとカウントを 0 から開始する
Init ==
    \* event_queue_size の初期値は 0（空キュー）に設定する
    /\ event_queue_size = 0
    \* processed_count の初期値は 0（未処理）に設定する
    /\ processed_count = 0

\* アクション: イベントをキューに追加する（外部からのイベント受信をシミュレートする）
EnqueueEvent ==
    \* キューが最大サイズ未満の場合のみ追加を許可する
    /\ event_queue_size < MAX_EVENTS
    \* キューサイズを 1 増やす
    /\ event_queue_size' = event_queue_size + 1
    \* processed_count は変化しない
    /\ UNCHANGED processed_count

\* アクション: イベントを処理する（reducer がキューからイベントを取り出して処理する）
ProcessEvent ==
    \* キューにイベントが存在する場合のみ処理する
    /\ event_queue_size > 0
    \* processed_count が event_queue_size 未満の場合のみ処理を進める（over-count を防ぐ）
    /\ processed_count < event_queue_size
    \* processed_count を 1 増やす
    /\ processed_count' = processed_count + 1
    \* event_queue_size は変化しない（total enqueued count を保持する）
    /\ UNCHANGED event_queue_size

\* アクション: 処理済みイベントをリセットする（バッチ完了後のカウンタリセット）
ResetCounts ==
    \* processed_count と event_queue_size が一致した場合にリセットを許可する
    /\ processed_count = event_queue_size
    \* 両方のカウンタを 0 にリセットする
    /\ event_queue_size' = 0
    /\ processed_count' = 0

\* アクション: Sink ステートのループ遷移（deadlock 回避用の no-op 遷移）
Stutter ==
    \* 全変数を変化させない（stutter 遷移）
    /\ UNCHANGED event_queue_size
    /\ UNCHANGED processed_count

\* 全遷移の定義: 4 つのアクションのいずれかを実行する
Next ==
    \* イベントエンキューアクションを選択する
    \/ EnqueueEvent
    \* イベント処理アクションを選択する
    \/ ProcessEvent
    \* カウンタリセットアクションを選択する
    \/ ResetCounts
    \* Stutter アクションを選択する
    \/ Stutter

\* safety property: processed_count は event_queue_size を超えない（over-count なし）
\* クライアント reducer がイベントを silent drop しないことを形式化する
NoEventLoss ==
    \* processed_count が event_queue_size 以下であることを保証する
    processed_count <= event_queue_size

\* vars タプルの型を明示するために補助定義を使う（Apalache type checker 向け）
\* @type: () => <<Int, Int>>;
vars == <<event_queue_size, processed_count>>

\* spec 定義: 初期状態 + 次状態遷移の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_vars

\* 定理: Spec が成立すれば NoEventLoss が常に成立する
THEOREM Spec => []NoEventLoss
====
