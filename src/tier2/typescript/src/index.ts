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

// Admin モジュールを re-export する（管理境界ガード / 操作種別定義）
export * from "./admin.js";

// CQRS モジュールを re-export する（読み取りモデル投影器 / ドメインイベント）
export * from "./cqrs.js";

// Attachment モジュールを re-export する（テナント分離ファイルストア）
export * from "./attachment.js";

// i18n formatter モジュールを re-export する（ICU 風 number / date / currency / unit）
export * from "./i18n_formatter.js";
