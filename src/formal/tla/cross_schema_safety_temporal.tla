\* k1s0-proof: PROOF-cross_schema-tsafe-059 -> IMPL-cross_schema-0001
\* cross_schema_safety_temporal.tla
\* cross_schema クロスカット関心事 — schema registry バージョン番号 monotone 増加 temporal safety
\* obligation_id: cross_schema_safety_059
\* cell_state: v1_accepted_with_assumption（Apalache 検証後に v1_baseline_verified に更新する）
\* property: SchemaBackwardCompat — schema registry のバージョン番号は monotone 増加のみ
---- MODULE cross_schema_safety_temporal ----
\* 標準ライブラリ Naturals と TLC をインポートする（整数演算とモデル検査に使用）
EXTENDS Naturals, TLC

\* 定数: バージョン番号の最大値を定義する（状態空間を有限に抑えるため）
MaxVersion == 10

\* 変数宣言: version は現在のスキーマバージョン番号を保持する（Apalache type annotation: Int）
VARIABLE
    \* @type: Int;
    version

\* 変数宣言: prev_version は前ステップのバージョン番号を保持する（monotone 検証用）
VARIABLE
    \* @type: Int;
    prev_version

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* version は 0 以上 MaxVersion 以下の整数でなければならない
    /\ version >= 0 /\ version <= MaxVersion
    \* prev_version は 0 以上 MaxVersion 以下の整数でなければならない
    /\ prev_version >= 0 /\ prev_version <= MaxVersion

\* 初期状態: バージョン 0 から開始する（両変数とも 0）
Init ==
    \* version の初期値は 0（初期バージョン）に設定する
    /\ version = 0
    \* prev_version の初期値は 0（初期ベースライン）に設定する
    /\ prev_version = 0

\* アクション: スキーマバージョンを 1 インクリメントする（前進のみ許可）
IncrementVersion ==
    \* 最大バージョン未満の場合のみインクリメントを許可する
    /\ version < MaxVersion
    \* prev_version を現在の version に記録する（monotone 検証用）
    /\ prev_version' = version
    \* バージョンを 1 増加させる（減少は禁止）
    /\ version' = version + 1

\* アクション: バージョンを維持する（no-op 遷移）
MaintainVersion ==
    \* バージョンを変化させない
    /\ version' = version
    \* prev_version も変化させない
    /\ prev_version' = prev_version

\* 全遷移の定義: バージョン増加または維持のいずれかを選択する
Next ==
    \* バージョン増加アクションを選択する
    \/ IncrementVersion
    \* バージョン維持アクションを選択する
    \/ MaintainVersion

\* safety invariant: スキーマバージョンは前バージョン以上でなければならない（backward compatibility 保証）
\* cross_schema 適合仕様の SchemaBackwardCompat 規定の形式化
SchemaBackwardCompat ==
    \* version は prev_version 以上でなければならない（減少しない保証）
    version >= prev_version

\* 変数タプル演算子: Apalache 型推論のための明示的 tuple 宣言（同型変数の曖昧性回避）
\* @type: <<Int, Int>>;
vars == <<version, prev_version>>

\* spec 定義: 初期状態 + 次状態遷移の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_vars

\* 定理: Spec が成立すれば SchemaBackwardCompat が常に成立する
THEOREM Spec => []SchemaBackwardCompat
====
