import { describe, it, expect } from 'vitest';
import { parseComponentsConfig } from '../src/config.js';
import { ComponentError } from '../src/errors.js';

describe('parseComponentsConfig', () => {
  it('should parse valid YAML with multiple components', () => {
    const yaml = `
components:
  - name: redis-store
    type: statestore
    version: "1.0"
    metadata:
      host: localhost
      port: "6379"
  - name: kafka-pubsub
    type: pubsub
`;

    const config = parseComponentsConfig(yaml);
    expect(config.components).toHaveLength(2);

    expect(config.components[0].name).toBe('redis-store');
    expect(config.components[0].type).toBe('statestore');
    expect(config.components[0].version).toBe('1.0');
    expect(config.components[0].metadata).toEqual({ host: 'localhost', port: '6379' });

    expect(config.components[1].name).toBe('kafka-pubsub');
    expect(config.components[1].type).toBe('pubsub');
  });

  it('should parse a component without optional fields', () => {
    const yaml = `
components:
  - name: basic
    type: binding
`;

    const config = parseComponentsConfig(yaml);
    expect(config.components).toHaveLength(1);
    expect(config.components[0].name).toBe('basic');
    expect(config.components[0].type).toBe('binding');
  });

  it('should parse an empty components array', () => {
    const yaml = `
components: []
`;

    const config = parseComponentsConfig(yaml);
    expect(config.components).toHaveLength(0);
  });

  it('should throw ComponentError when components field is missing', () => {
    const yaml = `
name: invalid
`;

    expect(() => parseComponentsConfig(yaml)).toThrow(ComponentError);
    expect(() => parseComponentsConfig(yaml)).toThrow('components field is required and must be an array');
  });

  it('should throw ComponentError when components is not an array', () => {
    const yaml = `
components: "not-an-array"
`;

    expect(() => parseComponentsConfig(yaml)).toThrow(ComponentError);
  });

  it('should throw ComponentError for completely empty YAML', () => {
    expect(() => parseComponentsConfig('')).toThrow(ComponentError);
  });
});
