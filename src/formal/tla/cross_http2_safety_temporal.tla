\* k1s0-proof: PROOF-cross_http2-tsafe-049 -> IMPL-cross_http2-0001
\* cross_http2_safety_temporal.tla
\* cross_http2 クロスカット関心事 — HTTP/2 強制 temporal safety
\* obligation_id: cross_http2_safety_049
\* cell_state: v1_accepted_with_assumption（Apalache 検証後に v1_baseline_verified に更新する）
\* property: HTTP2RequiresTLS — HTTP/2 プロトコルが選択された場合 TLS は常に有効でなければならない
---- MODULE cross_http2_safety_temporal ----
\* 標準ライブラリ Naturals と TLC をインポートする（整数演算とモデル検査に使用）
EXTENDS Naturals, TLC

\* 変数宣言: protocol は現在のプロトコル種別を保持する（Apalache type annotation: Str）
VARIABLE
    \* @type: Str;
    protocol

\* 変数宣言: tls_enabled は TLS の有効フラグを保持する（Apalache type annotation: Bool）
VARIABLE
    \* @type: Bool;
    tls_enabled

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* protocol は規定された 3 値の文字列でなければならない
    /\ protocol \in {"none", "http1", "http2"}
    \* tls_enabled は真偽値でなければならない
    /\ tls_enabled \in BOOLEAN

\* 初期状態: プロトコル未確定・TLS 無効から開始する
Init ==
    \* protocol の初期値は "none"（未確定状態）に設定する
    /\ protocol = "none"
    \* tls_enabled の初期値は FALSE（無効状態）に設定する
    /\ tls_enabled = FALSE

\* アクション: HTTP/1 プロトコルを選択する（TLS は任意）
SelectHTTP1 ==
    \* HTTP/1 への切り替えを行う
    /\ protocol' = "http1"
    \* TLS 状態はそのまま変化させない
    /\ UNCHANGED tls_enabled

\* アクション: HTTP/2 プロトコルを TLS 付きで選択する（TLS 強制）
SelectHTTP2WithTLS ==
    \* HTTP/2 への切り替えを行う
    /\ protocol' = "http2"
    \* HTTP/2 では TLS を必ず有効にする（cross_http2 強制規約）
    /\ tls_enabled' = TRUE

\* アクション: TLS を有効化する
EnableTLS ==
    \* TLS がまだ無効の場合のみ有効化する
    /\ tls_enabled = FALSE
    \* TLS を有効にする
    /\ tls_enabled' = TRUE
    \* protocol は変化しない
    /\ UNCHANGED protocol

\* アクション: プロトコルを "none" にリセットする（TLS も無効化）
ResetProtocol ==
    \* 現在 HTTP/2 以外の場合にのみリセットを許可する
    /\ protocol /= "http2"
    \* プロトコルを未確定状態に戻す
    /\ protocol' = "none"
    \* TLS を無効化する
    /\ tls_enabled' = FALSE

\* 全遷移の定義: 各アクションのいずれかを選択する
Next ==
    \* HTTP/1 選択アクションを選択する
    \/ SelectHTTP1
    \* HTTP/2 + TLS 選択アクションを選択する
    \/ SelectHTTP2WithTLS
    \* TLS 有効化アクションを選択する
    \/ EnableTLS
    \* リセットアクションを選択する
    \/ ResetProtocol

\* safety invariant: HTTP/2 が選択されている場合 TLS は必ず有効でなければならない
\* cross_http2 適合仕様の HTTP2RequiresTLS 規定の形式化
HTTP2RequiresTLS ==
    \* protocol = "http2" ならば tls_enabled = TRUE が成立することを主張する
    protocol = "http2" => tls_enabled = TRUE

\* spec 定義: 初期状態 + 次状態遷移の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_<<protocol, tls_enabled>>

\* 定理: Spec が成立すれば HTTP2RequiresTLS が常に成立する
THEOREM Spec => []HTTP2RequiresTLS
====
