// k1s0 tier3 outbox persistence 高水準 API
// IndexedDB encrypted store（indexeddb_store.ts）の上位 facade として
// PendingQueue / Draft 層の persist / restore を型安全に扱う
// 全 layer purge（5 trigger）も本モジュールが責務を持つ

import { createStore } from "./indexeddb_store.js";
import type { IndexedDbStore, PersistLayer } from "./indexeddb_store.js";

// Persistence サービスのシングルトンインスタンス（once 初期化）
let _storeInstance: IndexedDbStore | null = null;

// IndexedDB store を初期化してシングルトンを返す
// テスト時に idb を注入することで fake-indexeddb を利用できる
async function getStore(idb?: IDBFactory): Promise<IndexedDbStore> {
  // 既にインスタンスが存在する場合はそのまま返す
  if (_storeInstance !== null) {
    return _storeInstance;
  }
  // 新規に store を生成する
  _storeInstance = await createStore(idb);
  // 生成した store を返す
  return _storeInstance;
}

// テスト用: シングルトンをリセットする（teardown 時に呼ぶ）
export function _resetStoreForTest(): void {
  // シングルトンをクリアする
  _storeInstance = null;
}

// PendingQueue エントリを暗号化して永続化する高水準 API
// PendingQueueEntry は outbox.ts の型に依存せず object として受け取る
export async function persistPendingQueueEntry(
  entry: object,
  idb?: IDBFactory,
): Promise<void> {
  // store インスタンスを取得する
  const store = await getStore(idb);
  // 'pq' layer に暗号化保存する
  await store.persist("pq", entry);
}

// Draft エントリを暗号化して永続化する高水準 API
export async function persistDraftEntry(
  entry: object,
  idb?: IDBFactory,
): Promise<void> {
  // store インスタンスを取得する
  const store = await getStore(idb);
  // 'dr' layer に暗号化保存する
  await store.persist("dr", entry);
}

// PendingQueue の全 entry を復号して返す高水準 API
export async function restorePendingQueue(idb?: IDBFactory): Promise<object[]> {
  // store インスタンスを取得する
  const store = await getStore(idb);
  // 'pq' layer の全 entry を復号して返す
  return store.restore("pq");
}

// Draft の全 entry を復号して返す高水準 API
export async function restoreDraft(idb?: IDBFactory): Promise<object[]> {
  // store インスタンスを取得する
  const store = await getStore(idb);
  // 'dr' layer の全 entry を復号して返す
  return store.restore("dr");
}

// 指定 layer の全 entry を削除する高水準 API（layer 単独クリア）
export async function clearLayer(layer: PersistLayer, idb?: IDBFactory): Promise<void> {
  // store インスタンスを取得する
  const store = await getStore(idb);
  // 指定 layer をクリアする
  await store.clear(layer);
}

// 全 layer を purge する高水準 API（5 trigger 対応）
// 5 trigger: logout / refresh_token_expiry / tenant_switch / actor_switch / device_bound_key_rotate
export async function purgeAllLayers(idb?: IDBFactory): Promise<void> {
  // store インスタンスを取得する
  const store = await getStore(idb);
  // PQ と DR の両 layer を順番にクリアする
  await store.clear("pq");
  // DR もクリアする
  await store.clear("dr");
}

// 汎用 persist API（layer を引数で指定する）
export async function persist(
  layer: PersistLayer,
  entry: object,
  idb?: IDBFactory,
): Promise<void> {
  // store インスタンスを取得する
  const store = await getStore(idb);
  // 指定 layer に暗号化保存する
  await store.persist(layer, entry);
}

// 汎用 restore API（layer を引数で指定する）
export async function restore(
  layer: PersistLayer,
  idb?: IDBFactory,
): Promise<object[]> {
  // store インスタンスを取得する
  const store = await getStore(idb);
  // 指定 layer の全 entry を復号して返す
  return store.restore(layer);
}
