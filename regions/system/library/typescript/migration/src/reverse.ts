/**
 * Generate DOWN SQL from UP SQL using regex-based parsing.
 * Splits on semicolons, matches known DDL patterns, and reverses output order.
 */
export function generateDownSql(upSql: string): string {
  const statements = upSql
    .split(';')
    .map((s) => s.trim())
    .filter((s) => s.length > 0);

  const downStatements: string[] = [];

  for (const stmt of statements) {
    const createTable = stmt.match(/^CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?(\S+)/i);
    if (createTable) {
      downStatements.push(`DROP TABLE IF EXISTS ${createTable[1]} CASCADE;`);
      continue;
    }

    const createIndex = stmt.match(/^CREATE\s+(?:UNIQUE\s+)?INDEX\s+(?:IF\s+NOT\s+EXISTS\s+)?(\S+)/i);
    if (createIndex) {
      downStatements.push(`DROP INDEX IF EXISTS ${createIndex[1]};`);
      continue;
    }

    const addColumn = stmt.match(/^ALTER\s+TABLE\s+(\S+)\s+ADD\s+COLUMN\s+(\S+)/i);
    if (addColumn) {
      downStatements.push(`ALTER TABLE ${addColumn[1]} DROP COLUMN ${addColumn[2]};`);
      continue;
    }

    const addConstraint = stmt.match(/^ALTER\s+TABLE\s+(\S+)\s+ADD\s+CONSTRAINT\s+(\S+)/i);
    if (addConstraint) {
      downStatements.push(`ALTER TABLE ${addConstraint[1]} DROP CONSTRAINT ${addConstraint[2]};`);
      continue;
    }
  }

  downStatements.reverse();
  return downStatements.join('\n');
}
