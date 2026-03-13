export type BreakingChange =
  | { type: 'columnDropped'; table: string; column: string }
  | { type: 'columnTypeChanged'; table: string; column: string; from: string; to: string }
  | { type: 'tableDropped'; table: string }
  | { type: 'notNullAdded'; table: string; column: string }
  | { type: 'columnRenamed'; table: string; from: string; to: string };

export function formatBreakingChange(change: BreakingChange): string {
  switch (change.type) {
    case 'columnDropped':
      return `Column ${change.table}.${change.column} dropped`;
    case 'columnTypeChanged':
      return `Column ${change.table}.${change.column} type changed from ${change.from} to ${change.to}`;
    case 'tableDropped':
      return `Table ${change.table} dropped`;
    case 'notNullAdded':
      return `NOT NULL added to ${change.table}.${change.column}`;
    case 'columnRenamed':
      return `Column ${change.table}.${change.from} renamed to ${change.to}`;
  }
}

/**
 * Detect breaking changes in SQL using regex-based parsing.
 * Returns an empty array for invalid/unrecognized SQL.
 */
export function detectBreakingChanges(sql: string): BreakingChange[] {
  const statements = sql
    .split(';')
    .map((s) => s.trim())
    .filter((s) => s.length > 0);

  const changes: BreakingChange[] = [];

  for (const stmt of statements) {
    const dropTable = stmt.match(/^DROP\s+TABLE\s+(?:IF\s+EXISTS\s+)?(\S+)/i);
    if (dropTable) {
      changes.push({ type: 'tableDropped', table: dropTable[1] });
      continue;
    }

    const dropColumn = stmt.match(/^ALTER\s+TABLE\s+(\S+)\s+DROP\s+COLUMN\s+(?:IF\s+EXISTS\s+)?(\S+)/i);
    if (dropColumn) {
      changes.push({ type: 'columnDropped', table: dropColumn[1], column: dropColumn[2] });
      continue;
    }

    const setNotNull = stmt.match(/^ALTER\s+TABLE\s+(\S+)\s+ALTER\s+COLUMN\s+(\S+)\s+SET\s+NOT\s+NULL/i);
    if (setNotNull) {
      changes.push({ type: 'notNullAdded', table: setNotNull[1], column: setNotNull[2] });
      continue;
    }

    const typeChange = stmt.match(/^ALTER\s+TABLE\s+(\S+)\s+ALTER\s+COLUMN\s+(\S+)\s+(?:SET\s+DATA\s+)?TYPE\s+(\S+)/i);
    if (typeChange) {
      changes.push({
        type: 'columnTypeChanged',
        table: typeChange[1],
        column: typeChange[2],
        from: 'unknown',
        to: typeChange[3],
      });
      continue;
    }

    const renameColumn = stmt.match(/^ALTER\s+TABLE\s+(\S+)\s+RENAME\s+COLUMN\s+(\S+)\s+TO\s+(\S+)/i);
    if (renameColumn) {
      changes.push({
        type: 'columnRenamed',
        table: renameColumn[1],
        from: renameColumn[2],
        to: renameColumn[3],
      });
      continue;
    }
  }

  return changes;
}
