import { readFileSync } from 'node:fs';
import { parse } from 'yaml';
import deepmerge from 'deepmerge';
import { ConfigSchema, type Config } from './config.js';

/**
 * YAML を読み込み Config を返す。envPath があればマージする。
 */
export function load(basePath: string, envPath?: string): Config {
  const baseContent = readFileSync(basePath, 'utf-8');
  let config = parse(baseContent);

  if (envPath) {
    const envContent = readFileSync(envPath, 'utf-8');
    const envConfig = parse(envContent);
    config = deepmerge(config, envConfig);
  }

  return config as Config;
}

/**
 * 設定値のバリデーション。Zod スキーマでパースし、不正値は例外を投げる。
 */
export function validate(config: Config): void {
  ConfigSchema.parse(config);
}
