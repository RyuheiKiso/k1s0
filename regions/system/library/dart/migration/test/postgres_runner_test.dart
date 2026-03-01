import 'package:test/test.dart';
import 'package:k1s0_migration/migration.dart';

// Integration tests for PostgresMigrationRunner require a live database.
// These tests verify the MigrationRunner contract via InMemoryMigrationRunner.

void main() {
  final config = const MigrationConfig(
    migrationsDir: 'migrations',
    databaseUrl: 'postgres://localhost:5432/test',
  );

  group('PostgresMigrationRunner contract (via InMemory)', () {
    late InMemoryMigrationRunner runner;

    setUp(() {
      runner = InMemoryMigrationRunner(
        config: config,
        ups: [
          (
            version: '001',
            name: 'create_users',
            content: 'CREATE TABLE users (id UUID PRIMARY KEY);',
          ),
          (
            version: '002',
            name: 'add_email',
            content: 'ALTER TABLE users ADD COLUMN email TEXT;',
          ),
        ],
        downs: [
          (
            version: '002',
            name: 'add_email',
            content: 'ALTER TABLE users DROP COLUMN email;',
          ),
          (
            version: '001',
            name: 'create_users',
            content: 'DROP TABLE users;',
          ),
        ],
      );
    });

    test('runUp applies all pending migrations', () async {
      final report = await runner.runUp();
      expect(report.appliedCount, equals(2));
      expect(report.errors, isEmpty);
    });

    test('runUp is idempotent', () async {
      await runner.runUp();
      final report2 = await runner.runUp();
      expect(report2.appliedCount, equals(0));
    });

    test('pending returns unapplied migrations', () async {
      final pending = await runner.pending();
      expect(pending, hasLength(2));
      expect(pending.first.version, equals('001'));
    });

    test('pending returns empty after runUp', () async {
      await runner.runUp();
      final pending = await runner.pending();
      expect(pending, isEmpty);
    });

    test('status returns all migration statuses', () async {
      await runner.runUp();
      final statuses = await runner.status();
      expect(statuses, hasLength(2));
      expect(statuses[0].version, equals('001'));
      expect(statuses[0].appliedAt, isNotNull);
    });

    test('runDown rolls back specified steps', () async {
      await runner.runUp();
      final report = await runner.runDown(1);
      expect(report.appliedCount, equals(1));
      final pending = await runner.pending();
      expect(pending, hasLength(1));
      expect(pending.first.version, equals('002'));
    });

    test('runDown with more steps than applied stops gracefully', () async {
      await runner.runUp();
      final report = await runner.runDown(10);
      expect(report.appliedCount, equals(2));
    });
  });

  group('MigrationFile.parseFilename', () {
    test('parses up migration filename', () {
      final result = MigrationFile.parseFilename('001_create_users.up.sql');
      expect(result, isNotNull);
      expect(result!.version, equals('001'));
      expect(result.name, equals('create_users'));
      expect(result.direction, equals(MigrationDirection.up));
    });

    test('parses down migration filename', () {
      final result = MigrationFile.parseFilename('002_add_email.down.sql');
      expect(result, isNotNull);
      expect(result!.version, equals('002'));
      expect(result.name, equals('add_email'));
      expect(result.direction, equals(MigrationDirection.down));
    });

    test('returns null for invalid filename', () {
      expect(MigrationFile.parseFilename('not_a_migration.txt'), isNull);
      expect(MigrationFile.parseFilename('noversion.up.sql'), isNull);
    });
  });

  group('MigrationFile.computeChecksum', () {
    test('returns consistent checksum', () {
      const content = 'CREATE TABLE test (id UUID);';
      final cs1 = MigrationFile.computeChecksum(content);
      final cs2 = MigrationFile.computeChecksum(content);
      expect(cs1, equals(cs2));
    });

    test('returns different checksum for different content', () {
      final cs1 = MigrationFile.computeChecksum('SELECT 1');
      final cs2 = MigrationFile.computeChecksum('SELECT 2');
      expect(cs1, isNot(equals(cs2)));
    });
  });
}
