\* k1s0-proof: PROOF-tier1-tsafe-009 -> IMPL-tier1-0006
\* tier1_schema_evolution_compatibility.tla
\* tier1 スキーマ進化 後方互換性: proto スキーマの進化が既存クライアントを破壊しないことを保証
\* obligation_id: tier1_schema_evolution_tsp
\* property: BackwardCompatibility — フィールド追加は常に後方互換。フィールド削除は禁止。
\* cell_state: stub（Apalache で検証後に v1_baseline_verified に更新する）
---- MODULE tier1_schema_evolution_compatibility ----
EXTENDS Naturals, FiniteSets, TLC

\* スキーマはフィールド ID の集合として表現する（proto field numbers）
CONSTANTS MaxFieldId
FieldIds == 1..MaxFieldId

\* 現世代スキーマと次世代スキーマ
VARIABLES current_schema, next_schema

TypeInvariant ==
    /\ current_schema \subseteq FieldIds
    /\ next_schema \subseteq FieldIds

Init ==
    /\ current_schema = {1, 2, 3}   \* 初期スキーマ: field 1-3
    /\ next_schema = current_schema

\* フィールド追加（後方互換: 既存フィールドは保持）
AddField(fid) ==
    /\ fid \notin current_schema
    /\ next_schema' = current_schema \union {fid}
    /\ current_schema' = current_schema

\* フィールド削除は禁止（次世代スキーマが現世代の部分集合になる変更を禁止）
RemoveField(fid) ==
    /\ fid \in current_schema
    /\ next_schema' = current_schema \ {fid}  \* これは起きてはならない
    /\ current_schema' = current_schema

\* スキーマを更新する（後方互換チェック後）
CommitSchema ==
    /\ current_schema \subseteq next_schema   \* 既存フィールドが保持されていることを確認
    /\ current_schema' = next_schema
    /\ next_schema' = next_schema

Next == \/ \E fid \in (FieldIds \ current_schema) : AddField(fid)
        \/ CommitSchema

\* safety: 現世代スキーマのフィールドは次世代でも必ず保持される（後方互換性）
BackwardCompatibility ==
    current_schema \subseteq next_schema

Spec == Init /\ [][Next]_<<current_schema, next_schema>>

THEOREM Spec => []BackwardCompatibility
====
