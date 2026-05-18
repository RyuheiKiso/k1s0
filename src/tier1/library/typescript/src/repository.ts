/**
 * repository.ts — k1s0 tier1 Library TypeScript 実装: Repository<T> interface
 * 04_認証適合仕様.md §CI 不変条件 整合 5「生 SQL 文字列受付 API 禁止」に準拠する。
 * tenant_id を必須引数として受け取る（SQL 文字列受付 API は提供しない）。
 * 実装クラスは pg / postgres クライアントの parameterized query のみを使用する。
 */

/**
 * Repository<T> は生 SQL 文字列を受け取らない DB アクセス抽象 interface。
 * tenant_id は必須引数として受け取る（RLS と二重で tenant 境界を保証する）。
 * T は DB に永続化されるドメイン型。
 */
// Repository インターフェース定義
export interface Repository<T> {
  /**
   * findById は id と tenantId を受け取り、エンティティを返す。
   * tenantId は必須引数（RLS と二重で tenant 境界を保証する）。
   * エンティティが見つからない場合は null を返す。
   */
  // findById メソッド: id と tenantId でエンティティを取得する
  findById(
    // id: エンティティの主キー（UUID 文字列）
    id: string,
    // tenantId: テナント識別子（必須引数）
    tenantId: string,
  ): Promise<T | null>;

  /**
   * save はエンティティと tenantId を受け取り、DB に保存する。
   * tenantId は必須引数（RLS と二重で tenant 境界を保証する）。
   */
  // save メソッド: エンティティを保存する
  save(
    // entity: 保存するエンティティ
    entity: T,
    // tenantId: テナント識別子（必須引数）
    tenantId: string,
  ): Promise<void>;
}

/**
 * RlsBypassError は RLS bypass 検出時に throw するエラークラス。
 * RLS が物理的に保証するが、アプリ層でも二重検証する設計。
 */
// RlsBypassError クラス定義
export class RlsBypassError extends Error {
  // entityTenantId: エンティティに記録されたテナント ID
  readonly entityTenantId: string;
  // contextTenantId: コンテキストに設定されたテナント ID
  readonly contextTenantId: string;

  // コンストラクタ: entityTenantId と contextTenantId を受け取る
  constructor(entityTenantId: string, contextTenantId: string) {
    // Error クラスに RLS bypass メッセージを渡す
    super(
      `RLS bypass detected: entity.tenant_id=${entityTenantId} != context.tenant_id=${contextTenantId}`,
    );
    // name を設定する（Error サブクラスの標準的な作法）
    this.name = "RlsBypassError";
    // entityTenantId を設定する
    this.entityTenantId = entityTenantId;
    // contextTenantId を設定する
    this.contextTenantId = contextTenantId;
  }
}

/**
 * verifyTenantId は entity の tenantId が context の tenantId と一致することを検証する。
 * RLS が物理的に保証するが、アプリ層でも二重検証する設計。
 * 不一致の場合は RlsBypassError を throw する。
 */
// verifyTenantId ヘルパー関数: テナント ID の一致を検証する
export function verifyTenantId(entityTenantId: string, contextTenantId: string): void {
  // テナント ID が一致しない場合は RlsBypassError を throw する
  if (entityTenantId !== contextTenantId) {
    throw new RlsBypassError(entityTenantId, contextTenantId);
  }
  // テナント ID が一致した場合は何もしない（正常）
}
