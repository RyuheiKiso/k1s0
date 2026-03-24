import { parse } from 'smol-toml';
import { MigrationError } from './index.js';

interface ColumnSpec {
  name: string;
  type: string;
  primary_key?: boolean;
  nullable?: boolean;
  default?: string;
  unique?: boolean;
  references?: string;
}

interface TableSpec {
  name: string;
  columns: ColumnSpec[];
}

interface TableDef {
  table: TableSpec;
}

/**
 * Convert a TOML table definition to CREATE TABLE SQL.
 */
export function tomlToCreateSql(tomlStr: string): string {
  let def: TableDef;
  try {
    def = parse(tomlStr) as unknown as TableDef;
  } catch (e: unknown) {
    throw new MigrationError(
      `TOML parse error: ${e instanceof Error ? e.message : String(e)}`,
    );
  }

  if (!def.table || !def.table.name || !Array.isArray(def.table.columns)) {
    throw new MigrationError('TOML parse error: missing table definition');
  }

  const colDefs: string[] = [];
  const primaryKeys: string[] = [];

  for (const col of def.table.columns) {
    const parts: string[] = [`${col.name} ${col.type}`];

    const nullable = col.nullable ?? true;

    if (!nullable && !col.primary_key) {
      parts.push('NOT NULL');
    }

    if (col.unique) {
      parts.push('UNIQUE');
    }

    if (col.default !== undefined) {
      parts.push(`DEFAULT ${col.default}`);
    }

    if (col.references) {
      parts.push(`REFERENCES ${col.references}`);
    }

    if (col.primary_key) {
      primaryKeys.push(col.name);
    }

    colDefs.push(parts.join(' '));
  }

  if (primaryKeys.length > 0) {
    colDefs.push(`PRIMARY KEY (${primaryKeys.join(', ')})`);
  }

  return `CREATE TABLE ${def.table.name} (\n  ${colDefs.join(',\n  ')}\n);`;
}
