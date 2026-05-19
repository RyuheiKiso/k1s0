// cache_layer.ts — Valkey TTL キャッシュ層 TypeScript 実装（設計方針 15）
// TenantContext を key prefix にしてテナント分離を保証する
// Outbox subscribe イベントで cache invalidation を実行する
// wall-clock TTL 禁止: TTL は HLC ベースで管理する

// デフォルト TTL: 5 分（300,000 ミリ秒）
export const DEFAULT_CACHE_TTL_MS = 300_000;

// Valkey キャッシュキーのオプション
export interface CacheKeyOptions {
  // tenantId: テナント識別子（テナント分離に使用する）
  readonly tenantId: string;
  // namespace: データ種別（"read_model" / "projection" / "aggregate"）
  readonly namespace: string;
  // key: エンティティ識別子
  readonly key: string;
}

// asValkeyKey はキャッシュキーオプションから Valkey キー文字列を生成する
export function asValkeyKey(opts: CacheKeyOptions): string {
  // k1s0:t2:cache:{tenant_id}:{namespace}:{key} 形式でキーを生成する
  return `k1s0:t2:cache:${opts.tenantId}:${opts.namespace}:${opts.key}`;
}

// invalidationPattern は同テナント同 namespace の全キーにマッチするパターンを返す
export function invalidationPattern(tenantId: string, namespace: string): string {
  // テナント + namespace の全エントリにマッチするパターンを返す
  return `k1s0:t2:cache:${tenantId}:${namespace}:*`;
}

// CacheEntry はキャッシュエントリのメタデータを保持するインターフェース
export interface CacheEntry<T> {
  // value: キャッシュする値
  readonly value: T;
  // cachedAtHlc: キャッシュ格納時の HLC タイムスタンプ
  readonly cachedAtHlc: string;
  // ttlMs: TTL（ミリ秒）
  readonly ttlMs: number;
}

// HLC タイムスタンプを生成する（wall-clock TTL 禁止規約に従い記録目的のみ使用する）
function generateHlcTimestamp(): string {
  // Date.now() は記録目的のみ使用許可（TTL/deadline 計算への使用は禁止）
  // NOTE: production では @k1s0/hlc ライブラリを使用することを推奨する
  const ms = Date.now();
  // HLC 形式: {ms_hex_16}-{logical_0000}-{node_0000}
  const hex = ms.toString(16).padStart(16, '0');
  // HLC 文字列を返す
  return `${hex}-0000-0000`;
}

// ValkeyClient は Valkey（Redis 互換）クライアントのインターフェース
// production では ioredis または node-redis を使用する
export interface ValkeyClient {
  // get: キーに対応する値を取得する
  get(key: string): Promise<string | null>;
  // setex: TTL 付きでキーと値を設定する
  setex(key: string, ttlSecs: number, value: string): Promise<void>;
  // scan: パターンマッチするキーを列挙する
  scan(cursor: string, pattern: string): Promise<[string, string[]]>;
  // unlink: キーを非同期削除する（KEYS/DEL の production-safe 代替）
  unlink(keys: string[]): Promise<number>;
}

// CacheLayer は Valkey TTL キャッシュ層を提供するクラス
export class CacheLayer {
  // _valkeyClient: 注入された Valkey クライアント
  private readonly _valkeyClient: ValkeyClient;
  // _ttlMs: 適用する TTL（ミリ秒）
  private readonly _ttlMs: number;

  // constructor は Valkey クライアントと TTL を受け取る
  constructor(valkeyClient: ValkeyClient, ttlMs = DEFAULT_CACHE_TTL_MS) {
    // Valkey クライアントを設定する
    this._valkeyClient = valkeyClient;
    // TTL を設定する
    this._ttlMs = ttlMs;
  }

  // withTtl は TTL をカスタム値に設定した新しい CacheLayer を返す
  withTtl(ttlMs: number): CacheLayer {
    // 新しいインスタンスを返す
    return new CacheLayer(this._valkeyClient, ttlMs);
  }

  // get は指定したキャッシュキーの値を取得する
  async get<T>(opts: CacheKeyOptions): Promise<CacheEntry<T> | null> {
    // Valkey キーを生成する
    const vkey = asValkeyKey(opts);
    // Valkey から raw JSON を取得する
    const raw = await this._valkeyClient.get(vkey);
    // キャッシュミスの場合は null を返す
    if (raw === null) {
      // キャッシュミスを返す
      return null;
    }
    // CacheEntry を JSON パースして返す
    const entry = JSON.parse(raw) as CacheEntry<T>;
    // キャッシュエントリを返す
    return entry;
  }

  // set は指定したキャッシュキーに値を書き込む
  async set<T>(opts: CacheKeyOptions, value: T): Promise<void> {
    // Valkey キーを生成する
    const vkey = asValkeyKey(opts);
    // HLC タイムスタンプを生成する（記録目的のみ）
    const hlcTs = generateHlcTimestamp();
    // CacheEntry を構築する
    const entry: CacheEntry<T> = {
      // 値を設定する
      value,
      // HLC タイムスタンプを設定する
      cachedAtHlc: hlcTs,
      // TTL を設定する
      ttlMs: this._ttlMs,
    };
    // CacheEntry を JSON シリアライズする
    const json = JSON.stringify(entry);
    // TTL を秒単位に変換する（最低 1 秒）
    const ttlSecs = Math.max(Math.floor(this._ttlMs / 1000), 1);
    // Valkey に SETEX で書き込む
    await this._valkeyClient.setex(vkey, ttlSecs, json);
  }

  // invalidateByOutbox は Outbox subscribe イベントを受けてキャッシュを無効化する
  async invalidateByOutbox(tenantId: string, namespace: string): Promise<number> {
    // 無効化パターンを生成する
    const pattern = invalidationPattern(tenantId, namespace);
    // 削除したキー数を初期化する
    let deletedCount = 0;
    // SCAN カーソルを初期化する
    let cursor = '0';
    // SCAN ループでパターンマッチするキーを収集する
    do {
      // SCAN コマンドを実行する
      const [nextCursor, keys] = await this._valkeyClient.scan(cursor, pattern);
      // 次のカーソルを更新する
      cursor = nextCursor;
      // マッチするキーが存在する場合は UNLINK で削除する
      if (keys.length > 0) {
        // UNLINK コマンドで非同期削除する
        const count = await this._valkeyClient.unlink(keys);
        // 削除件数を加算する
        deletedCount += count;
      }
    // カーソルが '0' になるまでループする（全スキャン完了）
    } while (cursor !== '0');
    // 削除件数を返す
    return deletedCount;
  }
}
