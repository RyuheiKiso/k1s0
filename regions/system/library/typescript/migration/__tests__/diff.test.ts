import { describe, it, expect } from 'vitest';
import { diffSchemas } from '../src/diff.js';
import type { Schema, Column, Table } from '../src/schema.js';

function makeColumn(
  name: string,
  dataType: string,
  nullable: boolean,
): Column {
  return { name, dataType, nullable };
}

function makeTable(name: string, columns: Column[]): Table {
  return { name, columns, indexes: [], constraints: [] };
}

describe('diffSchemas', () => {
  it('should detect table added', () => {
    const oldSchema: Schema = { tables: [] };
    const newSchema: Schema = {
      tables: [makeTable('users', [makeColumn('id', 'UUID', false)])],
    };
    const diffs = diffSchemas(oldSchema, newSchema);
    expect(diffs).toHaveLength(1);
    expect(diffs[0].type).toBe('tableAdded');
    if (diffs[0].type === 'tableAdded') {
      expect(diffs[0].table.name).toBe('users');
    }
  });

  it('should detect table dropped', () => {
    const oldSchema: Schema = {
      tables: [makeTable('users', [makeColumn('id', 'UUID', false)])],
    };
    const newSchema: Schema = { tables: [] };
    const diffs = diffSchemas(oldSchema, newSchema);
    expect(diffs).toHaveLength(1);
    expect(diffs[0].type).toBe('tableDropped');
    if (diffs[0].type === 'tableDropped') {
      expect(diffs[0].tableName).toBe('users');
    }
  });

  it('should detect column added', () => {
    const oldSchema: Schema = {
      tables: [makeTable('users', [makeColumn('id', 'UUID', false)])],
    };
    const newSchema: Schema = {
      tables: [
        makeTable('users', [
          makeColumn('id', 'UUID', false),
          makeColumn('email', 'TEXT', true),
        ]),
      ],
    };
    const diffs = diffSchemas(oldSchema, newSchema);
    expect(diffs).toHaveLength(1);
    expect(diffs[0].type).toBe('columnAdded');
    if (diffs[0].type === 'columnAdded') {
      expect(diffs[0].table).toBe('users');
      expect(diffs[0].column.name).toBe('email');
    }
  });

  it('should detect column dropped', () => {
    const oldSchema: Schema = {
      tables: [
        makeTable('users', [
          makeColumn('id', 'UUID', false),
          makeColumn('email', 'TEXT', true),
        ]),
      ],
    };
    const newSchema: Schema = {
      tables: [makeTable('users', [makeColumn('id', 'UUID', false)])],
    };
    const diffs = diffSchemas(oldSchema, newSchema);
    expect(diffs).toHaveLength(1);
    expect(diffs[0].type).toBe('columnDropped');
    if (diffs[0].type === 'columnDropped') {
      expect(diffs[0].table).toBe('users');
      expect(diffs[0].columnName).toBe('email');
    }
  });

  it('should detect column changed', () => {
    const oldSchema: Schema = {
      tables: [makeTable('users', [makeColumn('name', 'TEXT', true)])],
    };
    const newSchema: Schema = {
      tables: [makeTable('users', [makeColumn('name', 'VARCHAR', false)])],
    };
    const diffs = diffSchemas(oldSchema, newSchema);
    expect(diffs).toHaveLength(1);
    expect(diffs[0].type).toBe('columnChanged');
    if (diffs[0].type === 'columnChanged') {
      expect(diffs[0].table).toBe('users');
      expect(diffs[0].columnName).toBe('name');
      expect(diffs[0].from.dataType).toBe('TEXT');
      expect(diffs[0].to.dataType).toBe('VARCHAR');
    }
  });

  it('should return empty array when no changes', () => {
    const schema: Schema = {
      tables: [makeTable('users', [makeColumn('id', 'UUID', false)])],
    };
    const diffs = diffSchemas(schema, schema);
    expect(diffs).toHaveLength(0);
  });
});
