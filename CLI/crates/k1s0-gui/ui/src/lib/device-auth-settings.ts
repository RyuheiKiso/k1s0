// MEDIUM-009 監査対応: localStorage に代わる安全なストレージとして Tauri Store を使用する。
// WebView 内のスクリプトから読み取り可能な localStorage に機密設定を保存しないことで、
// XSS 攻撃時の設定情報窃取リスクを排除する。
// LazyStore はモジュールトップレベルで同期的に初期化可能で、初回アクセス時にストアをロードする。
// new Store() は Tauri plugin-store v2 で deprecated のため LazyStore を使用する。
import { LazyStore } from '@tauri-apps/plugin-store';

import type { DeviceAuthorizationSettings } from './tauri-commands';

// localStorage でのレガシーキー（移行処理で参照するために保持する）
export const DEVICE_AUTH_SETTINGS_STORAGE_KEY = 'k1s0.deviceAuthSettings';

// Tauri Store のファイル名（OS アプリデータディレクトリに保存される）
const STORE_FILENAME = 'device-auth-settings.json';

// Tauri Store 内でのキー名
const STORE_KEY = 'settings';

// LazyStore インスタンスを生成する（モジュール読み込み時に一度だけ作成する）。
// LazyStore は初回の get/set 呼び出し時に自動的にストアをロードするため、
// await なしでモジュールトップレベルに定義できる。
const store = new LazyStore(STORE_FILENAME);

/**
 * localStorage から Tauri Store へのマイグレーションを行う（初回のみ）。
 * localStorage に既存データが存在する場合は Tauri Store へ移行し、localStorage から削除する。
 */
async function migrateFromLocalStorage(): Promise<void> {
  try {
    const legacyRaw = window.localStorage.getItem(DEVICE_AUTH_SETTINGS_STORAGE_KEY);
    if (!legacyRaw) {
      return;
    }
    const parsed = JSON.parse(legacyRaw) as Partial<DeviceAuthorizationSettings>;
    const migrated: Partial<DeviceAuthorizationSettings> = {
      discovery_url: parsed.discovery_url?.trim() ?? '',
      client_id: parsed.client_id?.trim() ?? '',
      scope: parsed.scope?.trim() ?? '',
    };
    // Tauri Store への書き込みが成功した場合のみ localStorage から削除する
    await store.set(STORE_KEY, migrated);
    await store.save();
    window.localStorage.removeItem(DEVICE_AUTH_SETTINGS_STORAGE_KEY);
  } catch {
    // マイグレーション失敗は無視する（次回起動時に再試行される）
  }
}

/**
 * 保存済みの OIDC デバイス認証設定を読み込む。
 * localStorage にレガシーデータが存在する場合は Tauri Store へ自動移行する。
 */
export async function loadStoredDeviceAuthSettings(): Promise<Partial<DeviceAuthorizationSettings> | null> {
  // レガシーデータが存在する場合は先に移行する
  await migrateFromLocalStorage();

  try {
    const settings = await store.get<Partial<DeviceAuthorizationSettings>>(STORE_KEY);
    if (!settings) {
      return null;
    }
    return {
      discovery_url: settings.discovery_url?.trim() ?? '',
      client_id: settings.client_id?.trim() ?? '',
      scope: settings.scope?.trim() ?? '',
    };
  } catch {
    return null;
  }
}

/**
 * OIDC デバイス認証設定を Tauri Store に保存する。
 */
export async function storeDeviceAuthSettings(settings: DeviceAuthorizationSettings): Promise<void> {
  try {
    await store.set(STORE_KEY, settings);
    await store.save();
  } catch (error) {
    throw new Error(`Failed to save device auth settings: ${String(error)}`);
  }
}

/**
 * 保存済みの OIDC デバイス認証設定を Tauri Store から削除する。
 */
export async function clearStoredDeviceAuthSettings(): Promise<void> {
  try {
    await store.delete(STORE_KEY);
    await store.save();
  } catch (error) {
    throw new Error(`Failed to clear device auth settings: ${String(error)}`);
  }
}
