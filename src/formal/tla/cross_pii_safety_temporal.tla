\* k1s0-proof: PROOF-cross_pii-tsafe-079 -> IMPL-cross_pii-0001
\* cross_pii_safety_temporal.tla
\* cross_pii クロスカット関心事 — PII データ non-PII audit log 書き込み禁止 temporal safety
\* obligation_id: cross_pii_safety_079
\* cell_state: v1_accepted_with_assumption（Apalache 検証後に v1_baseline_verified に更新する）
\* property: PIIIsolation — PII データは non-PII audit log には書き込まれない
---- MODULE cross_pii_safety_temporal ----
\* 標準ライブラリ Naturals と TLC をインポートする（整数演算とモデル検査に使用）
EXTENDS Naturals, TLC

\* 変数宣言: data_class は処理中データの PII 分類を保持する（Apalache type annotation: Str）
VARIABLE
    \* @type: Str;
    data_class

\* 変数宣言: log_target はデータの書き込み先ログを保持する（Apalache type annotation: Str）
VARIABLE
    \* @type: Str;
    log_target

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* data_class は規定された 2 値の文字列でなければならない
    /\ data_class \in {"pii", "non_pii"}
    \* log_target は規定された 3 値の文字列でなければならない
    /\ log_target \in {"pii_log", "audit_log", "none"}

\* 初期状態: データ分類なし・ログ書き込み先なしから開始する
Init ==
    \* data_class の初期値は "non_pii"（非 PII データ）に設定する
    /\ data_class = "non_pii"
    \* log_target の初期値は "none"（未決定）に設定する
    /\ log_target = "none"

\* アクション: PII データを PII ログに書き込む（許可されたパス）
WritePIIToPIILog ==
    \* PII データの場合のみ PII ログへの書き込みを許可する
    /\ data_class = "pii"
    \* PII ログに書き込む（PIIIsolation 規約に準拠する）
    /\ log_target' = "pii_log"
    \* data_class は変化しない
    /\ UNCHANGED data_class

\* アクション: 非 PII データを audit log に書き込む（許可されたパス）
WriteNonPIIToAuditLog ==
    \* 非 PII データの場合のみ audit log への書き込みを許可する
    /\ data_class = "non_pii"
    \* audit log に書き込む（非 PII データは audit log への書き込みを許可する）
    /\ log_target' = "audit_log"
    \* data_class は変化しない
    /\ UNCHANGED data_class

\* アクション: データ分類を PII に変更する
ClassifyAsPII ==
    \* 非 PII データを PII に再分類する
    /\ data_class = "non_pii"
    \* PII 分類に変更する
    /\ data_class' = "pii"
    \* ログ書き込み先をリセットする（再分類後は再度書き込み先を選択する）
    /\ log_target' = "none"

\* アクション: データ分類を非 PII に変更する
ClassifyAsNonPII ==
    \* PII データを非 PII に再分類する
    /\ data_class = "pii"
    \* 非 PII 分類に変更する
    /\ data_class' = "non_pii"
    \* ログ書き込み先をリセットする（再分類後は再度書き込み先を選択する）
    /\ log_target' = "none"

\* アクション: ログ書き込み先をリセットする
ResetLogTarget ==
    \* log_target を none にリセットする
    /\ log_target' = "none"
    \* data_class は変化しない
    /\ UNCHANGED data_class

\* 全遷移の定義: 各アクションのいずれかを選択する
Next ==
    \* PII データの PII ログ書き込みアクションを選択する
    \/ WritePIIToPIILog
    \* 非 PII データの audit log 書き込みアクションを選択する
    \/ WriteNonPIIToAuditLog
    \* PII 分類アクションを選択する
    \/ ClassifyAsPII
    \* 非 PII 分類アクションを選択する
    \/ ClassifyAsNonPII
    \* ログ書き込み先リセットアクションを選択する
    \/ ResetLogTarget

\* safety invariant: PII データは audit_log に書き込まれない
\* cross_pii 適合仕様の PIIIsolation 規定の形式化
PIIIsolation ==
    \* data_class = "pii" ならば log_target /= "audit_log" が成立することを主張する
    data_class = "pii" => log_target /= "audit_log"

\* 変数タプル演算子: Apalache 型推論のための明示的 tuple 宣言（同型変数の曖昧性回避）
\* @type: <<Str, Str>>;
vars == <<data_class, log_target>>

\* spec 定義: 初期状態 + 次状態遷移の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_vars

\* 定理: Spec が成立すれば PIIIsolation が常に成立する
THEOREM Spec => []PIIIsolation
====
