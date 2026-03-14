import type { Schema, Table, Column } from './schema.js';

export type SchemaDiff =
  | { type: 'tableAdded'; table: Table }
  | { type: 'tableDropped'; tableName: string }
  | { type: 'columnAdded'; table: string; column: Column }
  | { type: 'columnDropped'; table: string; columnName: string }
  | { type: 'columnChanged'; table: string; columnName: string; from: Column; to: Column };

export function diffSchemas(oldSchema: Schema, newSchema: Schema): SchemaDiff[] {
  const diffs: SchemaDiff[] = [];

  const oldTables = new Map(oldSchema.tables.map((t) => [t.name, t]));
  const newTables = new Map(newSchema.tables.map((t) => [t.name, t]));

  // Detect dropped tables
  for (const [name] of oldTables) {
    if (!newTables.has(name)) {
      diffs.push({ type: 'tableDropped', tableName: name });
    }
  }

  // Detect added tables and column changes
  for (const [name, newTable] of newTables) {
    const oldTable = oldTables.get(name);
    if (!oldTable) {
      diffs.push({ type: 'tableAdded', table: newTable });
    } else {
      diffColumns(diffs, name, oldTable, newTable);
    }
  }

  return diffs;
}

function diffColumns(
  diffs: SchemaDiff[],
  table: string,
  oldTable: Table,
  newTable: Table,
): void {
  const oldCols = new Map(oldTable.columns.map((c) => [c.name, c]));
  const newCols = new Map(newTable.columns.map((c) => [c.name, c]));

  for (const [colName] of oldCols) {
    if (!newCols.has(colName)) {
      diffs.push({ type: 'columnDropped', table, columnName: colName });
    }
  }

  for (const [colName, newCol] of newCols) {
    const oldCol = oldCols.get(colName);
    if (!oldCol) {
      diffs.push({ type: 'columnAdded', table, column: newCol });
    } else if (
      oldCol.dataType !== newCol.dataType ||
      oldCol.nullable !== newCol.nullable ||
      oldCol.default !== newCol.default
    ) {
      diffs.push({
        type: 'columnChanged',
        table,
        columnName: colName,
        from: oldCol,
        to: newCol,
      });
    }
  }
}
