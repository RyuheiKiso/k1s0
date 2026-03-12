import type { DeviceAuthorizationSettings } from './tauri-commands';

export const DEVICE_AUTH_SETTINGS_STORAGE_KEY = 'k1s0.deviceAuthSettings';

export function loadStoredDeviceAuthSettings(): Partial<DeviceAuthorizationSettings> | null {
  const raw = window.localStorage.getItem(DEVICE_AUTH_SETTINGS_STORAGE_KEY);
  if (!raw) {
    return null;
  }

  try {
    const parsed = JSON.parse(raw) as Partial<DeviceAuthorizationSettings>;
    return {
      discovery_url: parsed.discovery_url?.trim() ?? '',
      client_id: parsed.client_id?.trim() ?? '',
      scope: parsed.scope?.trim() ?? '',
    };
  } catch {
    return null;
  }
}

export function storeDeviceAuthSettings(settings: DeviceAuthorizationSettings) {
  window.localStorage.setItem(DEVICE_AUTH_SETTINGS_STORAGE_KEY, JSON.stringify(settings));
}

export function clearStoredDeviceAuthSettings() {
  window.localStorage.removeItem(DEVICE_AUTH_SETTINGS_STORAGE_KEY);
}
