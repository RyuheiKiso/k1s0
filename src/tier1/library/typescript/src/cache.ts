/**
 * cache.ts — k1s0 tier1 Library TypeScript 実装: KeyValue / Cache の L3 interface
 * 08_キャッシュ適合仕様.md §CacheClient（OSS 中立 L3）に準拠する。
 * Redis / Memcached 等 OSS の API を一切露出しない Wire protocol 抽象 interface を宣言する。
 * wall-clock TTL 禁止規約に準拠して HLC LogicalTicks のみを TTL として受け付ける。
 */

/**
 * CacheTtl は HLC ベースの TTL を宣言する型。
 * wall-clock TTL 禁止規約に準拠して論理クロック差分 (logicalTicks) で表現する。
 * 実装側は HLC の hlc_lib を参照して変換する（Date.now() 等の直接使用を禁止する）。
 */
// CacheTtl 型定義: HLC ベースの TTL
export interface CacheTtl {
  // logicalTicks: HLC 論理クロック差分（tick 単位）
  // 1 tick = 実装固有の物理時間（通常はマイクロ秒単位）
  readonly logicalTicks: bigint;
}

/**
 * CacheSetOptions は CacheClient.set に渡すオプションを宣言する型。
 */
// CacheSetOptions 型定義
export interface CacheSetOptions {
  // ttl: HLC ベースのキャッシュ有効期限（未指定 = 無期限キャッシュ）
  // wall-clock TTL 禁止規約に準拠して HLC Duration のみを許容する。
  readonly ttl?: CacheTtl | undefined;
  // ifNotExists: true の場合はキーが存在しない場合のみ設定する（SET NX 相当）
  readonly ifNotExists?: boolean | undefined;
  // ifExists: true の場合はキーが既に存在する場合のみ更新する（SET XX 相当）
  readonly ifExists?: boolean | undefined;
}

/**
 * CacheClient は KeyValue / Cache の L3 抽象 interface を宣言する。
 * Redis / Memcached / DragonflyDB 等 OSS を透過的に切り替え可能にする。
 * OSS 型（redis.Client 等）を引数・戻り値に一切含まない。
 */
// CacheClient インターフェース定義
export interface CacheClient {
  /**
   * get はキーに対応する値を返す。
   * キーが存在しない場合は null を返す（エラーと区別する）。
   * tenantId を prefix とした名前空間で tenant 分離を保証する。
   */
  // get メソッド: キーの値を取得する
  get(tenantId: string, key: string): Promise<Uint8Array | null>;

  /**
   * set はキーと値のペアをキャッシュに保存する。
   * opts 未指定の場合は無期限キャッシュとして保存する。
   * tenantId を prefix とした名前空間で tenant 分離を保証する。
   */
  // set メソッド: キーと値のペアを保存する
  set(tenantId: string, key: string, value: Uint8Array, opts?: CacheSetOptions): Promise<void>;

  /**
   * delete はキーをキャッシュから削除する。
   * キーが存在しない場合はエラーを返さない（idempotent 操作）。
   * tenantId を prefix とした名前空間で tenant 分離を保証する。
   */
  // delete メソッド: キーを削除する
  delete(tenantId: string, key: string): Promise<void>;

  /**
   * exists はキーが存在するかどうかを返す。
   */
  // exists メソッド: キーの存在確認
  exists(tenantId: string, key: string): Promise<boolean>;

  /**
   * getMany は複数キーの値を一括取得する（MGET 相当）。
   * 戻り値は keys と同順の値配列で、存在しないキーは null とする。
   */
  // getMany メソッド: 複数キーの値を一括取得する
  getMany(tenantId: string, keys: readonly string[]): Promise<Array<Uint8Array | null>>;

  /**
   * setMany は複数キーと値のペアを一括保存する（MSET 相当）。
   * pairs はキーと値のペアマップ、opts は全ペアに適用する共通オプション。
   */
  // setMany メソッド: 複数キーと値のペアを一括保存する
  setMany(tenantId: string, pairs: ReadonlyMap<string, Uint8Array>, opts?: CacheSetOptions): Promise<void>;

  /**
   * deleteMany は複数キーを一括削除する（DEL 相当）。
   * 存在しないキーは無視する（idempotent 操作）。
   */
  // deleteMany メソッド: 複数キーを一括削除する
  deleteMany(tenantId: string, keys: readonly string[]): Promise<void>;

  /**
   * increment はキーの数値を delta だけアトミックに加算して新しい値を返す。
   * キーが存在しない場合は 0 を初期値として delta を加算する（INCRBY 相当）。
   */
  // increment メソッド: キーの数値をアトミックに加算する
  increment(tenantId: string, key: string, delta: bigint): Promise<bigint>;
}

/**
 * CacheLockOptions は分散ロック（RedLock / Fencing Token）のオプションを宣言する型。
 * wall-clock TTL 禁止規約に準拠して HLC ベースのロック有効期限のみを受け付ける。
 */
// CacheLockOptions 型定義
export interface CacheLockOptions {
  // ttl: HLC ベースのロック有効期限（必須: 無期限ロックは禁止する）
  readonly ttl: CacheTtl;
  // retryCount: ロック取得リトライ回数（0 = リトライなし）
  readonly retryCount?: number | undefined;
  // fencingToken: Fencing Token 機能を有効にするかどうか（stale write 防止）
  readonly fencingToken?: boolean | undefined;
}

/**
 * CacheLock はキャッシュ分散ロックの L3 抽象 interface を宣言する。
 * RedLock / Redisson 等を隠蔽する。
 */
// CacheLock インターフェース定義
export interface CacheLock {
  /**
   * acquire はロックを取得する（取得できなかった場合はエラーを投げる）。
   * name はロック名（tenantId 内で一意な識別子）。
   * 戻り値は Fencing Token として使用するロックトークン。
   */
  // acquire メソッド: ロックを取得する
  acquire(tenantId: string, name: string, opts: CacheLockOptions): Promise<string>;

  /**
   * release はロックを解放する（token は acquire で返されたトークン）。
   * token が不一致の場合はロックを解放せずにエラーを投げる（Fencing Token 保護）。
   */
  // release メソッド: ロックを解放する
  release(tenantId: string, name: string, token: string): Promise<void>;

  /**
   * refresh はロックの有効期限を延長する（long-running 処理用）。
   */
  // refresh メソッド: ロックの有効期限を延長する
  refresh(tenantId: string, name: string, token: string, opts: CacheLockOptions): Promise<void>;
}
