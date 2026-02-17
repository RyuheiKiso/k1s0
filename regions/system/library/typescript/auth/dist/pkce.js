/**
 * PKCE (Proof Key for Code Exchange) ユーティリティ
 * RFC 7636 準拠
 */
/**
 * ランダムな code_verifier を生成する。
 * Base64url エンコードされた 32 バイトのランダム値。
 * @param getRandomValues - crypto.getRandomValues 互換の関数（テスト用に注入可能）
 */
export function generateCodeVerifier(getRandomValues = (array) => crypto.getRandomValues(array)) {
    const array = new Uint8Array(32);
    getRandomValues(array);
    return base64UrlEncode(array);
}
/**
 * code_verifier から code_challenge を計算する（S256）。
 * SHA-256 ハッシュの Base64url エンコード。
 * @param codeVerifier - PKCE code_verifier
 * @param subtleDigest - crypto.subtle.digest 互換の関数（テスト用に注入可能）
 */
export async function generateCodeChallenge(codeVerifier, subtleDigest = (algorithm, data) => crypto.subtle.digest(algorithm, data)) {
    const encoder = new TextEncoder();
    const data = encoder.encode(codeVerifier);
    const digest = await subtleDigest('SHA-256', data);
    return base64UrlEncode(new Uint8Array(digest));
}
/**
 * Uint8Array を Base64url エンコードする。
 * RFC 4648 Section 5 準拠（パディングなし）。
 */
export function base64UrlEncode(buffer) {
    let binary = '';
    for (let i = 0; i < buffer.length; i++) {
        binary += String.fromCharCode(buffer[i]);
    }
    return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}
//# sourceMappingURL=pkce.js.map