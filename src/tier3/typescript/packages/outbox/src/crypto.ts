// k1s0 tier3 WebCrypto AES-GCM ラッパー
// PQ / DR の IndexedDB encrypted at rest を実現する（5 層 defense-in-depth の層 D）
// non-extractable CryptoKey を使用する（device_bound_key）

// 暗号化パラメータ定数
const AES_GCM_ALGORITHM = "AES-GCM";
// AES-256 を使用する
const AES_KEY_LENGTH = 256;
// GCM nonce の長さ（96 bit）
const GCM_IV_LENGTH = 12;

// device_bound CryptoKey を生成する（OS keychain wrap の代理実装）
// non-extractable で生成することで key が JS 外部に露出しないことを保証する
export async function generateDeviceBoundKey(): Promise<CryptoKey> {
  // AES-GCM 256bit non-extractable key を生成する
  return await crypto.subtle.generateKey(
    { name: AES_GCM_ALGORITHM, length: AES_KEY_LENGTH },
    // extractable: false → key を外部に export 禁止
    false,
    // encrypt / decrypt の用途のみ許可する
    ["encrypt", "decrypt"],
  );
}

// Uint8Array を ArrayBuffer に変換するヘルパー（WebCrypto API の型要件に合わせる）
function toArrayBuffer(u8: Uint8Array): ArrayBuffer {
  // byteOffset / length を考慮して slice する
  return u8.buffer.slice(u8.byteOffset, u8.byteOffset + u8.byteLength) as ArrayBuffer;
}

// プレーンテキストを AES-GCM で暗号化する
export async function encryptData(
  key: CryptoKey,
  plaintext: Uint8Array,
): Promise<{ ciphertext: Uint8Array; iv: Uint8Array }> {
  // GCM nonce を乱数生成する
  const iv = crypto.getRandomValues(new Uint8Array(GCM_IV_LENGTH));
  // AES-GCM で暗号化する（WebCrypto API は ArrayBuffer を要求するため変換する）
  const ciphertext = await crypto.subtle.encrypt(
    { name: AES_GCM_ALGORITHM, iv: toArrayBuffer(iv) },
    key,
    toArrayBuffer(plaintext),
  );
  return { ciphertext: new Uint8Array(ciphertext), iv };
}

// AES-GCM で復号する
export async function decryptData(
  key: CryptoKey,
  ciphertext: Uint8Array,
  iv: Uint8Array,
): Promise<Uint8Array> {
  // AES-GCM で復号する（WebCrypto API は ArrayBuffer を要求するため変換する）
  const plaintext = await crypto.subtle.decrypt(
    { name: AES_GCM_ALGORITHM, iv: toArrayBuffer(iv) },
    key,
    toArrayBuffer(ciphertext),
  );
  return new Uint8Array(plaintext);
}

// JSON オブジェクトを暗号化して Base64 エンコードされた文字列に変換する
export async function encryptJson(key: CryptoKey, data: unknown): Promise<string> {
  // JSON 文字列を UTF-8 バイト列に変換する
  const json = JSON.stringify(data);
  const bytes = new TextEncoder().encode(json);
  // 暗号化する
  const { ciphertext, iv } = await encryptData(key, bytes);
  // IV + ciphertext を連結して Base64 エンコードする
  const combined = new Uint8Array(iv.length + ciphertext.length);
  combined.set(iv, 0);
  combined.set(ciphertext, iv.length);
  return btoa(String.fromCharCode(...combined));
}

// Base64 エンコードされた文字列を復号して JSON オブジェクトに変換する
export async function decryptJson<T>(key: CryptoKey, encoded: string): Promise<T> {
  // Base64 デコードして IV + ciphertext に分割する
  const combined = Uint8Array.from(atob(encoded), (c) => c.charCodeAt(0));
  const iv = combined.slice(0, GCM_IV_LENGTH);
  const ciphertext = combined.slice(GCM_IV_LENGTH);
  // 復号する
  const bytes = await decryptData(key, ciphertext, iv);
  // UTF-8 バイト列を JSON 文字列に変換する
  const json = new TextDecoder().decode(bytes);
  return JSON.parse(json) as T;
}
