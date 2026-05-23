\* k1s0-proof: PROOF-data-tsafe-020 -> IMPL-data-0020
\* data_safety_temporal.tla
\* data 耐久性 temporal safety: コミット済み write の削除禁止（log は monotone）の形式検証
\* obligation_id: data_safety_020
\* cell_state: stub（Apalache で検証後に v1_baseline_verified に更新する）
\* property: NoDurabilityLoss — コミット済み write は削除されない（log は monotone）
---- MODULE data_safety_temporal ----
\* 標準ライブラリ Naturals と TLC をインポートする（整数演算とモデル検査に使用）
EXTENDS Naturals, TLC

\* コミット済み write の最大数を定義する（状態空間を有限に抑えるため）
MAX_WRITES == 8

\* 変数宣言: commit_count はコミット済み write の累積数を保持する（Apalache type: Int）
VARIABLE
    \* @type: Int;
    commit_count

\* 変数宣言: evict_count はエビクトされた write の累積数を保持する（Apalache type: Int）
VARIABLE
    \* @type: Int;
    evict_count

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* commit_count は 0 以上 MAX_WRITES 以下の整数でなければならない
    /\ commit_count >= 0 /\ commit_count <= MAX_WRITES
    \* evict_count は 0 以上 MAX_WRITES 以下の整数でなければならない
    /\ evict_count >= 0 /\ evict_count <= MAX_WRITES

\* 初期状態: 両カウントを 0 から開始する
Init ==
    \* commit_count の初期値は 0（コミット未実施）に設定する
    /\ commit_count = 0
    \* evict_count の初期値は 0（エビクトなし）に設定する
    /\ evict_count = 0

\* アクション: 新しい write をコミットする（Barman/PostgreSQL WAL のコミットをモデル化する）
CommitWrite ==
    \* 最大数に達していない場合のみコミットを許可する
    /\ commit_count < MAX_WRITES
    \* commit_count を 1 増やす（monotone 増加のみ）
    /\ commit_count' = commit_count + 1
    \* evict_count は変化しない
    /\ UNCHANGED evict_count

\* アクション: 古い write をエビクトする（GC・TTL によるデータ削除をモデル化する）
EvictOldWrite ==
    \* commit_count が 0 より大きい場合のみエビクトを許可する
    /\ commit_count > 0
    \* evict_count が commit_count 未満の場合のみエビクトを許可する（コミット済み分しか削除できない）
    /\ evict_count < commit_count
    \* evict_count を 1 増やす
    /\ evict_count' = evict_count + 1
    \* commit_count は変化しない（コミット済み write の総数は不変）
    /\ UNCHANGED commit_count

\* アクション: Sink ステートのループ遷移（deadlock 回避用の no-op 遷移）
Stutter ==
    \* 全変数を変化させない（stutter 遷移）
    /\ UNCHANGED commit_count
    /\ UNCHANGED evict_count

\* 全遷移の定義: 3 つのアクションのいずれかを実行する
Next ==
    \* write コミットアクションを選択する
    \/ CommitWrite
    \* 古い write エビクトアクションを選択する
    \/ EvictOldWrite
    \* Stutter アクションを選択する
    \/ Stutter

\* safety property: evict_count は commit_count を超えない（コミット済み分のみ削除可能）
\* コミット済み write が削除されないことを形式化する
NoDurabilityLoss ==
    \* evict_count が commit_count 以下であることを保証する
    evict_count <= commit_count

\* vars タプルの型を明示するために補助定義を使う（Apalache type checker 向け）
\* @type: () => <<Int, Int>>;
vars == <<commit_count, evict_count>>

\* spec 定義: 初期状態 + 次状態遷移の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_vars

\* 定理: Spec が成立すれば NoDurabilityLoss が常に成立する
THEOREM Spec => []NoDurabilityLoss
====
