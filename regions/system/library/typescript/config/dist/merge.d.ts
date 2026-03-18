import { type Config } from './config.js';
/**
 * YAML を読み込み Config を返す。envPath があればマージする。
 * Zod スキーマによるバリデーションを実行し、不正な設定値は早期に検出する。
 */
export declare function load(basePath: string, envPath?: string): Config;
/**
 * 設定値のバリデーション。Zod スキーマでパースし、不正値は例外を投げる。
 */
export declare function validate(config: Config): void;
//# sourceMappingURL=merge.d.ts.map