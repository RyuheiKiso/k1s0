/**
 * dbDistributed.ts — k1s0 tier1 Library TypeScript 実装: Relational Store / Distributed SQL の L1+ interface
 * 14_分散SQL適合仕様.md §DistributedDbClient（CockroachDB / Spanner L1+ 深耕）に準拠する。
 * Distributed SQL の full API を Library 独自語彙で表現しつつ、AuthContext 伝播と tenant 分離を強制する。
 * OSS 型（pg / spanner 等）を公開シグネチャに一切含まない。
 */

// DbTxIsoLevel / DbTx は単一ノード DB と共通なので db.ts から import する
import type { DbTxIsoLevel, DbTx } from "./db.js";

/**
 * DistributedTxPriority はトランザクションの実行優先度を宣言する enum。
 * CockroachDB の transaction priority に準拠した Library 独自語彙とする。
 */
// DistributedTxPriority 列挙型定義
export const enum DistributedTxPriority {
  // Normal: 通常優先度（デフォルト）
  Normal = "normal",
  // High: 高優先度（コンテンション時に優先してコミットする）
  High = "high",
  // Low: 低優先度（バックグラウンドジョブに使用する）
  Low = "low",
}

/**
 * DistributedTxOptions は Distributed SQL のトランザクション開始オプションを宣言する型。
 * DbTxOptions を継承しつつ、Distributed SQL 固有オプションを追加する。
 */
// DistributedTxOptions 型定義
export interface DistributedTxOptions {
  // isoLevel: トランザクション分離レベル（Distributed SQL では Serializable 推奨）
  readonly isoLevel?: DbTxIsoLevel | undefined;
  // readOnly: 読み取り専用トランザクションかどうか
  readonly readOnly?: boolean | undefined;
  // priority: Distributed SQL のトランザクション優先度
  readonly priority?: DistributedTxPriority | undefined;
  // asOfSystemTimeTick: 過去の特定タイムスタンプを読む（AOST: CockroachDB 固有機能）
  // wall-clock TTL 禁止規約に準拠して HLC tick 値で指定する（0 = 最新を読む）。
  readonly asOfSystemTimeTick?: bigint | undefined;
  // maxRetries: 自動リトライ回数（楽観的ロック競合時に自動リトライする）
  readonly maxRetries?: number | undefined;
}

/**
 * DistributedDbTx は Distributed SQL のアクティブなトランザクションを宣言する interface。
 * DbTx を継承しつつ、Distributed SQL 固有メソッドを追加する。
 */
// DistributedDbTx インターフェース定義
export interface DistributedDbTx extends DbTx {
  /**
   * savepoint は名前付きセーブポイントを作成する（ネストトランザクション用）。
   * name はセーブポイント名（英数字 + アンダースコアのみ許容する）。
   */
  // savepoint メソッド: セーブポイントを作成する
  savepoint(name: string): Promise<void>;

  /**
   * rollbackToSavepoint はセーブポイントにロールバックする（部分ロールバック）。
   */
  // rollbackToSavepoint メソッド: セーブポイントにロールバックする
  rollbackToSavepoint(name: string): Promise<void>;

  /**
   * releaseSavepoint はセーブポイントをリリースする（セーブポイントを確定する）。
   */
  // releaseSavepoint メソッド: セーブポイントをリリースする
  releaseSavepoint(name: string): Promise<void>;

  /** priority は現在のトランザクション優先度を返す。 */
  // priority プロパティ
  readonly priority: DistributedTxPriority;
}

/**
 * DistributedDbClient は Relational Store / Distributed SQL の L1+ 抽象 interface を宣言する。
 * CockroachDB / Google Cloud Spanner / YugabyteDB 等を抽象化する。
 * OSS 型を一切含まない。
 * 単一ノード PostgreSQL と区別するために DbClient を継承しない（別 interface として宣言する）。
 */
// DistributedDbClient インターフェース定義
export interface DistributedDbClient {
  /**
   * begin はトランザクションを開始して DistributedDbTx を返す。
   * opts は Distributed SQL 固有オプション（priority / AOST 等）。
   */
  // begin メソッド: トランザクションを開始する
  begin(opts?: DistributedTxOptions): Promise<DistributedDbTx>;

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
   * inTx はトランザクション内でコールバック fn を実行する（自動リトライ + 自動 COMMIT/ROLLBACK）。
   * fn が reject した場合は自動 ROLLBACK 後に opts.maxRetries まで自動リトライする。
   */
  // inTx メソッド: トランザクション内でコールバックを実行する
  inTx<T>(
    opts: DistributedTxOptions | undefined,
    fn: (tx: DistributedDbTx) => Promise<T>,
  ): Promise<T>;

  /**
   * ping は DB への接続確認を行う（health check 用途）。
   */
  // ping メソッド: 接続確認を行う
  ping(): Promise<void>;

  /**
   * nodeInfo は接続先 Distributed SQL クラスターのノード情報を返す（診断用）。
   */
  // nodeInfo メソッド: クラスターのノード情報を取得する
  nodeInfo(): Promise<DistributedDbNodeInfo>;
}

/**
 * DistributedDbNodeInfo は Distributed SQL クラスターのノード情報を宣言する型。
 */
// DistributedDbNodeInfo 型定義
export interface DistributedDbNodeInfo {
  // nodeCount: クラスター内のノード数
  readonly nodeCount: number;
  // region: 接続先リージョン名
  readonly region: string;
  // version: DB バージョン文字列
  readonly version: string;
  // isReadOnly: 接続先ノードが読み取り専用かどうか（リードレプリカ接続時）
  readonly isReadOnly: boolean;
}

/**
 * BulkInsertOptions は Distributed SQL への大量データ一括挿入オプションを宣言する型。
 */
// BulkInsertOptions 型定義
export interface BulkInsertOptions {
  // batchSize: 1 バッチあたりの行数
  readonly batchSize: number;
  // onConflict: 主キー競合時の処理（"update" / "ignore" / "error"）
  readonly onConflict?: "update" | "ignore" | "error" | undefined;
  // returnInserted: 挿入した行のデータを戻り値に含めるかどうか（RETURNING 句相当）
  readonly returnInserted?: boolean | undefined;
}

/**
 * DistributedBulkClient は Distributed SQL への大量データ挿入に特化した interface を宣言する。
 */
// DistributedBulkClient インターフェース定義
export interface DistributedBulkClient {
  /**
   * bulkInsert は複数行を一括挿入する（バッチ分割は自動で行う）。
   * 戻り値は挿入した行数。
   */
  // bulkInsert メソッド: 複数行を一括挿入する
  bulkInsert(
    table: string,
    columns: readonly string[],
    rows: ReadonlyArray<readonly unknown[]>,
    opts?: BulkInsertOptions,
  ): Promise<number>;

  /**
   * bulkUpsert は複数行を一括 UPSERT する（INSERT ON CONFLICT DO UPDATE 相当）。
   */
  // bulkUpsert メソッド: 複数行を一括 UPSERT する
  bulkUpsert(
    table: string,
    columns: readonly string[],
    rows: ReadonlyArray<readonly unknown[]>,
    conflictColumns: readonly string[],
    opts?: BulkInsertOptions,
  ): Promise<number>;
}
