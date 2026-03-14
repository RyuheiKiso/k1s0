import { describe, it, expect } from 'vitest';
import { tomlToCreateSql } from '../src/declarative.js';

describe('tomlToCreateSql', () => {
  it('should generate SQL for a basic table', () => {
    const toml = `
[table]
name = "users"

[[table.columns]]
name = "id"
type = "UUID"
primary_key = true
nullable = false

[[table.columns]]
name = "name"
type = "TEXT"
nullable = false

[[table.columns]]
name = "email"
type = "TEXT"
nullable = true
unique = true
`;
    const sql = tomlToCreateSql(toml);
    expect(sql).toContain('CREATE TABLE users');
    expect(sql).toContain('id UUID');
    expect(sql).toContain('name TEXT NOT NULL');
    expect(sql).toContain('email TEXT UNIQUE');
    expect(sql).toContain('PRIMARY KEY (id)');
  });

  it('should handle column with default value', () => {
    const toml = `
[table]
name = "settings"

[[table.columns]]
name = "active"
type = "BOOLEAN"
nullable = false
default = "true"
`;
    const sql = tomlToCreateSql(toml);
    expect(sql).toContain('active BOOLEAN NOT NULL DEFAULT true');
  });

  it('should handle column with references', () => {
    const toml = `
[table]
name = "orders"

[[table.columns]]
name = "id"
type = "UUID"
primary_key = true
nullable = false

[[table.columns]]
name = "user_id"
type = "UUID"
nullable = false
references = "users(id)"
`;
    const sql = tomlToCreateSql(toml);
    expect(sql).toContain('user_id UUID NOT NULL REFERENCES users(id)');
  });

  it('should throw MigrationError for invalid TOML', () => {
    expect(() => tomlToCreateSql('not valid toml {{{}}}'))
      .toThrow('TOML parse error');
  });

  it('should handle composite primary key', () => {
    const toml = `
[table]
name = "order_items"

[[table.columns]]
name = "order_id"
type = "UUID"
primary_key = true
nullable = false

[[table.columns]]
name = "item_id"
type = "UUID"
primary_key = true
nullable = false

[[table.columns]]
name = "quantity"
type = "INT"
nullable = false
`;
    const sql = tomlToCreateSql(toml);
    expect(sql).toContain('PRIMARY KEY (order_id, item_id)');
  });
});
