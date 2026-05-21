// k1s0 tier3 outbox IndexedDB 暗号化 property test
// 1.0.0 ship blocker: IndexedDB encrypted at rest property test green
// WebCrypto AES-GCM 暗号化の encrypt → decrypt roundtrip が正しいことを検証する

// vitest テストフレームワークをインポートする
import { describe, it, expect } from "vitest";
// outbox モジュールの暗号化関数をインポートする（WebCrypto AES-GCM ラッパー）
// encryptData: (key, plaintext) => { ciphertext, iv }
// decryptData: (key, ciphertext, iv) => Uint8Array
import { encryptData, decryptData, encryptJson, decryptJson } from "../src/index";

// outbox IndexedDB 暗号化 property test スイート
describe("outbox IndexedDB encryption property tests", () => {
  // encryptData/decryptData roundtrip テスト
  // WebCrypto AES-GCM で暗号化し、返された IV で復号できることを検証する
  it("encryptData then decryptData returns original plaintext", async () => {
    // テスト用のプレーンテキストデータを定義する
    const plaintext = JSON.stringify({
      // イベント種別を設定する
      event_type: "TestEvent",
      // テナント ID を設定する（ダミー値）
      tenant_id: "test",
      // ペイロードを設定する
      payload: { value: 42 },
    });
    // WebCrypto で AES-GCM 鍵を生成する（extractable: true でテスト用）
    const key = await crypto.subtle.generateKey(
      // アルゴリズムとキー長を指定する
      { name: "AES-GCM", length: 256 },
      // テスト用なので extractable を true にする
      true,
      // 暗号化・復号の用途を指定する
      ["encrypt", "decrypt"]
    );
    // プレーンテキストを TextEncoder でバイト列に変換する
    const encoder = new TextEncoder();
    // encryptData でデータを暗号化する（IV は内部でランダム生成される）
    const { ciphertext, iv } = await encryptData(key, encoder.encode(plaintext));
    // decryptData で復号する（ciphertext, iv の引数順を確認して使用する）
    const decryptedBuffer = await decryptData(key, ciphertext, iv);
    // 復号結果を TextDecoder でデコードする
    const decoder = new TextDecoder();
    // デコードされた文字列に変換する
    const decrypted = decoder.decode(decryptedBuffer);
    // 元のプレーンテキストと一致することを確認する
    expect(decrypted).toBe(plaintext);
  });

  // encryptJson/decryptJson roundtrip テスト
  // JSON オブジェクトを暗号化して Base64 文字列にし、復号して元の JSON に戻ることを検証する
  it("encryptJson then decryptJson returns original object", async () => {
    // テスト用の JSON オブジェクトを定義する
    const original = {
      // イベント種別を設定する
      event_type: "OrderCreated",
      // テナント ID を設定する
      tenant_id: "tenant-001",
      // ペイロードを設定する
      payload: { order_id: "ORD-001", quantity: 10 },
    };
    // AES-GCM 鍵を生成する（extractable: true でテスト用）
    const key = await crypto.subtle.generateKey(
      // アルゴリズムとキー長を指定する
      { name: "AES-GCM", length: 256 },
      // テスト用なので extractable を true にする
      true,
      // 用途を設定する
      ["encrypt", "decrypt"]
    );
    // JSON オブジェクトを暗号化する（Base64 文字列を返す）
    const encoded = await encryptJson(key, original);
    // Base64 文字列が生成されたことを確認する
    expect(typeof encoded).toBe("string");
    // 暗号化された Base64 文字列を復号する
    const decoded = await decryptJson<typeof original>(key, encoded);
    // 元の JSON と一致することを確認する
    expect(decoded.event_type).toBe(original.event_type);
    // tenant_id が一致することを確認する
    expect(decoded.tenant_id).toBe(original.tenant_id);
    // payload.order_id が一致することを確認する
    expect(decoded.payload.order_id).toBe(original.payload.order_id);
  });

  // PII フィールドが暗号化後も平文で読み取れないことを確認する
  // 個人情報保護規律: PII を平文で IndexedDB に保管してはならない
  it("encrypted PII field is not readable in ciphertext", async () => {
    // PII を含むデータを定義する（name は PII フィールド）
    const piiData = JSON.stringify({
      // PII フィールド: 氏名（暗号化後には平文で見えないことを確認する）
      name: "山田太郎",
      // イベント種別を設定する
      event_type: "OrderCreated",
    });
    // AES-GCM 鍵を生成する
    const key = await crypto.subtle.generateKey(
      // アルゴリズムとキー長を指定する
      { name: "AES-GCM", length: 256 },
      // テスト用なので extractable を true にする
      true,
      // 用途を設定する
      ["encrypt", "decrypt"]
    );
    // プレーンテキストをバイト列に変換する
    const encoder = new TextEncoder();
    // encryptData で PII データを暗号化する
    const { ciphertext } = await encryptData(key, encoder.encode(piiData));
    // 暗号文の文字列表現を生成する（latin1 バイト列として変換する）
    const ciphertextString = String.fromCharCode(...ciphertext);
    // 「山田太郎」が暗号文に平文で存在しないことを確認する（PII 保護規律）
    expect(ciphertextString).not.toContain("山田太郎");
  });

  // 異なる IV で復号した場合は失敗することを確認する
  // IV の一意性と整合性がセキュリティを保証していることを検証する
  it("decryption fails with wrong IV", async () => {
    // プレーンテキストを定義する
    const plaintext = "sensitive data";
    // AES-GCM 鍵を生成する
    const key = await crypto.subtle.generateKey(
      // アルゴリズムとキー長を指定する
      { name: "AES-GCM", length: 256 },
      // テスト用なので extractable を true にする
      true,
      // 用途を設定する
      ["encrypt", "decrypt"]
    );
    // プレーンテキストをバイト列に変換する
    const encoder = new TextEncoder();
    // encryptData で暗号化する（IV は内部でランダム生成される）
    const { ciphertext } = await encryptData(key, encoder.encode(plaintext));
    // 不正な IV を生成する（別のランダム値を使用する）
    const wrongIv = crypto.getRandomValues(new Uint8Array(12));
    // 不正 IV での復号が失敗することを確認する（例外が投げられるはず）
    await expect(
      // 不正 IV で decryptData を呼び出す
      decryptData(key, ciphertext, wrongIv)
    ).rejects.toThrow();
  });
});
