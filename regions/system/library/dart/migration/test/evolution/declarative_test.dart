import 'package:test/test.dart';
import 'package:k1s0_migration/migration.dart';

void main() {
  group('tomlToCreateSql', () {
    test('basic table', () {
      final toml = '''
[table]
name = "users"

[[table.columns]]
name = "id"
type = "UUID"
primary_key = true
nullable = false

[[table.columns]]
name = "name"
type = "TEXT"
nullable = false

[[table.columns]]
name = "email"
type = "TEXT"
nullable = true
unique = true
''';
      final sql = tomlToCreateSql(toml);
      expect(sql, contains('CREATE TABLE users'));
      expect(sql, contains('id UUID'));
      expect(sql, contains('name TEXT NOT NULL'));
      expect(sql, contains('email TEXT UNIQUE'));
      expect(sql, contains('PRIMARY KEY (id)'));
    });

    test('column with default', () {
      final toml = '''
[table]
name = "settings"

[[table.columns]]
name = "active"
type = "BOOLEAN"
nullable = false
default = "true"
''';
      final sql = tomlToCreateSql(toml);
      expect(sql, contains('active BOOLEAN NOT NULL DEFAULT true'));
    });

    test('column with references', () {
      final toml = '''
[table]
name = "orders"

[[table.columns]]
name = "id"
type = "UUID"
primary_key = true
nullable = false

[[table.columns]]
name = "user_id"
type = "UUID"
nullable = false
references = "users(id)"
''';
      final sql = tomlToCreateSql(toml);
      expect(sql, contains('user_id UUID NOT NULL REFERENCES users(id)'));
    });

    test('invalid TOML throws error', () {
      expect(
        () => tomlToCreateSql('not valid toml {{{}}}'),
        throwsA(isA<MigrationError>()),
      );
    });

    test('composite primary key', () {
      final toml = '''
[table]
name = "order_items"

[[table.columns]]
name = "order_id"
type = "UUID"
primary_key = true
nullable = false

[[table.columns]]
name = "item_id"
type = "UUID"
primary_key = true
nullable = false

[[table.columns]]
name = "quantity"
type = "INT"
nullable = false
''';
      final sql = tomlToCreateSql(toml);
      expect(sql, contains('PRIMARY KEY (order_id, item_id)'));
    });
  });
}
