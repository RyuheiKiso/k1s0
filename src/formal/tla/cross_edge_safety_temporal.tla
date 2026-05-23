\* k1s0-proof: PROOF-cross_edge-tsafe-084 -> IMPL-cross_edge-0001
\* cross_edge_safety_temporal.tla
\* cross_edge クロスカット関心事 — クライアントリクエスト available クラスタへの必須ルーティング temporal safety
\* obligation_id: cross_edge_safety_084
\* cell_state: v1_accepted_with_assumption（Apalache 検証後に v1_baseline_verified に更新する）
\* property: EdgeRouting — クライアントリクエストは available なクラスタに必ずルートされる
---- MODULE cross_edge_safety_temporal ----
\* 標準ライブラリ Naturals と TLC をインポートする（整数演算とモデル検査に使用）
EXTENDS Naturals, TLC

\* 変数宣言: request_state は現在のリクエスト処理状態を保持する（Apalache type annotation: Str）
VARIABLE
    \* @type: Str;
    request_state

\* 変数宣言: cluster_available はクラスタの利用可能性フラグを保持する（Apalache type annotation: Bool）
VARIABLE
    \* @type: Bool;
    cluster_available

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* request_state は規定された 3 値の文字列でなければならない
    /\ request_state \in {"pending", "routed", "dropped"}
    \* cluster_available は真偽値でなければならない
    /\ cluster_available \in BOOLEAN

\* 初期状態: リクエスト待機中・クラスタ利用可能な状態から開始する
Init ==
    \* request_state の初期値は "pending"（待機状態）に設定する
    /\ request_state = "pending"
    \* cluster_available の初期値は TRUE（クラスタ利用可能）に設定する
    /\ cluster_available = TRUE

\* アクション: クラスタが利用可能な場合にリクエストをルートする
RouteToCluster ==
    \* pending 状態のリクエストのみルーティング可能である
    /\ request_state = "pending"
    \* クラスタが利用可能な場合のみルーティングを実行する（EdgeRouting の核心）
    /\ cluster_available = TRUE
    \* リクエストをルーティング済み状態に遷移させる
    /\ request_state' = "routed"
    \* cluster_available は変化しない
    /\ UNCHANGED cluster_available

\* アクション: クラスタが利用不可能な場合にリクエストをドロップする
DropRequest ==
    \* pending 状態のリクエストのみドロップ可能である
    /\ request_state = "pending"
    \* クラスタが利用不可能な場合のみドロップを許可する
    /\ cluster_available = FALSE
    \* リクエストをドロップ状態に遷移させる
    /\ request_state' = "dropped"
    \* cluster_available は変化しない
    /\ UNCHANGED cluster_available

\* アクション: クラスタが利用可能になる（クラスタ復旧をシミュレートする）
\* ドロップされたリクエストは再試行のため pending 状態にリセットする（EdgeRouting 不変条件を維持する）
ClusterBecomesAvailable ==
    \* クラスタが現在利用不可能な場合のみ復旧アクションを実行する
    /\ cluster_available = FALSE
    \* クラスタを利用可能状態に変更する
    /\ cluster_available' = TRUE
    \* 以前ドロップされたリクエストを pending に戻してルーティング機会を与える
    /\ IF request_state = "dropped"
       THEN request_state' = "pending"
       ELSE UNCHANGED request_state

\* アクション: クラスタが利用不可能になる（クラスタ障害をシミュレートする）
ClusterBecomesUnavailable ==
    \* クラスタが現在利用可能な場合のみ障害アクションを実行する
    /\ cluster_available = TRUE
    \* クラスタを利用不可能状態に変更する
    /\ cluster_available' = FALSE
    \* request_state は変化しない
    /\ UNCHANGED request_state

\* アクション: 新しいリクエストを受け付ける（リセット）
AcceptNewRequest ==
    \* 既に処理済みのリクエストのみリセットできる
    /\ request_state \in {"routed", "dropped"}
    \* 新しいリクエストを待機状態に設定する
    /\ request_state' = "pending"
    \* cluster_available は変化しない
    /\ UNCHANGED cluster_available

\* 全遷移の定義: 各アクションのいずれかを選択する
Next ==
    \* クラスタへのルーティングアクションを選択する
    \/ RouteToCluster
    \* リクエストドロップアクションを選択する
    \/ DropRequest
    \* クラスタ復旧アクションを選択する
    \/ ClusterBecomesAvailable
    \* クラスタ障害アクションを選択する
    \/ ClusterBecomesUnavailable
    \* 新しいリクエスト受け付けアクションを選択する
    \/ AcceptNewRequest

\* safety invariant: クラスタが利用可能な場合 pending リクエストはドロップされない
\* cross_edge 適合仕様の EdgeRouting 規定の形式化
EdgeRouting ==
    \* cluster_available = TRUE ならば request_state /= "dropped" が成立することを主張する
    cluster_available = TRUE => request_state /= "dropped"

\* spec 定義: 初期状態 + 次状態遷移の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_<<request_state, cluster_available>>

\* 定理: Spec が成立すれば EdgeRouting が常に成立する
THEOREM Spec => []EdgeRouting
====
