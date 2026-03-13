import 'package:test/test.dart';
import 'package:k1s0_migration/migration.dart';

void main() {
  group('detectBreakingChanges', () {
    test('detects DROP TABLE', () {
      final changes = detectBreakingChanges('DROP TABLE users;');
      expect(changes.length, 1);
      expect(changes[0], isA<TableDroppedChange>());
      expect((changes[0] as TableDroppedChange).table, 'users');
    });

    test('detects DROP COLUMN', () {
      final changes =
          detectBreakingChanges('ALTER TABLE users DROP COLUMN email;');
      expect(changes.length, 1);
      expect(changes[0], isA<ColumnDroppedChange>());
      final change = changes[0] as ColumnDroppedChange;
      expect(change.table, 'users');
      expect(change.column, 'email');
    });

    test('detects SET NOT NULL', () {
      final changes = detectBreakingChanges(
          'ALTER TABLE users ALTER COLUMN email SET NOT NULL;');
      expect(changes.length, 1);
      expect(changes[0], isA<NotNullAddedChange>());
      final change = changes[0] as NotNullAddedChange;
      expect(change.table, 'users');
      expect(change.column, 'email');
    });

    test('detects SET DATA TYPE', () {
      final changes = detectBreakingChanges(
          'ALTER TABLE users ALTER COLUMN age SET DATA TYPE BIGINT;');
      expect(changes.length, 1);
      expect(changes[0], isA<ColumnTypeChangedChange>());
      final change = changes[0] as ColumnTypeChangedChange;
      expect(change.table, 'users');
      expect(change.column, 'age');
      expect(change.to, 'BIGINT');
    });

    test('detects RENAME COLUMN', () {
      final changes = detectBreakingChanges(
          'ALTER TABLE users RENAME COLUMN old_name TO new_name;');
      expect(changes.length, 1);
      expect(changes[0], isA<ColumnRenamedChange>());
      final change = changes[0] as ColumnRenamedChange;
      expect(change.table, 'users');
      expect(change.from, 'old_name');
      expect(change.to, 'new_name');
    });

    test('ADD COLUMN is not a breaking change', () {
      final changes =
          detectBreakingChanges('ALTER TABLE users ADD COLUMN email TEXT;');
      expect(changes, isEmpty);
    });

    test('description formatting', () {
      final change =
          ColumnDroppedChange(table: 'users', column: 'email');
      expect(change.description, 'Column users.email dropped');
    });

    test('invalid SQL returns empty list', () {
      final changes = detectBreakingChanges('NOT VALID SQL AT ALL !!!');
      expect(changes, isEmpty);
    });
  });
}
