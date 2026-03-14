import 'package:test/test.dart';
import 'package:k1s0_migration/migration.dart';

// Integration tests for PostgresMigrationRunner require a live database.
// These tests verify the MigrationRunner contract via InMemoryMigrationRunner.

void main() {
  final config = const MigrationConfig(
    migrationsDir: 'migrations',
    databaseUrl: 'postgres://localhost:5432/test',
  );

  group('PostgresMigrationRunnerのコントラクト（InMemory経由）', () {
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

    test('runUpで全ての未適用マイグレーションが適用されること', () async {
      final report = await runner.runUp();
      expect(report.appliedCount, equals(2));
      expect(report.errors, isEmpty);
    });

    test('runUpが冪等であること', () async {
      await runner.runUp();
      final report2 = await runner.runUp();
      expect(report2.appliedCount, equals(0));
    });

    test('pendingで未適用マイグレーションが返されること', () async {
      final pending = await runner.pending();
      expect(pending, hasLength(2));
      expect(pending.first.version, equals('001'));
    });

    test('runUp後にpendingが空になること', () async {
      await runner.runUp();
      final pending = await runner.pending();
      expect(pending, isEmpty);
    });

    test('statusで全マイグレーションのステータスが返されること', () async {
      await runner.runUp();
      final statuses = await runner.status();
      expect(statuses, hasLength(2));
      expect(statuses[0].version, equals('001'));
      expect(statuses[0].appliedAt, isNotNull);
    });

    test('runDownで指定ステップ数がロールバックされること', () async {
      await runner.runUp();
      final report = await runner.runDown(1);
      expect(report.appliedCount, equals(1));
      final pending = await runner.pending();
      expect(pending, hasLength(1));
      expect(pending.first.version, equals('002'));
    });

    test('runDownで適用済み数を超えた場合も正常に停止すること', () async {
      await runner.runUp();
      final report = await runner.runDown(10);
      expect(report.appliedCount, equals(2));
    });
  });

  group('MigrationFile.parseFilename', () {
    test('upマイグレーションのファイル名がパースされること', () {
      final result = MigrationFile.parseFilename('001_create_users.up.sql');
      expect(result, isNotNull);
      expect(result!.version, equals('001'));
      expect(result.name, equals('create_users'));
      expect(result.direction, equals(MigrationDirection.up));
    });

    test('downマイグレーションのファイル名がパースされること', () {
      final result = MigrationFile.parseFilename('002_add_email.down.sql');
      expect(result, isNotNull);
      expect(result!.version, equals('002'));
      expect(result.name, equals('add_email'));
      expect(result.direction, equals(MigrationDirection.down));
    });

    test('無効なファイル名の場合nullが返されること', () {
      expect(MigrationFile.parseFilename('not_a_migration.txt'), isNull);
      expect(MigrationFile.parseFilename('noversion.up.sql'), isNull);
    });
  });

  group('MigrationFile.computeChecksum', () {
    test('一貫したチェックサムが返されること', () {
      const content = 'CREATE TABLE test (id UUID);';
      final cs1 = MigrationFile.computeChecksum(content);
      final cs2 = MigrationFile.computeChecksum(content);
      expect(cs1, equals(cs2));
    });

    test('異なる内容では異なるチェックサムが返されること', () {
      final cs1 = MigrationFile.computeChecksum('SELECT 1');
      final cs2 = MigrationFile.computeChecksum('SELECT 2');
      expect(cs1, isNot(equals(cs2)));
    });
  });
}
