// k1s0 tier3 DPoP (Demonstration of Proof-of-Possession) 実装
// RFC 9449 準拠: ECDSA P-256 non-extractable key pair を使い DPoP JWT を生成する
// BFF auth-edge が DPoP bound access_token を要求する場合に使用する
// key は non-extractable → private key が JS 外部に露出しないことを保証する

// DPoP JWT のヘッダー typ 固定値（RFC 9449 Section 4.2）
const DPOP_JWT_TYPE = "dpop+jwt";
// ECDSA P-256 アルゴリズム名
const EC_ALGORITHM = "ECDSA";
// P-256 名前付き曲線
const EC_NAMED_CURVE = "P-256";
// ECDSA 署名ハッシュアルゴリズム（SHA-256）
const SIGN_HASH = "SHA-256";
// DPoP JWT の有効期間（60 秒）
const DPOP_PROOF_TTL_SEC = 60;

// DPoP 用 ECDSA P-256 キーペアを生成する（non-extractable）
// private key は exportKey できない設計 → JS 外部に漏れない
export async function generateDpopKey(): Promise<CryptoKeyPair> {
  // ECDSA P-256 non-extractable キーペアを生成する
  return crypto.subtle.generateKey(
    {
      // アルゴリズムを ECDSA に設定する
      name: EC_ALGORITHM,
      // P-256 曲線を使用する
      namedCurve: EC_NAMED_CURVE,
    },
    // extractable: false → private key の exportKey を禁止する
    false,
    // sign / verify 用途のみ許可する
    ["sign", "verify"],
  );
}

// Base64url エンコードヘルパー（padding なし）
function base64url(buffer: ArrayBuffer): string {
  // バイト列を Base64 に変換する
  const b64 = btoa(String.fromCharCode(...new Uint8Array(buffer)));
  // Base64url に変換する（+ → - / / → _ / padding = を除去する）
  return b64.replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

// CryptoKey（ECDSA public key）を JWK 形式に変換するヘルパー
async function exportPublicKeyAsJwk(publicKey: CryptoKey): Promise<JsonWebKey> {
  // public key は extractable=true で生成されるため exportKey 可能
  return crypto.subtle.exportKey("jwk", publicKey);
}

// JSON オブジェクトを Base64url エンコードするヘルパー
function jsonToBase64url(obj: object): string {
  // JSON 文字列をバイト列に変換する
  const json = JSON.stringify(obj);
  // UTF-8 バイト列に変換する
  const bytes = new TextEncoder().encode(json);
  // btoa を使って Base64 変換する（Uint8Array を spread する）
  const b64 = btoa(String.fromCharCode(...bytes));
  // Base64url に変換する
  return b64.replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

// DPoP proof JWT を生成する（RFC 9449 Section 4）
// htm: HTTP method（例: "POST"）
// htu: HTTP target URI（例: "https://api.example.com/resource"）
// jti は crypto.randomUUID() で生成して replay 防止を担保する
export async function createDpopProof(
  keyPair: CryptoKeyPair,
  htm: string,
  htu: string,
): Promise<string> {
  // public key を JWK として取得する（DPoP ヘッダーの jwk に埋め込む）
  const jwk = await exportPublicKeyAsJwk(keyPair.publicKey);
  // JWK から不要なフィールドを除去する（key_ops / ext はヘッダーに不要）
  const { key_ops: _ko, ext: _ext, ...cleanedJwk } = jwk;
  // DPoP JWT ヘッダーを構築する（RFC 9449 Section 4.2）
  const header = {
    // typ は dpop+jwt 固定（RFC 9449 必須）
    typ: DPOP_JWT_TYPE,
    // alg は ES256（ECDSA P-256 + SHA-256）
    alg: "ES256",
    // jwk に public key を埋め込む（private key は含まない）
    jwk: cleanedJwk,
  };
  // DPoP JWT ペイロードを構築する（RFC 9449 Section 4.2）
  const nowSec = Math.floor(Date.now() / 1000);
  // jti: UUID v4 で replay 防止する（crypto.randomUUID() を使う）
  const jti: string =
    typeof crypto.randomUUID === "function"
      ? crypto.randomUUID()
      : `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
  const payload = {
    // jti: 一意の識別子（replay 防止）
    jti,
    // htm: HTTP メソッド（大文字）
    htm: htm.toUpperCase(),
    // htu: HTTP ターゲット URI
    htu,
    // iat: 発行日時（秒単位 UNIX タイムスタンプ）
    iat: nowSec,
    // exp: 有効期限（発行から 60 秒後）
    exp: nowSec + DPOP_PROOF_TTL_SEC,
  };
  // ヘッダーとペイロードを Base64url エンコードして連結する
  const headerB64 = jsonToBase64url(header);
  const payloadB64 = jsonToBase64url(payload);
  // 署名対象文字列（header.payload）を UTF-8 バイト列に変換する
  const signingInput = `${headerB64}.${payloadB64}`;
  const signingBytes = new TextEncoder().encode(signingInput);
  // ECDSA P-256 + SHA-256 で署名する（non-extractable private key を使う）
  const signature = await crypto.subtle.sign(
    { name: EC_ALGORITHM, hash: SIGN_HASH },
    // private key で署名する（non-extractable のため JS 外部に露出しない）
    keyPair.privateKey,
    signingBytes,
  );
  // 署名を Base64url エンコードする
  const signatureB64 = base64url(signature);
  // DPoP proof JWT を返す（header.payload.signature 形式）
  return `${headerB64}.${payloadB64}.${signatureB64}`;
}
