import 'package:toml/toml.dart';
import '../error.dart';

/// Converts a TOML table definition to CREATE TABLE SQL.
///
/// Expected TOML format:
/// ```toml
/// [table]
/// name = "users"
///
/// [[table.columns]]
/// name = "id"
/// type = "UUID"
/// primary_key = true
/// nullable = false
/// ```
String tomlToCreateSql(String tomlStr) {
  final Map<String, dynamic> doc;
  try {
    doc = TomlDocument.parse(tomlStr).toMap();
  } catch (e) {
    throw MigrationError('TOML parse error: $e', code: 'PARSE_ERROR');
  }

  final table = doc['table'] as Map<String, dynamic>?;
  if (table == null) {
    throw MigrationError('Missing [table] section', code: 'PARSE_ERROR');
  }

  final tableName = table['name'] as String?;
  if (tableName == null) {
    throw MigrationError('Missing table.name', code: 'PARSE_ERROR');
  }

  final columns = table['columns'] as List<dynamic>?;
  if (columns == null || columns.isEmpty) {
    throw MigrationError('Missing table.columns', code: 'PARSE_ERROR');
  }

  final colDefs = <String>[];
  final primaryKeys = <String>[];

  for (final col in columns) {
    final colMap = col as Map<String, dynamic>;
    final colName = colMap['name'] as String;
    final colType = colMap['type'] as String;
    final primaryKey = colMap['primary_key'] as bool? ?? false;
    final nullable = colMap['nullable'] as bool? ?? true;
    final unique = colMap['unique'] as bool? ?? false;
    final defaultValue = colMap['default'] as String?;
    final references = colMap['references'] as String?;

    final parts = <String>[
      '$colName $colType',
    ];

    if (!nullable && !primaryKey) {
      parts.add('NOT NULL');
    }

    if (unique) {
      parts.add('UNIQUE');
    }

    if (defaultValue != null) {
      parts.add('DEFAULT $defaultValue');
    }

    if (references != null) {
      parts.add('REFERENCES $references');
    }

    if (primaryKey) {
      primaryKeys.add(colName);
    }

    colDefs.add(parts.join(' '));
  }

  if (primaryKeys.isNotEmpty) {
    colDefs.add('PRIMARY KEY (${primaryKeys.join(', ')})');
  }

  return 'CREATE TABLE $tableName (\n  ${colDefs.join(',\n  ')}\n);';
}
