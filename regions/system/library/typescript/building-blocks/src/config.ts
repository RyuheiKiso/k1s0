import { parse } from 'yaml';
import { readFileSync } from 'node:fs';
import { ComponentError } from './errors.js';

export interface ComponentConfig {
  name: string;
  type: string;
  version?: string;
  metadata?: Record<string, string>;
}

export interface ComponentsConfig {
  components: ComponentConfig[];
}

export function loadComponentsConfig(path: string): ComponentsConfig {
  const content = readFileSync(path, 'utf-8');
  return parseComponentsConfig(content);
}

export function parseComponentsConfig(yaml_content: string): ComponentsConfig {
  const config = parse(yaml_content) as ComponentsConfig;
  if (!config.components || !Array.isArray(config.components)) {
    throw new ComponentError('config', 'parse', 'components field is required and must be an array');
  }
  return config;
}
