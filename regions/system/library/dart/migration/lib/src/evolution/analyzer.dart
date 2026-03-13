/// Breaking change detection for SQL migrations.

sealed class BreakingChange {
  String get description;
}

class ColumnDroppedChange extends BreakingChange {
  final String table;
  final String column;
  ColumnDroppedChange({required this.table, required this.column});

  @override
  String get description => 'Column $table.$column dropped';

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is ColumnDroppedChange &&
          table == other.table &&
          column == other.column;

  @override
  int get hashCode => Object.hash(table, column);
}

class ColumnTypeChangedChange extends BreakingChange {
  final String table;
  final String column;
  final String from;
  final String to;

  ColumnTypeChangedChange({
    required this.table,
    required this.column,
    required this.from,
    required this.to,
  });

  @override
  String get description =>
      'Column $table.$column type changed from $from to $to';

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is ColumnTypeChangedChange &&
          table == other.table &&
          column == other.column &&
          from == other.from &&
          to == other.to;

  @override
  int get hashCode => Object.hash(table, column, from, to);
}

class TableDroppedChange extends BreakingChange {
  final String table;
  TableDroppedChange({required this.table});

  @override
  String get description => 'Table $table dropped';

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is TableDroppedChange && table == other.table;

  @override
  int get hashCode => table.hashCode;
}

class NotNullAddedChange extends BreakingChange {
  final String table;
  final String column;
  NotNullAddedChange({required this.table, required this.column});

  @override
  String get description => 'NOT NULL added to $table.$column';

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is NotNullAddedChange &&
          table == other.table &&
          column == other.column;

  @override
  int get hashCode => Object.hash(table, column);
}

class ColumnRenamedChange extends BreakingChange {
  final String table;
  final String from;
  final String to;

  ColumnRenamedChange({
    required this.table,
    required this.from,
    required this.to,
  });

  @override
  String get description => 'Column $table.$from renamed to $to';

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is ColumnRenamedChange &&
          table == other.table &&
          from == other.from &&
          to == other.to;

  @override
  int get hashCode => Object.hash(table, from, to);
}

/// Detects breaking changes in SQL migration statements using regex-based parsing.
List<BreakingChange> detectBreakingChanges(String sql) {
  final trimmed = sql.trim();
  if (trimmed.isEmpty) return [];

  final statements = trimmed
      .split(';')
      .map((s) => s.trim())
      .where((s) => s.isNotEmpty)
      .toList();

  final changes = <BreakingChange>[];

  final dropTableRe = RegExp(
      r'DROP\s+TABLE\s+(?:IF\s+EXISTS\s+)?(\S+)', caseSensitive: false);
  final dropColumnRe = RegExp(
      r'ALTER\s+TABLE\s+(\S+)\s+DROP\s+COLUMN\s+(?:IF\s+EXISTS\s+)?(\S+)',
      caseSensitive: false);
  final setNotNullRe = RegExp(
      r'ALTER\s+TABLE\s+(\S+)\s+ALTER\s+COLUMN\s+(\S+)\s+SET\s+NOT\s+NULL',
      caseSensitive: false);
  final setDataTypeRe = RegExp(
      r'ALTER\s+TABLE\s+(\S+)\s+ALTER\s+COLUMN\s+(\S+)\s+(?:SET\s+DATA\s+)?TYPE\s+(\S+)',
      caseSensitive: false);
  final renameColumnRe = RegExp(
      r'ALTER\s+TABLE\s+(\S+)\s+RENAME\s+COLUMN\s+(\S+)\s+TO\s+(\S+)',
      caseSensitive: false);

  for (final stmt in statements) {
    // Check DROP TABLE
    final dropTableMatch = dropTableRe.firstMatch(stmt);
    if (dropTableMatch != null) {
      changes.add(TableDroppedChange(table: dropTableMatch.group(1)!));
      continue;
    }

    // Check DROP COLUMN
    final dropColumnMatch = dropColumnRe.firstMatch(stmt);
    if (dropColumnMatch != null) {
      changes.add(ColumnDroppedChange(
        table: dropColumnMatch.group(1)!,
        column: dropColumnMatch.group(2)!,
      ));
      continue;
    }

    // Check SET NOT NULL
    final setNotNullMatch = setNotNullRe.firstMatch(stmt);
    if (setNotNullMatch != null) {
      changes.add(NotNullAddedChange(
        table: setNotNullMatch.group(1)!,
        column: setNotNullMatch.group(2)!,
      ));
      continue;
    }

    // Check RENAME COLUMN (before SET DATA TYPE since both are ALTER TABLE ALTER COLUMN)
    final renameColumnMatch = renameColumnRe.firstMatch(stmt);
    if (renameColumnMatch != null) {
      changes.add(ColumnRenamedChange(
        table: renameColumnMatch.group(1)!,
        from: renameColumnMatch.group(2)!,
        to: renameColumnMatch.group(3)!,
      ));
      continue;
    }

    // Check SET DATA TYPE
    final setDataTypeMatch = setDataTypeRe.firstMatch(stmt);
    if (setDataTypeMatch != null) {
      changes.add(ColumnTypeChangedChange(
        table: setDataTypeMatch.group(1)!,
        column: setDataTypeMatch.group(2)!,
        from: 'unknown',
        to: setDataTypeMatch.group(3)!,
      ));
      continue;
    }
  }

  return changes;
}
