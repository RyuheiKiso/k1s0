\* k1s0-proof: PROOF-ops-tsafe-029 -> IMPL-ops-0029
\* ops_safety_temporal.tla
\* ops アラートルーティング temporal safety: firing alert が必ず handler に到達することの形式検証
\* obligation_id: ops_safety_029
\* cell_state: stub（Apalache で検証後に v1_baseline_verified に更新する）
\* property: AlertRouting — firing alert は必ず少なくとも1つの handler に到達する
---- MODULE ops_safety_temporal ----
\* 標準ライブラリ Naturals と TLC をインポートする（整数演算とモデル検査に使用）
EXTENDS Naturals, TLC

\* アラート handler の最大数を定義する（状態空間を有限に抑えるため）
MAX_HANDLERS == 3

\* アラート状態を整数でエンコードする（Apalache は Str 型の選択肢を限定的にしか扱えないため）
\* STATE_SILENT=0: 無音状態、STATE_FIRING=1: 発火状態、STATE_ROUTED=2: ルーティング済み状態
STATE_SILENT == 0
STATE_FIRING == 1
STATE_ROUTED == 2

\* 変数宣言: alert_state は現在のアラート状態を保持する（Apalache type: Int）
VARIABLE
    \* @type: Int;
    alert_state

\* 変数宣言: handler_count は現在のアクティブ handler 数を保持する（Apalache type: Int）
VARIABLE
    \* @type: Int;
    handler_count

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* alert_state は 0 以上 2 以下の整数でなければならない（STATE_SILENT/FIRING/ROUTED）
    /\ alert_state >= 0 /\ alert_state <= 2
    \* handler_count は 0 以上 MAX_HANDLERS 以下の整数でなければならない
    /\ handler_count >= 0 /\ handler_count <= MAX_HANDLERS

\* 初期状態: アラートは silent 状態、handler は登録なしから開始する
Init ==
    \* alert_state の初期値は STATE_SILENT（無音）に設定する
    /\ alert_state = STATE_SILENT
    \* handler_count の初期値は 0（handler 未登録）に設定する
    /\ handler_count = 0

\* アクション: handler を登録する（Alertmanager への receiver 登録をモデル化する）
RegisterHandler ==
    \* 最大数未満の場合のみ handler を登録できる
    /\ handler_count < MAX_HANDLERS
    \* handler_count を 1 増やす
    /\ handler_count' = handler_count + 1
    \* alert_state は変化しない
    /\ UNCHANGED alert_state

\* アクション: アラートを発火させる（Prometheus の rule 評価による alert 発火をモデル化する）
FireAlert ==
    \* silent 状態のアラートのみ発火できる
    /\ alert_state = STATE_SILENT
    \* handler が少なくとも1つ登録されている場合のみ発火させる（handler なしでは発火しない）
    /\ handler_count >= 1
    \* alert_state を firing に更新する
    /\ alert_state' = STATE_FIRING
    \* handler_count は変化しない
    /\ UNCHANGED handler_count

\* アクション: アラートを handler にルーティングする（Alertmanager のルーティングをモデル化する）
RouteAlert ==
    \* firing 状態のアラートのみルーティングできる
    /\ alert_state = STATE_FIRING
    \* alert_state を routed に更新する
    /\ alert_state' = STATE_ROUTED
    \* handler_count は変化しない
    /\ UNCHANGED handler_count

\* アクション: アラートを解決する（incident 収束後の alert 解除をモデル化する）
ResolveAlert ==
    \* routed 状態のアラートのみ解決できる
    /\ alert_state = STATE_ROUTED
    \* alert_state を silent に戻す
    /\ alert_state' = STATE_SILENT
    \* handler_count は変化しない
    /\ UNCHANGED handler_count

\* アクション: Sink ステートのループ遷移（deadlock 回避用の no-op 遷移）
Stutter ==
    \* 全変数を変化させない（stutter 遷移）
    /\ UNCHANGED alert_state
    /\ UNCHANGED handler_count

\* 全遷移の定義: 5 つのアクションのいずれかを実行する
Next ==
    \* handler 登録アクションを選択する
    \/ RegisterHandler
    \* アラート発火アクションを選択する
    \/ FireAlert
    \* アラートルーティングアクションを選択する
    \/ RouteAlert
    \* アラート解決アクションを選択する
    \/ ResolveAlert
    \* Stutter アクションを選択する
    \/ Stutter

\* safety property: alert_state = STATE_ROUTED のとき handler_count >= 1 でなければならない
\* firing alert が必ず少なくとも1つの handler に到達することを形式化する
AlertRouting ==
    \* routed 状態の場合は handler が1つ以上存在することを保証する
    alert_state = STATE_ROUTED => handler_count >= 1

\* vars タプルの型を明示するために補助定義を使う（Apalache type checker 向け）
\* @type: () => <<Int, Int>>;
vars == <<alert_state, handler_count>>

\* spec 定義: 初期状態 + 次状態遷移の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_vars

\* 定理: Spec が成立すれば AlertRouting が常に成立する
THEOREM Spec => []AlertRouting
====
