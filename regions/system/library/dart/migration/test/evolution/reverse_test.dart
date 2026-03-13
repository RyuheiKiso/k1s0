import 'package:test/test.dart';
import 'package:k1s0_migration/migration.dart';

void main() {
  group('generateDownSql', () {
    test('CREATE TABLE generates DROP TABLE', () {
      final up = 'CREATE TABLE users (id UUID PRIMARY KEY, name TEXT NOT NULL);';
      final down = generateDownSql(up);
      expect(down, contains('DROP TABLE IF EXISTS users CASCADE;'));
    });

    test('ADD COLUMN generates DROP COLUMN', () {
      final up = 'ALTER TABLE users ADD COLUMN email TEXT;';
      final down = generateDownSql(up);
      expect(down, contains('ALTER TABLE users DROP COLUMN email;'));
    });

    test('CREATE INDEX generates DROP INDEX', () {
      final up = 'CREATE INDEX idx_users_name ON users (name);';
      final down = generateDownSql(up);
      expect(down, contains('DROP INDEX IF EXISTS idx_users_name;'));
    });

    test('CREATE UNIQUE INDEX generates DROP INDEX', () {
      final up = 'CREATE UNIQUE INDEX idx_users_email ON users (email);';
      final down = generateDownSql(up);
      expect(down, contains('DROP INDEX IF EXISTS idx_users_email;'));
    });

    test('multiple statements are reversed', () {
      final up =
          'CREATE TABLE users (id UUID PRIMARY KEY);\nCREATE INDEX idx_users_id ON users (id);';
      final down = generateDownSql(up);
      final lines = down.split('\n');
      expect(lines.length, 2);
      expect(lines[0], contains('DROP INDEX'));
      expect(lines[1], contains('DROP TABLE'));
    });

    test('empty SQL returns empty string', () {
      final down = generateDownSql('');
      expect(down, isEmpty);
    });
  });
}
