/**
 * k1s0 tier2 テナントコンテキスト TypeScript 実装
 * Rust 実装（tenant_context.rs）と 4 言語等価強度を持つ TypeScript 版
 * PostgreSQL session GUC を transaction 開始時に SET LOCAL で自動注入する
 */

/**
 * PostgreSQL session GUC の app.purpose 許容値
 * 10_テナント分離適合仕様.md の purpose enum と完全整合する
 */
// セッション目的の型定義（文字列リテラル型で許容値を制限する）
export type SessionPurpose =
  // 通常業務操作: テナント所有データへのアクセス
  | "business_op"
  // サポートアクセス: support_engineer role での PII 参照
  | "support"
  // データエクスポート: バッチ出力
  | "export"
  // マイグレーション: スキーマ移行期間中の特権操作
  | "migration"
  // 緊急オペレーション: 障害対応時の緊急権限（最小化・全記録必須）
  | "emergency";

/**
 * テナントセッションコンテキスト
 * 4 つの PostgreSQL session GUC をまとめて管理する
 * tenant_id は外部から直接 API 引数として渡せない（fromAuth 経由のみ）
 */
// TenantContext クラス定義
export class TenantContext {
  // テナント識別子（private: 外部からの直接アクセスを禁止する）
  readonly #tenantId: string;
  // アクター識別子（Keycloak subject）
  readonly #actorId: string;
  // セッション目的
  readonly #purpose: SessionPurpose;
  // 委譲チェーン（通常操作では空配列）
  readonly #delegationChain: readonly string[];

  // コンストラクタは private（fromAuth ファクトリメソッド経由のみ生成可）
  private constructor(
    tenantId: string,
    actorId: string,
    purpose: SessionPurpose,
    delegationChain: readonly string[],
  ) {
    // フィールドを初期化する
    this.#tenantId = tenantId;
    this.#actorId = actorId;
    this.#purpose = purpose;
    this.#delegationChain = delegationChain;
  }

  /**
   * 認証済み情報から TenantContext を生成するファクトリメソッド
   * tenant_id は AuthContext 経由のみ許容する設計
   */
  // fromAuth ファクトリメソッド（外部から呼べる唯一の生成経路）
  static fromAuth(
    tenantId: string,
    actorId: string,
    purpose: SessionPurpose,
  ): TenantContext {
    // 委譲チェーンは空で初期化する
    return new TenantContext(tenantId, actorId, purpose, []);
  }

  /**
   * 委譲元の actor を追加した TenantContext を返す（イミュータブルコピー）
   */
  // withDelegation メソッド（委譲操作専用）
  withDelegation(delegatorId: string): TenantContext {
    // 既存の委譲チェーンに delegator を追加した新しいインスタンスを返す
    return new TenantContext(
      this.#tenantId,
      this.#actorId,
      this.#purpose,
      [...this.#delegationChain, delegatorId],
    );
  }

  /**
   * 4 GUC を一括 SET LOCAL する SQL 文字列を返す
   * pg / postgres クライアントが実行する（このメソッドは SQL 生成のみ担う）
   */
  // toSetLocalSql メソッド（DB クライアントに渡す SQL を生成する）
  toSetLocalSql(): string {
    // delegation_chain の配列リテラルを構築する
    const chainLiteral =
      this.#delegationChain.length === 0
        ? "'{}'"
        : `'{${this.#delegationChain.map((s) => `"${s.replace(/"/g, '\\"')}"`).join(",")}}'`;
    // 4 GUC の SET LOCAL SQL を返す（SQL injection 対策: single quote をエスケープする）
    return [
      `SET LOCAL app.tenant_id = '${this.#tenantId.replace(/'/g, "''")}';`,
      `SET LOCAL app.actor_id = '${this.#actorId.replace(/'/g, "''")}';`,
      `SET LOCAL app.purpose = '${this.#purpose}';`,
      `SET LOCAL app.delegation_chain = ${chainLiteral};`,
    ].join(" ");
  }

  // tenantId ゲッター（リポジトリ抽象のみが使用する、外部パッケージへの直接露出は最小化）
  get tenantId(): string {
    // テナント ID を返す
    return this.#tenantId;
  }
}
