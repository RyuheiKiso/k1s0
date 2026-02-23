import 'package:test/test.dart';
import 'package:k1s0_migration/migration.dart';

InMemoryMigrationRunner createRunner() {
  return InMemoryMigrationRunner(
    config: const MigrationConfig(
      migrationsDir: '.',
      databaseUrl: 'memory://',
    ),
    ups: [
      (
        version: '20240101000001',
        name: 'create_users',
        content: 'CREATE TABLE users (id INT);',
      ),
      (
        version: '20240101000002',
        name: 'add_email',
        content: 'ALTER TABLE users ADD COLUMN email TEXT;',
      ),
      (
        version: '20240201000001',
        name: 'create_orders',
        content: 'CREATE TABLE orders (id INT);',
      ),
    ],
    downs: [
      (
        version: '20240101000001',
        name: 'create_users',
        content: 'DROP TABLE users;',
      ),
      (
        version: '20240101000002',
        name: 'add_email',
        content: 'ALTER TABLE users DROP COLUMN email;',
      ),
      (
        version: '20240201000001',
        name: 'create_orders',
        content: 'DROP TABLE orders;',
      ),
    ],
  );
}

void main() {
  group('parseFilename', () {
    test('parses up migration', () {
      final result =
          MigrationFile.parseFilename('20240101000001_create_users.up.sql');
      expect(result, isNotNull);
      expect(result!.version, equals('20240101000001'));
      expect(result.name, equals('create_users'));
      expect(result.direction, equals(MigrationDirection.up));
    });

    test('parses down migration', () {
      final result =
          MigrationFile.parseFilename('20240101000001_create_users.down.sql');
      expect(result, isNotNull);
      expect(result!.version, equals('20240101000001'));
      expect(result.name, equals('create_users'));
      expect(result.direction, equals(MigrationDirection.down));
    });

    test('returns null for invalid filenames', () {
      expect(MigrationFile.parseFilename('invalid.sql'), isNull);
      expect(MigrationFile.parseFilename('no_direction.sql'), isNull);
      expect(MigrationFile.parseFilename('_.up.sql'), isNull);
    });
  });

  group('checksum', () {
    test('is deterministic', () {
      const content = 'CREATE TABLE users (id SERIAL PRIMARY KEY);';
      expect(
        MigrationFile.computeChecksum(content),
        equals(MigrationFile.computeChecksum(content)),
      );
    });

    test('differs for different content', () {
      expect(
        MigrationFile.computeChecksum('CREATE TABLE users;'),
        isNot(equals(MigrationFile.computeChecksum('CREATE TABLE orders;'))),
      );
    });
  });

  group('MigrationError', () {
    test('creates with message', () {
      final err = MigrationError('test error');
      expect(err.message, equals('test error'));
    });

    test('creates connectionFailed', () {
      final err = MigrationError.connectionFailed('failed');
      expect(err.code, equals('CONNECTION_FAILED'));
    });

    test('creates checksumMismatch', () {
      final err = MigrationError.checksumMismatch('v1', 'abc', 'def');
      expect(err.code, equals('CHECKSUM_MISMATCH'));
    });

    test('creates directoryNotFound', () {
      final err = MigrationError.directoryNotFound('/tmp');
      expect(err.code, equals('DIRECTORY_NOT_FOUND'));
    });
  });

  group('MigrationConfig', () {
    test('default table name', () {
      const config = MigrationConfig(
        migrationsDir: '.',
        databaseUrl: 'memory://',
      );
      expect(config.tableName, equals('_migrations'));
    });

    test('custom table name', () {
      const config = MigrationConfig(
        migrationsDir: '.',
        databaseUrl: 'memory://',
        tableName: 'custom',
      );
      expect(config.tableName, equals('custom'));
    });
  });

  group('InMemoryMigrationRunner', () {
    late InMemoryMigrationRunner runner;

    setUp(() {
      runner = createRunner();
    });

    test('runUp applies all', () async {
      final report = await runner.runUp();
      expect(report.appliedCount, equals(3));
      expect(report.errors, isEmpty);
    });

    test('runUp is idempotent', () async {
      await runner.runUp();
      final report = await runner.runUp();
      expect(report.appliedCount, equals(0));
    });

    test('runDown rolls back one step', () async {
      await runner.runUp();
      final report = await runner.runDown(1);
      expect(report.appliedCount, equals(1));

      final p = await runner.pending();
      expect(p.length, equals(1));
      expect(p[0].version, equals('20240201000001'));
    });

    test('runDown rolls back multiple steps', () async {
      await runner.runUp();
      final report = await runner.runDown(2);
      expect(report.appliedCount, equals(2));

      final p = await runner.pending();
      expect(p.length, equals(2));
    });

    test('runDown handles more than applied', () async {
      await runner.runUp();
      final report = await runner.runDown(10);
      expect(report.appliedCount, equals(3));
    });

    test('status shows all pending initially', () async {
      final statuses = await runner.status();
      expect(statuses.length, equals(3));
      for (final s in statuses) {
        expect(s.appliedAt, isNull);
      }
    });

    test('status shows all applied after runUp', () async {
      await runner.runUp();
      final statuses = await runner.status();
      expect(statuses.length, equals(3));
      for (final s in statuses) {
        expect(s.appliedAt, isNotNull);
      }
    });

    test('pending returns all unapplied', () async {
      final p = await runner.pending();
      expect(p.length, equals(3));
      expect(p[0].version, equals('20240101000001'));
      expect(p[1].version, equals('20240101000002'));
      expect(p[2].version, equals('20240201000001'));
    });

    test('pending returns empty after full apply', () async {
      await runner.runUp();
      final p = await runner.pending();
      expect(p, isEmpty);
    });
  });
}
