\* k1s0-proof: PROOF-tier1-tsafe-008 -> IMPL-tier1-0002
\* tier1_migration_pair_rollback.tla
\* tier1 移行ペア ロールバック可能性: あらゆる migration は safe な rollback が存在する temporal safety
\* obligation_id: tier1_migration_pair_tsp
\* property: RollbackAlwaysPossible — migration 適用後に元のスキーマ状態に戻れることを保証する
\* cell_state: stub（Apalache で検証後に v1_baseline_verified に更新する）
---- MODULE tier1_migration_pair_rollback ----
\* migration pair モデル（forward / rollback の対称性）
EXTENDS Naturals, TLC

\* スキーマバージョン集合（N を有限とする）
CONSTANTS N
SchemaVersions == 0..N

\* 現在のスキーマバージョンと migration 適用状態
VARIABLES current_version, migration_applied

TypeInvariant ==
    /\ current_version \in SchemaVersions
    /\ migration_applied \in BOOLEAN

Init ==
    /\ current_version = 0
    /\ migration_applied = FALSE

\* forward migration: v → v+1
ApplyMigration ==
    /\ migration_applied = FALSE
    /\ current_version < N
    /\ current_version' = current_version + 1
    /\ migration_applied' = TRUE

\* rollback migration: v+1 → v（常に可能）
RollbackMigration ==
    /\ migration_applied = TRUE
    /\ current_version > 0
    /\ current_version' = current_version - 1
    /\ migration_applied' = FALSE

Next == ApplyMigration \/ RollbackMigration

\* safety: forward migration が適用された後に必ず rollback が可能であること
RollbackAlwaysPossible ==
    migration_applied => ENABLED RollbackMigration

Spec == Init /\ [][Next]_<<current_version, migration_applied>>

THEOREM Spec => []RollbackAlwaysPossible
====
