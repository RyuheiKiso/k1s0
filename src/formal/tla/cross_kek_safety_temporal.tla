\* k1s0-proof: PROOF-cross_kek-tsafe-054 -> IMPL-cross_kek-0001
\* cross_kek_safety_temporal.tla
\* cross_kek クロスカット関心事 — KEK raw bytes ceremony window 外メモリ排除 temporal safety
\* obligation_id: cross_kek_safety_054
\* cell_state: v1_accepted_with_assumption（Apalache 検証後に v1_baseline_verified に更新する）
\* property: KEKExclusion — KEK raw bytes は ceremony window 外でメモリに存在しない
---- MODULE cross_kek_safety_temporal ----
\* 標準ライブラリ Naturals と TLC をインポートする（整数演算とモデル検査に使用）
EXTENDS Naturals, TLC

\* 変数宣言: ceremony_state は現在の KEK ceremony 状態を保持する（Apalache type annotation: Str）
VARIABLE
    \* @type: Str;
    ceremony_state

\* 変数宣言: kek_in_memory は KEK raw bytes のメモリ存在フラグを保持する（Apalache type annotation: Bool）
VARIABLE
    \* @type: Bool;
    kek_in_memory

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* ceremony_state は規定された 3 値の文字列でなければならない
    /\ ceremony_state \in {"idle", "active", "closed"}
    \* kek_in_memory は真偽値でなければならない
    /\ kek_in_memory \in BOOLEAN

\* 初期状態: ceremony 未開始・KEK メモリ不在から開始する
Init ==
    \* ceremony_state の初期値は "idle"（待機状態）に設定する
    /\ ceremony_state = "idle"
    \* kek_in_memory の初期値は FALSE（メモリ不在）に設定する
    /\ kek_in_memory = FALSE

\* アクション: ceremony を開始する（KEK raw bytes をメモリにロードする）
StartCeremony ==
    \* 待機状態の場合のみ ceremony を開始できる
    /\ ceremony_state = "idle"
    \* ceremony 状態を active に遷移させる
    /\ ceremony_state' = "active"
    \* ceremony 開始時に KEK raw bytes をメモリにロードする
    /\ kek_in_memory' = TRUE

\* アクション: ceremony を完了する（KEK raw bytes をメモリから消去する）
CloseCeremony ==
    \* active 状態の場合のみ ceremony を完了できる
    /\ ceremony_state = "active"
    \* ceremony 状態を closed に遷移させる
    /\ ceremony_state' = "closed"
    \* ceremony 完了時に KEK raw bytes をメモリから消去する（KEKExclusion 規約）
    /\ kek_in_memory' = FALSE

\* アクション: ceremony を idle にリセットする（closed からのリセット）
ResetCeremony ==
    \* closed 状態の場合のみリセットできる
    /\ ceremony_state = "closed"
    \* ceremony 状態を idle に戻す
    /\ ceremony_state' = "idle"
    \* KEK はメモリに存在しない状態を維持する
    /\ kek_in_memory' = FALSE

\* 全遷移の定義: 各アクションのいずれかを選択する
Next ==
    \* ceremony 開始アクションを選択する
    \/ StartCeremony
    \* ceremony 完了アクションを選択する
    \/ CloseCeremony
    \* ceremony リセットアクションを選択する
    \/ ResetCeremony

\* safety invariant: ceremony window 外では KEK raw bytes はメモリに存在しない
\* cross_kek 適合仕様の KEKExclusion 規定の形式化
KEKExclusion ==
    \* ceremony_state /= "active" ならば kek_in_memory = FALSE が成立することを主張する
    ceremony_state /= "active" => kek_in_memory = FALSE

\* spec 定義: 初期状態 + 次状態遷移の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_<<ceremony_state, kek_in_memory>>

\* 定理: Spec が成立すれば KEKExclusion が常に成立する
THEOREM Spec => []KEKExclusion
====
