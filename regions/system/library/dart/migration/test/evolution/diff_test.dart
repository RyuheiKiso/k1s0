import 'package:test/test.dart';
import 'package:k1s0_migration/migration.dart';

Column makeColumn(String name, String dataType, bool nullable) {
  return Column(name: name, dataType: dataType, nullable: nullable);
}

Table makeTable(String name, List<Column> columns) {
  return Table(name: name, columns: columns);
}

void main() {
  group('diffSchemas', () {
    test('テーブルの追加が検出されること', () {
      final oldSchema = Schema(tables: []);
      final newSchema = Schema(tables: [
        makeTable('users', [makeColumn('id', 'UUID', false)]),
      ]);
      final diffs = diffSchemas(oldSchema, newSchema);
      expect(diffs.length, 1);
      expect(diffs[0], isA<TableAddedDiff>());
      expect((diffs[0] as TableAddedDiff).table.name, 'users');
    });

    test('テーブルの削除が検出されること', () {
      final oldSchema = Schema(tables: [
        makeTable('users', [makeColumn('id', 'UUID', false)]),
      ]);
      final newSchema = Schema(tables: []);
      final diffs = diffSchemas(oldSchema, newSchema);
      expect(diffs.length, 1);
      expect(diffs[0], isA<TableDroppedDiff>());
      expect((diffs[0] as TableDroppedDiff).tableName, 'users');
    });

    test('カラムの追加が検出されること', () {
      final oldSchema = Schema(tables: [
        makeTable('users', [makeColumn('id', 'UUID', false)]),
      ]);
      final newSchema = Schema(tables: [
        makeTable('users', [
          makeColumn('id', 'UUID', false),
          makeColumn('email', 'TEXT', true),
        ]),
      ]);
      final diffs = diffSchemas(oldSchema, newSchema);
      expect(diffs.length, 1);
      expect(diffs[0], isA<ColumnAddedDiff>());
      final diff = diffs[0] as ColumnAddedDiff;
      expect(diff.table, 'users');
      expect(diff.column.name, 'email');
    });

    test('カラムの削除が検出されること', () {
      final oldSchema = Schema(tables: [
        makeTable('users', [
          makeColumn('id', 'UUID', false),
          makeColumn('email', 'TEXT', true),
        ]),
      ]);
      final newSchema = Schema(tables: [
        makeTable('users', [makeColumn('id', 'UUID', false)]),
      ]);
      final diffs = diffSchemas(oldSchema, newSchema);
      expect(diffs.length, 1);
      expect(diffs[0], isA<ColumnDroppedDiff>());
      final diff = diffs[0] as ColumnDroppedDiff;
      expect(diff.table, 'users');
      expect(diff.columnName, 'email');
    });

    test('カラムの変更が検出されること', () {
      final oldSchema = Schema(tables: [
        makeTable('users', [makeColumn('name', 'TEXT', true)]),
      ]);
      final newSchema = Schema(tables: [
        makeTable('users', [makeColumn('name', 'VARCHAR', false)]),
      ]);
      final diffs = diffSchemas(oldSchema, newSchema);
      expect(diffs.length, 1);
      expect(diffs[0], isA<ColumnChangedDiff>());
      final diff = diffs[0] as ColumnChangedDiff;
      expect(diff.table, 'users');
      expect(diff.columnName, 'name');
      expect(diff.from.dataType, 'TEXT');
      expect(diff.to.dataType, 'VARCHAR');
    });

    test('同一スキーマの場合に差分がないこと', () {
      final schema = Schema(tables: [
        makeTable('users', [makeColumn('id', 'UUID', false)]),
      ]);
      final diffs = diffSchemas(schema, schema);
      expect(diffs, isEmpty);
    });
  });
}
