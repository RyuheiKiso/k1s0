/**
 * k1s0 tier2 TypeScript ライブラリ エントリポイント
 * tier2 の全モジュールを re-export する
 */

// TenantContext モジュールを re-export する（GUC 注入・テナント識別子管理）
export * from "./tenantContext.js";

// AtomicTripleWrite モジュールを re-export する（P1-P4 atomic 三表書込）
export * from "./atomicTripleWrite.js";

// Outbox モジュールを re-export する（Debezium CDC 経由 Kafka 転送）
export * from "./outbox.js";

// Repository モジュールを re-export する（テナントスコープ内 DB アクセス抽象）
export * from "./repository.js";
