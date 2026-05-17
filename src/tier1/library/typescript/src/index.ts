/**
 * index.ts — k1s0 tier1 Library TypeScript 公開 API エントリーポイント
 * 05_鍵管理適合仕様.md / 04_認証適合仕様.md に基づく 4 言語等価強度 SDK の TypeScript 実装。
 * 生 key bytes / 生 access_token は公開 API シグネチャに一切露出しない。
 */

// KeyClass enum + KeyHandle abstract class + StubKeyHandle class を再エクスポートする
export { KeyClass, KeyHandle, StubKeyHandle } from "./keyHandle.js";

// AuthClass enum + AuthContext class を再エクスポートする
export { AuthClass, AuthContext } from "./authContext.js";

// Repository<T> interface + RlsBypassError + verifyTenantId を再エクスポートする
export { Repository, RlsBypassError, verifyTenantId } from "./repository.js";
