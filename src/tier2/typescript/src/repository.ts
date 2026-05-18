/**
 * k1s0 tier2 Repository abstraction TypeScript 実装
 * Rust 実装（repository.rs）と 4 言語等価強度を持つ TypeScript 版
 * 生 SQL 文字列を受け取る public API を持たない設計で tier2 の DB アクセスを封鎖する
 */

// TenantContext を import する（GUC 注入に使用する）
import { TenantContext } from "./tenantContext.js";

/**
 * RlsBypassError: RLS bypass 検出エラー
 * SELECT 結果の tenant_id が TenantContext と一致しない場合にスローする
 * RLS が物理的に保証するが、アプリ層でも二重検証する設計
 */
// RlsBypassError クラス定義
export class RlsBypassError extends Error {
  // エンティティの tenant_id
  readonly entityTenantId: string;
  // コンテキストの tenant_id
  readonly contextTenantId: string;

  // コンストラクタ（entity と context の tenant_id を受け取る）
  constructor(entityTenantId: string, contextTenantId: string) {
    // エラーメッセージを構築する
    super(
      `RLS bypass detected: entity.tenant_id=${entityTenantId}, context.tenant_id=${contextTenantId}`,
    );
    // エラー名を設定する
    this.name = "RlsBypassError";
    // フィールドを初期化する
    this.entityTenantId = entityTenantId;
    this.contextTenantId = contextTenantId;
  }
}

/**
 * TenantScopedEntity: テナントスコープ内のエンティティを表す汎用インターフェース
 * PII 列は含まない（pii_segregated table は別の型で管理する）
 */
// TenantScopedEntity インターフェース定義
export interface TenantScopedEntity {
  // エンティティの主キー（UUID 文字列）
  readonly id: string;
  // テナント ID（read-only、RLS が保証する）
  readonly tenantId: string;
  // エンティティバージョン（楽観的ロックに使用する）
  readonly version: number;
  // エンティティペイロード（業界中立的な汎用 JSON）
  readonly payload: unknown;
}

/**
 * Repository<T>: テナントスコープ内のエンティティ操作インターフェース
 * tenant_id は引数で受け取らず、実装内部で TenantContext から注入する
 */
// Repository インターフェース定義（ジェネリクスで型安全を保証する）
export interface Repository<T extends TenantScopedEntity = TenantScopedEntity> {
  /**
   * エンティティを主キーで取得する
   * tenant_id は引数で受け取らず、内部的に TenantContext から注入する
   * 見つからない場合は null を返す
   */
  // findById メソッド（主キー検索操作）
  findById(id: string): Promise<T | null>;

  /**
   * テナント ID に紐づく全エンティティを取得する
   * tenant_id は AuthContext から取得するため引数で受け取らない
   */
  // findByTenantId メソッド（テナントスコープ一覧操作）
  findByTenantId(): Promise<T[]>;

  /**
   * エンティティを永続化する（INSERT または UPDATE）
   * tenant_id は TenantContext から注入するため entity に含めない
   */
  // save メソッド（永続化操作）
  save(entity: T): Promise<void>;

  /**
   * エンティティを削除する
   * tenant_id は TenantContext から自動注入される（API 引数経由禁止）
   */
  // delete メソッド（削除操作）
  delete(id: string): Promise<void>;
}

/**
 * RepositoryContext: Repository を実行するコンテキスト
 * TenantContext をラップして DB アクセス時の GUC 注入を担う
 * 生 SQL 文字列は受け取らない設計を型で表現する
 */
// RepositoryContext クラス定義
export class RepositoryContext {
  // テナントコンテキスト（GUC 注入に使用する、外部からの直接アクセスを禁止する）
  readonly #tenantContext: TenantContext;

  // コンストラクタ（TenantContext を受け取る）
  constructor(tenantContext: TenantContext) {
    // TenantContext を格納する
    this.#tenantContext = tenantContext;
  }

  /**
   * テナントコンテキストを参照する（読み取り専用）
   */
  // tenantContext ゲッター（Repository 実装が使用する）
  get tenantContext(): TenantContext {
    // テナントコンテキストを返す
    return this.#tenantContext;
  }

  /**
   * SET LOCAL SQL を取得する
   * Repository 実装が DB に注入するために使用する（4 GUC を一括 SET LOCAL する）
   */
  // setLocalSql メソッド（DB クライアントに渡す GUC SQL を生成する）
  setLocalSql(): string {
    // TenantContext の toSetLocalSql() を呼んで SQL を返す
    return this.#tenantContext.toSetLocalSql();
  }
}

/**
 * verifySelectResult: SELECT 結果の tenant_id が TenantContext と一致することを検証する
 * RLS が物理的に保証するが、アプリ層でも二重検証する設計
 * 一致しない場合は RlsBypassError をスローする
 */
// verifySelectResult 関数（アプリ層での二重 RLS 検証）
export function verifySelectResult(
  entity: TenantScopedEntity,
  ctx: RepositoryContext,
): void {
  // TenantContext の tenantId を取得する
  const expectedTenantId = ctx.tenantContext.tenantId;
  // エンティティの tenant_id と TenantContext の tenant_id を比較する
  if (entity.tenantId !== expectedTenantId) {
    // RLS bypass が発生した場合は即座に RlsBypassError をスローする
    throw new RlsBypassError(entity.tenantId, expectedTenantId);
  }
}
