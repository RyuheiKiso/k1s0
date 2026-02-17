/**
 * PKCE (Proof Key for Code Exchange) ユーティリティ
 * RFC 7636 準拠
 */
/**
 * ランダムな code_verifier を生成する。
 * Base64url エンコードされた 32 バイトのランダム値。
 * @param getRandomValues - crypto.getRandomValues 互換の関数（テスト用に注入可能）
 */
export declare function generateCodeVerifier(getRandomValues?: (array: Uint8Array) => Uint8Array): string;
/**
 * code_verifier から code_challenge を計算する（S256）。
 * SHA-256 ハッシュの Base64url エンコード。
 * @param codeVerifier - PKCE code_verifier
 * @param subtleDigest - crypto.subtle.digest 互換の関数（テスト用に注入可能）
 */
export declare function generateCodeChallenge(codeVerifier: string, subtleDigest?: (algorithm: string, data: BufferSource) => Promise<ArrayBuffer>): Promise<string>;
/**
 * Uint8Array を Base64url エンコードする。
 * RFC 4648 Section 5 準拠（パディングなし）。
 */
export declare function base64UrlEncode(buffer: Uint8Array): string;
//# sourceMappingURL=pkce.d.ts.map