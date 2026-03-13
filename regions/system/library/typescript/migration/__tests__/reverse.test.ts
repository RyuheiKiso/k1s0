import { describe, it, expect } from 'vitest';
import { generateDownSql } from '../src/reverse.js';

describe('generateDownSql', () => {
  it('should generate DROP TABLE from CREATE TABLE', () => {
    const up = 'CREATE TABLE users (id UUID PRIMARY KEY, name TEXT NOT NULL);';
    const down = generateDownSql(up);
    expect(down).toContain('DROP TABLE IF EXISTS users CASCADE;');
  });

  it('should generate DROP COLUMN from ADD COLUMN', () => {
    const up = 'ALTER TABLE users ADD COLUMN email TEXT;';
    const down = generateDownSql(up);
    expect(down).toContain('ALTER TABLE users DROP COLUMN email;');
  });

  it('should generate DROP INDEX from CREATE INDEX', () => {
    const up = 'CREATE INDEX idx_users_name ON users (name);';
    const down = generateDownSql(up);
    expect(down).toContain('DROP INDEX IF EXISTS idx_users_name;');
  });

  it('should generate DROP INDEX from CREATE UNIQUE INDEX', () => {
    const up = 'CREATE UNIQUE INDEX idx_users_email ON users (email);';
    const down = generateDownSql(up);
    expect(down).toContain('DROP INDEX IF EXISTS idx_users_email;');
  });

  it('should reverse order of multiple statements', () => {
    const up =
      'CREATE TABLE users (id UUID PRIMARY KEY);\nCREATE INDEX idx_users_id ON users (id);';
    const down = generateDownSql(up);
    const lines = down.split('\n');
    expect(lines).toHaveLength(2);
    expect(lines[0]).toContain('DROP INDEX');
    expect(lines[1]).toContain('DROP TABLE');
  });

  it('should return empty string for empty SQL', () => {
    const down = generateDownSql('');
    expect(down).toBe('');
  });
});
