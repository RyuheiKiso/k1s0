\* k1s0-proof: PROOF-test-tsafe-039 -> IMPL-test-0039
\* test_safety_temporal.tla
\* test regression corpus monotone temporal safety: corpus エントリ数の減少禁止の形式検証
\* obligation_id: test_safety_039
\* cell_state: stub（Apalache で検証後に v1_baseline_verified に更新する）
\* property: RegressionCorpusMonotone — regression corpus のエントリ数は減少しない
---- MODULE test_safety_temporal ----
\* 標準ライブラリ Naturals と TLC をインポートする（整数演算とモデル検査に使用）
EXTENDS Naturals, TLC

\* corpus の最大エントリ数を定義する（状態空間を有限に抑えるため）
MAX_CORPUS == 10

\* 変数宣言: corpus_size は現在の regression corpus エントリ数を保持する（Apalache type: Int）
VARIABLE
    \* @type: Int;
    corpus_size

\* 変数宣言: prev_corpus_size は前のステップの corpus_size を保持する（monotone 検証用）
VARIABLE
    \* @type: Int;
    prev_corpus_size

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* corpus_size は 0 以上 MAX_CORPUS 以下の整数でなければならない
    /\ corpus_size >= 0 /\ corpus_size <= MAX_CORPUS
    \* prev_corpus_size は 0 以上 MAX_CORPUS 以下の整数でなければならない
    /\ prev_corpus_size >= 0 /\ prev_corpus_size <= MAX_CORPUS

\* 初期状態: corpus は空から開始する
Init ==
    \* corpus_size の初期値は 0（空の corpus）に設定する
    /\ corpus_size = 0
    \* prev_corpus_size の初期値は 0（変化点追跡の初期値）に設定する
    /\ prev_corpus_size = 0

\* アクション: regression corpus にテストケースを追加する（新しい失敗ケースの登録をモデル化する）
AddCorpusEntry ==
    \* 最大数未満の場合のみ追加を許可する
    /\ corpus_size < MAX_CORPUS
    \* prev_corpus_size に現在の corpus_size を記録する（monotone 追跡のため）
    /\ prev_corpus_size' = corpus_size
    \* corpus_size を 1 増やす（monotone 増加のみ）
    /\ corpus_size' = corpus_size + 1

\* アクション: Sink ステートのループ遷移（deadlock 回避用・削除アクションは定義しない）
\* corpus からのエントリ削除はモデル化しない（property: 削除アクションは不可）
Stutter ==
    \* 全変数を変化させない（stutter 遷移）
    /\ UNCHANGED corpus_size
    /\ UNCHANGED prev_corpus_size

\* 全遷移の定義: 2 つのアクションのいずれかを実行する
Next ==
    \* corpus エントリ追加アクションを選択する
    \/ AddCorpusEntry
    \* Stutter アクションを選択する
    \/ Stutter

\* safety property: corpus_size は prev_corpus_size 以上でなければならない（monotone non-decreasing）
\* regression corpus のエントリ数が減少しないことを形式化する
RegressionCorpusMonotone ==
    \* corpus_size が prev_corpus_size 以上であることを保証する
    corpus_size >= prev_corpus_size

\* vars タプルの型を明示するために補助定義を使う（Apalache type checker 向け）
\* @type: () => <<Int, Int>>;
vars == <<corpus_size, prev_corpus_size>>

\* spec 定義: 初期状態 + 次状態遷移の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_vars

\* 定理: Spec が成立すれば RegressionCorpusMonotone が常に成立する
THEOREM Spec => []RegressionCorpusMonotone
====
