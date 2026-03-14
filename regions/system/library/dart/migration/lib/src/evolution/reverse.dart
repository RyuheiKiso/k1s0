/// Generates DOWN SQL from UP SQL using regex-based parsing.
///
/// For each recognized statement, produces the inverse operation:
/// - CREATE TABLE → DROP TABLE IF EXISTS ... CASCADE
/// - CREATE INDEX → DROP INDEX IF EXISTS ...
/// - ALTER TABLE ADD COLUMN → ALTER TABLE DROP COLUMN
/// - ALTER TABLE ADD CONSTRAINT → ALTER TABLE DROP CONSTRAINT
///
/// The resulting statements are in reverse order.
String generateDownSql(String upSql) {
  final trimmed = upSql.trim();
  if (trimmed.isEmpty) return '';

  final statements = trimmed
      .split(';')
      .map((s) => s.trim())
      .where((s) => s.isNotEmpty)
      .toList();

  final downStatements = <String>[];

  final createTableRe =
      RegExp(r'CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?(\S+)', caseSensitive: false);
  final createIndexRe =
      RegExp(r'CREATE\s+(?:UNIQUE\s+)?INDEX\s+(?:IF\s+NOT\s+EXISTS\s+)?(\S+)', caseSensitive: false);
  final addColumnRe = RegExp(
      r'ALTER\s+TABLE\s+(\S+)\s+ADD\s+COLUMN\s+(\S+)', caseSensitive: false);
  final addConstraintRe = RegExp(
      r'ALTER\s+TABLE\s+(\S+)\s+ADD\s+CONSTRAINT\s+(\S+)', caseSensitive: false);

  for (final stmt in statements) {
    final createTableMatch = createTableRe.firstMatch(stmt);
    if (createTableMatch != null) {
      final tableName = createTableMatch.group(1)!;
      downStatements.add('DROP TABLE IF EXISTS $tableName CASCADE;');
      continue;
    }

    final createIndexMatch = createIndexRe.firstMatch(stmt);
    if (createIndexMatch != null) {
      final indexName = createIndexMatch.group(1)!;
      downStatements.add('DROP INDEX IF EXISTS $indexName;');
      continue;
    }

    final addColumnMatch = addColumnRe.firstMatch(stmt);
    if (addColumnMatch != null) {
      final tableName = addColumnMatch.group(1)!;
      final colName = addColumnMatch.group(2)!;
      downStatements.add('ALTER TABLE $tableName DROP COLUMN $colName;');
      continue;
    }

    final addConstraintMatch = addConstraintRe.firstMatch(stmt);
    if (addConstraintMatch != null) {
      final tableName = addConstraintMatch.group(1)!;
      final constraintName = addConstraintMatch.group(2)!;
      downStatements
          .add('ALTER TABLE $tableName DROP CONSTRAINT $constraintName;');
      continue;
    }
  }

  // Reverse order so drops happen in reverse dependency order
  return downStatements.reversed.join('\n');
}
