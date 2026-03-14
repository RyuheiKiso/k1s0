import { parse } from 'yaml';
import { readFileSync } from 'node:fs';
import { ComponentError } from './errors.js';

// 単一コンポーネントの設定を表すインターフェース。name と type は必須、version とメタデータは任意。
export interface ComponentConfig {
  name: string;
  type: string;
  version?: string;
  metadata?: Record<string, string>;
}

// 複数コンポーネントの設定をまとめるルートインターフェース。YAMLファイルのトップレベル構造に対応する。
export interface ComponentsConfig {
  components: ComponentConfig[];
}

// 指定パスのYAMLファイルを読み込み、コンポーネント設定としてパースして返す。
export function loadComponentsConfig(path: string): ComponentsConfig {
  const content = readFileSync(path, 'utf-8');
  return parseComponentsConfig(content);
}

// YAML文字列をパースしてコンポーネント設定を返す。components フィールドが配列でない場合は ComponentError をスローする。
export function parseComponentsConfig(yaml_content: string): ComponentsConfig {
  const config = parse(yaml_content) as ComponentsConfig | null;
  if (!config || !config.components || !Array.isArray(config.components)) {
    throw new ComponentError('config', 'parse', 'components field is required and must be an array');
  }
  return config;
}
