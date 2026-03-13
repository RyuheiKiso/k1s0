/// Schema model classes for migration evolution.

class Column {
  final String name;
  final String dataType;
  final bool nullable;
  final String? defaultValue;

  const Column({
    required this.name,
    required this.dataType,
    this.nullable = true,
    this.defaultValue,
  });

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is Column &&
          runtimeType == other.runtimeType &&
          name == other.name &&
          dataType == other.dataType &&
          nullable == other.nullable &&
          defaultValue == other.defaultValue;

  @override
  int get hashCode => Object.hash(name, dataType, nullable, defaultValue);

  @override
  String toString() =>
      'Column(name: $name, dataType: $dataType, nullable: $nullable, defaultValue: $defaultValue)';
}

class Index {
  final String name;
  final String table;
  final List<String> columns;
  final bool unique;

  const Index({
    required this.name,
    required this.table,
    required this.columns,
    this.unique = false,
  });

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is Index &&
          runtimeType == other.runtimeType &&
          name == other.name &&
          table == other.table &&
          _listEquals(columns, other.columns) &&
          unique == other.unique;

  @override
  int get hashCode => Object.hash(name, table, Object.hashAll(columns), unique);

  @override
  String toString() =>
      'Index(name: $name, table: $table, columns: $columns, unique: $unique)';
}

sealed class Constraint {}

class PrimaryKeyConstraint extends Constraint {
  final List<String> columns;
  PrimaryKeyConstraint({required this.columns});

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is PrimaryKeyConstraint &&
          _listEquals(columns, other.columns);

  @override
  int get hashCode => Object.hashAll(columns);
}

class ForeignKeyConstraint extends Constraint {
  final List<String> columns;
  final String refTable;
  final List<String> refColumns;

  ForeignKeyConstraint({
    required this.columns,
    required this.refTable,
    required this.refColumns,
  });

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is ForeignKeyConstraint &&
          _listEquals(columns, other.columns) &&
          refTable == other.refTable &&
          _listEquals(refColumns, other.refColumns);

  @override
  int get hashCode =>
      Object.hash(Object.hashAll(columns), refTable, Object.hashAll(refColumns));
}

class UniqueConstraint extends Constraint {
  final List<String> columns;
  UniqueConstraint({required this.columns});

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is UniqueConstraint &&
          _listEquals(columns, other.columns);

  @override
  int get hashCode => Object.hashAll(columns);
}

class CheckConstraint extends Constraint {
  final String expression;
  CheckConstraint({required this.expression});

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is CheckConstraint && expression == other.expression;

  @override
  int get hashCode => expression.hashCode;
}

class Table {
  final String name;
  final List<Column> columns;
  final List<Index> indexes;
  final List<Constraint> constraints;

  const Table({
    required this.name,
    required this.columns,
    this.indexes = const [],
    this.constraints = const [],
  });

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is Table &&
          runtimeType == other.runtimeType &&
          name == other.name &&
          _listEquals(columns, other.columns);

  @override
  int get hashCode => Object.hash(name, Object.hashAll(columns));

  @override
  String toString() => 'Table(name: $name, columns: $columns)';
}

class Schema {
  final List<Table> tables;
  const Schema({required this.tables});
}

bool _listEquals<T>(List<T> a, List<T> b) {
  if (a.length != b.length) return false;
  for (var i = 0; i < a.length; i++) {
    if (a[i] != b[i]) return false;
  }
  return true;
}
