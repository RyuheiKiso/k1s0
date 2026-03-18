import { readFileSync } from 'node:fs';
import { parse } from 'yaml';
import deepmerge from 'deepmerge';
import { ConfigSchema, type Config } from './config.js';

/**
 * YAML を読み込み Config を返す。envPath があればマージする。
 * Zod スキーマによるバリデーションを実行し、不正な設定値は早期に検出する。
 */
export function load(basePath: string, envPath?: string): Config {
  // ベース設定ファイルを読み込む
  const baseContent = readFileSync(basePath, 'utf-8');
  let config = parse(baseContent);

  // 環境別設定がある場合はディープマージする
  if (envPath) {
    const envContent = readFileSync(envPath, 'utf-8');
    const envConfig = parse(envContent);
    config = deepmerge(config, envConfig);
  }

  // Zod スキーマでバリデーションし、型安全な Config を返す
  return ConfigSchema.parse(config);
}

/**
 * 設定値のバリデーション。Zod スキーマでパースし、不正値は例外を投げる。
 */
export function validate(config: Config): void {
  ConfigSchema.parse(config);
}
