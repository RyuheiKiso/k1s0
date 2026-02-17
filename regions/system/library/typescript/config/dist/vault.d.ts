import type { Config } from './config.js';
/**
 * Vault から取得したシークレットで設定値を上書きする。
 * 元の Config を変更せず、新しい Config を返す。
 */
export declare function mergeVaultSecrets(config: Config, secrets: Record<string, string>): Config;
//# sourceMappingURL=vault.d.ts.map