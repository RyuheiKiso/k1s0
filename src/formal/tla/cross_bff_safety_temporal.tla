\* k1s0-proof: PROOF-cross_bff-tsafe-074 -> IMPL-cross_bff-0001
\* cross_bff_safety_temporal.tla
\* cross_bff クロスカット関心事 — 未認証リクエスト backend 到達禁止 temporal safety
\* obligation_id: cross_bff_safety_074
\* cell_state: v1_accepted_with_assumption（Apalache 検証後に v1_baseline_verified に更新する）
\* property: AuthGatekeeper — 認証されていないリクエストは backend に到達しない
---- MODULE cross_bff_safety_temporal ----
\* 標準ライブラリ Naturals と TLC をインポートする（整数演算とモデル検査に使用）
EXTENDS Naturals, TLC

\* 変数宣言: request_state は現在のリクエスト処理状態を保持する（Apalache type annotation: Str）
VARIABLE
    \* @type: Str;
    request_state

\* 変数宣言: authenticated はリクエストの認証完了フラグを保持する（Apalache type annotation: Bool）
VARIABLE
    \* @type: Bool;
    authenticated

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* request_state は規定された 4 値の文字列でなければならない
    /\ request_state \in {"received", "authenticated", "rejected", "forwarded"}
    \* authenticated は真偽値でなければならない
    /\ authenticated \in BOOLEAN

\* 初期状態: リクエスト受信直後・未認証状態から開始する
Init ==
    \* request_state の初期値は "received"（受信状態）に設定する
    /\ request_state = "received"
    \* authenticated の初期値は FALSE（未認証）に設定する
    /\ authenticated = FALSE

\* アクション: リクエストを認証する（認証成功パス）
AuthenticateRequest ==
    \* received 状態のリクエストのみ認証できる
    /\ request_state = "received"
    \* 認証状態に遷移する
    /\ request_state' = "authenticated"
    \* 認証フラグを TRUE に設定する
    /\ authenticated' = TRUE

\* アクション: 認証失敗によりリクエストを拒否する（認証失敗パス）
RejectRequest ==
    \* received 状態のリクエストのみ拒否できる
    /\ request_state = "received"
    \* 拒否状態に遷移する
    /\ request_state' = "rejected"
    \* 認証フラグは FALSE のまま維持する
    /\ UNCHANGED authenticated

\* アクション: 認証済みリクエストを backend に転送する（転送パス）
ForwardToBackend ==
    \* 認証済みリクエストのみ転送できる（AuthGatekeeper の核心）
    /\ request_state = "authenticated"
    \* authenticated フラグが TRUE であることを前提とする
    /\ authenticated = TRUE
    \* 転送状態に遷移する
    /\ request_state' = "forwarded"
    \* authenticated フラグは変化しない
    /\ UNCHANGED authenticated

\* 全遷移の定義: 各アクションのいずれかを選択する
Next ==
    \* 認証アクションを選択する
    \/ AuthenticateRequest
    \* 拒否アクションを選択する
    \/ RejectRequest
    \* 転送アクションを選択する
    \/ ForwardToBackend

\* safety invariant: forwarded 状態のリクエストは必ず認証済みである
\* cross_bff 適合仕様の AuthGatekeeper 規定の形式化
AuthGatekeeper ==
    \* request_state = "forwarded" ならば authenticated = TRUE が成立することを主張する
    request_state = "forwarded" => authenticated = TRUE

\* spec 定義: 初期状態 + 次状態遷移の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_<<request_state, authenticated>>

\* 定理: Spec が成立すれば AuthGatekeeper が常に成立する
THEOREM Spec => []AuthGatekeeper
====
