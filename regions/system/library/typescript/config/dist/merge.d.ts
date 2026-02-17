import { type Config } from './config.js';
/**
 * YAML を読み込み Config を返す。envPath があればマージする。
 */
export declare function load(basePath: string, envPath?: string): Config;
/**
 * 設定値のバリデーション。Zod スキーマでパースし、不正値は例外を投げる。
 */
export declare function validate(config: Config): void;
//# sourceMappingURL=merge.d.ts.map