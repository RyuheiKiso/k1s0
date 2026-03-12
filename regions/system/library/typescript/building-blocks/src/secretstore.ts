import type { Component } from './component.js';

export interface SecretValue {
  key: string;
  value: string;
  metadata: Record<string, string>;
}

export interface SecretStore extends Component {
  getSecret(key: string): Promise<SecretValue>;
  bulkGet(keys: string[]): Promise<Record<string, SecretValue>>;
}
