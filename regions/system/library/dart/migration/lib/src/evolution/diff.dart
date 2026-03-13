import 'schema.dart';

sealed class SchemaDiff {}

class TableAddedDiff extends SchemaDiff {
  final Table table;
  TableAddedDiff({required this.table});
}

class TableDroppedDiff extends SchemaDiff {
  final String tableName;
  TableDroppedDiff({required this.tableName});
}

class ColumnAddedDiff extends SchemaDiff {
  final String table;
  final Column column;
  ColumnAddedDiff({required this.table, required this.column});
}

class ColumnDroppedDiff extends SchemaDiff {
  final String table;
  final String columnName;
  ColumnDroppedDiff({required this.table, required this.columnName});
}

class ColumnChangedDiff extends SchemaDiff {
  final String table;
  final String columnName;
  final Column from;
  final Column to;

  ColumnChangedDiff({
    required this.table,
    required this.columnName,
    required this.from,
    required this.to,
  });
}

/// Computes the differences between two schemas.
List<SchemaDiff> diffSchemas(Schema oldSchema, Schema newSchema) {
  final diffs = <SchemaDiff>[];

  final oldTables = {for (final t in oldSchema.tables) t.name: t};
  final newTables = {for (final t in newSchema.tables) t.name: t};

  // Detect dropped tables
  for (final entry in oldTables.entries) {
    if (!newTables.containsKey(entry.key)) {
      diffs.add(TableDroppedDiff(tableName: entry.value.name));
    }
  }

  // Detect added tables and column changes
  for (final entry in newTables.entries) {
    final oldTable = oldTables[entry.key];
    if (oldTable == null) {
      diffs.add(TableAddedDiff(table: entry.value));
    } else {
      _diffColumns(diffs, entry.key, oldTable, entry.value);
    }
  }

  return diffs;
}

void _diffColumns(
    List<SchemaDiff> diffs, String table, Table oldTable, Table newTable) {
  final oldCols = {for (final c in oldTable.columns) c.name: c};
  final newCols = {for (final c in newTable.columns) c.name: c};

  // Detect dropped columns
  for (final colName in oldCols.keys) {
    if (!newCols.containsKey(colName)) {
      diffs.add(ColumnDroppedDiff(table: table, columnName: colName));
    }
  }

  // Detect added and changed columns
  for (final entry in newCols.entries) {
    final oldCol = oldCols[entry.key];
    if (oldCol == null) {
      diffs.add(ColumnAddedDiff(table: table, column: entry.value));
    } else if (oldCol != entry.value) {
      diffs.add(ColumnChangedDiff(
        table: table,
        columnName: entry.key,
        from: oldCol,
        to: entry.value,
      ));
    }
  }
}
