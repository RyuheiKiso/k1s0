/**
 * db.ts — k1s0 tier1 Library TypeScript 実装: Relational Store / Single-leader の L1+ interface
 * 13_リレーショナルDB適合仕様.md §DbClient（PostgreSQL L1+ 深耕）に準拠する。
 * PostgreSQL の full API を Library 独自語彙で表現しつつ、AuthContext 伝播と RLS を強制する。
 * OSS 型（pg / postgres 等）を公開シグネチャに一切含まない。
 */

/**
 * DbTxIsoLevel はトランザクション分離レベルを宣言する enum。
 * PostgreSQL の isolation level に準拠した Library 独自語彙とする。
 */
// DbTxIsoLevel 列挙型定義
export const enum DbTxIsoLevel {
  // ReadCommitted: Read Committed（デフォルト: RLS と組み合わせて tenant 分離を保証する）
  ReadCommitted = "read_committed",
  // RepeatableRead: Repeatable Read（整合性スナップショット読み取り）
  RepeatableRead = "repeatable_read",
  // Serializable: Serializable（SSI: 完全な直列化保証）
  Serializable = "serializable",
}

/**
 * DbTxOptions はトランザクション開始オプションを宣言する型。
 */
// DbTxOptions 型定義
export interface DbTxOptions {
  // isoLevel: トランザクション分離レベル（未指定 = DbTxIsoLevel.ReadCommitted）
  readonly isoLevel?: DbTxIsoLevel | undefined;
  // readOnly: 読み取り専用トランザクションかどうか（true = BEGIN READ ONLY）
  readonly readOnly?: boolean | undefined;
  // deferConstraints: 制約チェックを DEFERRED にするかどうか
  readonly deferConstraints?: boolean | undefined;
}

/**
 * DbTx はアクティブなトランザクションを宣言する interface。
 * OSS の pg.PoolClient / postgres.TransactionSql を露出せず Library 独自語彙で表現する。
 * 全ての DB 操作は AuthContext を伝播して RLS を機能させる。
 */
// DbTx インターフェース定義
export interface DbTx {
  /**
   * queryRow は単一行クエリを実行して結果行を返す。
   * query はプリペアドクエリの SQL テンプレート（$1 / $2 等のプレースホルダー形式）。
   * 生 SQL 文字列受付 API を禁止する（template literal での SQL 組み立て禁止）。
   * 戻り値が null の場合は行が存在しない（ErrDbNoRows 相当）。
   */
  // queryRow メソッド: 単一行クエリを実行する
  queryRow<T>(query: string, ...args: unknown[]): Promise<T | null>;

  /**
   * query は複数行クエリを実行して結果行の配列を返す。
   */
  // query メソッド: 複数行クエリを実行する
  query<T>(query: string, ...args: unknown[]): Promise<readonly T[]>;

  /**
   * exec は DML / DDL を実行して変更された行数を返す。
   */
  // exec メソッド: DML を実行する
  exec(query: string, ...args: unknown[]): Promise<number>;

  /**
   * commit はトランザクションをコミットする。
   */
  // commit メソッド: トランザクションをコミットする
  commit(): Promise<void>;

  /**
   * rollback はトランザクションをロールバックする。
   * エラー / finally ブロックで必ず呼び出す（idempotent 操作: 既にコミット済みでも安全）。
   */
  // rollback メソッド: トランザクションをロールバックする
  rollback(): Promise<void>;
}

/**
 * DbClient は Relational Store / Single-leader の L1+ 抽象 interface を宣言する。
 * PostgreSQL の full API を Library 独自語彙で表現する。
 * OSS 型（pg.Pool / postgres 等）を一切含まない。
 * AuthContext が伝播されていることを前提とする（RLS / GUC 設定のため必須）。
 */
// DbClient インターフェース定義
export interface DbClient {
  /**
   * begin はトランザクションを開始して DbTx を返す。
   * 実装側は BEGIN 後に AuthContext.toGucSetters() で GUC を SET LOCAL する。
   */
  // begin メソッド: トランザクションを開始する
  begin(opts?: DbTxOptions): Promise<DbTx>;

  /**
   * queryRow は単一行クエリを autocommit モードで実行する。
   */
  // queryRow メソッド: 単一行クエリを autocommit で実行する
  queryRow<T>(query: string, ...args: unknown[]): Promise<T | null>;

  /**
   * query は複数行クエリを autocommit モードで実行する。
   */
  // query メソッド: 複数行クエリを autocommit で実行する
  query<T>(query: string, ...args: unknown[]): Promise<readonly T[]>;

  /**
   * exec は DML を autocommit モードで実行する。
   */
  // exec メソッド: DML を autocommit で実行する
  exec(query: string, ...args: unknown[]): Promise<number>;

  /**
   * inTx はトランザクション内でコールバック fn を実行する（COMMIT / ROLLBACK は自動管理）。
   * fn が Promise を reject した場合は自動 ROLLBACK する。
   * fn が resolve した場合は自動 COMMIT する。
   */
  // inTx メソッド: トランザクション内でコールバックを実行する
  inTx<T>(opts: DbTxOptions | undefined, fn: (tx: DbTx) => Promise<T>): Promise<T>;

  /**
   * ping は DB への接続確認を行う（health check 用途）。
   */
  // ping メソッド: 接続確認を行う
  ping(): Promise<void>;
}

/**
 * DbPoolStats は接続プールの統計情報を宣言する型。
 * OSS の Pool stats を露出せず Library 独自語彙で表現する。
 */
// DbPoolStats 型定義
export interface DbPoolStats {
  // totalConnections: 接続プールの総接続数
  readonly totalConnections: number;
  // idleConnections: アイドル状態の接続数
  readonly idleConnections: number;
  // acquiredConnections: 使用中の接続数
  readonly acquiredConnections: number;
  // waitCount: 接続待ちリクエスト数
  readonly waitCount: number;
  // maxConnections: 接続プールの最大接続数設定値
  readonly maxConnections: number;
}
