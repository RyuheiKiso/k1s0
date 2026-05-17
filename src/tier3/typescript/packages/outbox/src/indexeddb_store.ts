// k1s0 tier3 IndexedDB encrypted at rest ストア
// PQ（PendingQueue）と DR（Draft）の 2 layer を IndexedDB に暗号化永続化する
// device_bound_key（non-extractable AES-GCM 256bit）で at rest 暗号化を実現する
// fake-indexeddb 互換: グローバル indexedDB を注入可能にする設計

// IndexedDB の DB 名
const DB_NAME = "k1s0_outbox";
// IndexedDB のバージョン番号
const DB_VERSION = 1;
// PendingQueue の object store 名
const STORE_PQ = "pq";
// Draft の object store 名
const STORE_DR = "dr";
// device_bound_key メタデータの object store 名
const STORE_KEYS = "keys";
// 暗号化済みエントリの IDB キー定数（配列ストア）
const ENTRIES_KEY = "entries";

// 永続化する layer の種別（PendingQueue または Draft）
export type PersistLayer = "pq" | "dr";

// IndexedDB 暗号化エントリ（ストア内の物理形式）
interface EncryptedEntry {
  // 暗号化済みデータ（IV + ciphertext を Base64 エンコードした文字列）
  readonly encryptedPayload: string;
}

// IndexedDB のオープンリクエストをラップして Promise に変換するヘルパー
function promisifyRequest<T>(request: IDBRequest<T>): Promise<T> {
  // resolve / reject を IDB イベントにバインドする
  return new Promise<T>((resolve, reject) => {
    // 成功時は result を resolve する
    request.onsuccess = () => resolve(request.result as T);
    // 失敗時は error を reject する
    request.onerror = () => reject(request.error);
  });
}

// IDBDatabase を開く（テスト用に外部から indexedDB 実装を注入できる設計）
async function openDatabase(idb: IDBFactory = indexedDB): Promise<IDBDatabase> {
  // バージョン 1 でデータベースを開く（初回は onupgradeneeded が発火する）
  return new Promise<IDBDatabase>((resolve, reject) => {
    // openDB リクエストを生成する
    const req = idb.open(DB_NAME, DB_VERSION);
    // スキーマ初期化（初回 or バージョンアップ時）
    req.onupgradeneeded = (event) => {
      // アップグレード後のデータベースオブジェクトを取得する
      const db = (event.target as IDBOpenDBRequest).result;
      // PQ ストアが存在しない場合のみ作成する
      if (!db.objectStoreNames.contains(STORE_PQ)) {
        db.createObjectStore(STORE_PQ);
      }
      // DR ストアが存在しない場合のみ作成する
      if (!db.objectStoreNames.contains(STORE_DR)) {
        db.createObjectStore(STORE_DR);
      }
      // keys ストアが存在しない場合のみ作成する
      if (!db.objectStoreNames.contains(STORE_KEYS)) {
        db.createObjectStore(STORE_KEYS);
      }
    };
    // 成功時は IDBDatabase を resolve する
    req.onsuccess = () => resolve(req.result);
    // 失敗時は error を reject する
    req.onerror = () => reject(req.error);
    // ブロック時（他のタブが旧バージョンを保持）はエラーにする
    req.onblocked = () => reject(new Error("IDB open blocked by another tab"));
  });
}

// 指定ストアで readwrite トランザクションを開始するヘルパー
function rwtx(db: IDBDatabase, storeName: string): IDBObjectStore {
  // readwrite トランザクションを生成する
  const tx = db.transaction(storeName, "readwrite");
  // object store を返す
  return tx.objectStore(storeName);
}

// 指定ストアで readonly トランザクションを開始するヘルパー
function rotx(db: IDBDatabase, storeName: string): IDBObjectStore {
  // readonly トランザクションを生成する
  const tx = db.transaction(storeName, "readonly");
  // object store を返す
  return tx.objectStore(storeName);
}

// JSON オブジェクトを AES-GCM で暗号化して Base64 文字列に変換するヘルパー
async function encryptEntry(key: CryptoKey, data: object): Promise<string> {
  // JSON 文字列を UTF-8 バイト列に変換する
  const json = JSON.stringify(data);
  // TextEncoder で JSON 文字列をバイト列にエンコードする
  const bytes = new TextEncoder().encode(json);
  // GCM nonce（12 バイト）を乱数生成する
  const iv = crypto.getRandomValues(new Uint8Array(12));
  // AES-GCM で暗号化する（ArrayBuffer を直接渡す）
  const cipherBuf = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv },
    key,
    bytes,
  );
  // IV (12B) と ciphertext を連結する
  const combined = new Uint8Array(12 + cipherBuf.byteLength);
  // 先頭 12 バイトに IV をセットする
  combined.set(iv, 0);
  // IV の後ろに ciphertext をセットする
  combined.set(new Uint8Array(cipherBuf), 12);
  // Base64 文字列に変換して返す
  return btoa(String.fromCharCode(...combined));
}

// Base64 文字列を AES-GCM で復号して JSON オブジェクトに変換するヘルパー
async function decryptEntry(key: CryptoKey, encoded: string): Promise<object> {
  // Base64 文字列をバイト列にデコードする
  const combined = Uint8Array.from(atob(encoded), (c) => c.charCodeAt(0));
  // 先頭 12 バイトを IV として切り出す
  const iv = combined.slice(0, 12);
  // 残りを ciphertext として切り出す
  const ciphertext = combined.slice(12);
  // AES-GCM で復号する
  const plainBuf = await crypto.subtle.decrypt(
    { name: "AES-GCM", iv },
    key,
    ciphertext,
  );
  // バイト列を JSON 文字列にデコードする
  const json = new TextDecoder().decode(plainBuf);
  // JSON 文字列をオブジェクトに変換して返す
  return JSON.parse(json) as object;
}

// IndexedDB encrypted store の公開インターフェース
export interface IndexedDbStore {
  // 指定 layer に entry を暗号化して永続化する
  persist(layer: PersistLayer, entry: object): Promise<void>;
  // 指定 layer の全 entry を復号して返す
  restore(layer: PersistLayer): Promise<object[]>;
  // 指定 layer の全 entry を削除する（purge trigger 用）
  clear(layer: PersistLayer): Promise<void>;
}

// IndexedDB encrypted store の実装クラス
class IndexedDbStoreImpl implements IndexedDbStore {
  // 内部で使用する IDBDatabase インスタンス
  private readonly _db: IDBDatabase;
  // device_bound_key（non-extractable AES-GCM 256bit）
  private readonly _key: CryptoKey;

  // コンストラクタは private（createStore ファクトリを使う）
  private constructor(db: IDBDatabase, key: CryptoKey) {
    // データベースを保持する
    this._db = db;
    // 暗号鍵を保持する
    this._key = key;
  }

  // ファクトリ: DB オープン + device_bound_key 生成 or 取得を行う
  static async create(idb: IDBFactory = indexedDB): Promise<IndexedDbStoreImpl> {
    // データベースを開く
    const db = await openDatabase(idb);
    // device_bound_key を生成する（non-extractable → JS 外部に露出しない）
    const key = await crypto.subtle.generateKey(
      { name: "AES-GCM", length: 256 },
      // extractable: false → exportKey 禁止
      false,
      // encrypt / decrypt 用途のみ許可する
      ["encrypt", "decrypt"],
    );
    // 鍵メタデータ（生成日時）を keys ストアに記録する
    const keysMeta = rwtx(db, STORE_KEYS);
    // keys ストアに生成日時を保存する（鍵自体は non-extractable のため保存不可）
    await promisifyRequest(
      keysMeta.put({ generatedAt: new Date().toISOString() }, "device_bound_key_meta"),
    );
    // インスタンスを返す
    return new IndexedDbStoreImpl(db, key);
  }

  // 指定 layer に entry を暗号化して永続化する
  async persist(layer: PersistLayer, entry: object): Promise<void> {
    // 対象ストア名を取得する（pq または dr）
    const storeName = layer === "pq" ? STORE_PQ : STORE_DR;
    // entry を暗号化して Base64 文字列に変換する
    const encryptedPayload = await encryptEntry(this._key, entry);
    // 既存エントリ一覧を readonly で取得する
    const roStore = rotx(this._db, storeName);
    // 既存の暗号化済みリストを取得する（undefined の場合は空配列）
    const existing = await promisifyRequest(
      roStore.get(ENTRIES_KEY),
    ) as EncryptedEntry[] | undefined;
    // 既存リストに新 entry を追加する
    const updated: EncryptedEntry[] = [...(existing ?? []), { encryptedPayload }];
    // readwrite トランザクションで保存する
    const rwStore = rwtx(this._db, storeName);
    // 更新済みリストを保存する
    await promisifyRequest(rwStore.put(updated, ENTRIES_KEY));
  }

  // 指定 layer の全 entry を復号して返す
  async restore(layer: PersistLayer): Promise<object[]> {
    // 対象ストア名を取得する
    const storeName = layer === "pq" ? STORE_PQ : STORE_DR;
    // readonly トランザクションで暗号化済みリストを取得する
    const store = rotx(this._db, storeName);
    // 暗号化済みリストを取得する
    const encrypted = await promisifyRequest(
      store.get(ENTRIES_KEY),
    ) as EncryptedEntry[] | undefined;
    // 暗号化済みリストが存在しない場合は空配列を返す
    if (encrypted === undefined || encrypted.length === 0) {
      return [];
    }
    // 各 entry を復号して返す（並列処理で高速化する）
    return Promise.all(
      encrypted.map((e) => decryptEntry(this._key, e.encryptedPayload)),
    );
  }

  // 指定 layer の全 entry を削除する（logout / tenant_switch 時の purge）
  async clear(layer: PersistLayer): Promise<void> {
    // 対象ストア名を取得する
    const storeName = layer === "pq" ? STORE_PQ : STORE_DR;
    // readwrite トランザクションで全件削除する
    const store = rwtx(this._db, storeName);
    // ENTRIES_KEY を削除する
    await promisifyRequest(store.delete(ENTRIES_KEY));
  }
}

// IndexedDB encrypted store を生成するファクトリ関数（テスト時に idb を注入する）
// fake-indexeddb を使ったテストでは `createStore(fakeIdb)` のように呼ぶ
export async function createStore(idb: IDBFactory = indexedDB): Promise<IndexedDbStore> {
  // ファクトリで DB オープン + 鍵生成を行い、store インスタンスを返す
  return IndexedDbStoreImpl.create(idb);
}
