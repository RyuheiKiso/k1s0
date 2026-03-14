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
    test('upマイグレーションのファイル名がパースされること', () {
      final result =
          MigrationFile.parseFilename('20240101000001_create_users.up.sql');
      expect(result, isNotNull);
      expect(result!.version, equals('20240101000001'));
      expect(result.name, equals('create_users'));
      expect(result.direction, equals(MigrationDirection.up));
    });

    test('downマイグレーションのファイル名がパースされること', () {
      final result =
          MigrationFile.parseFilename('20240101000001_create_users.down.sql');
      expect(result, isNotNull);
      expect(result!.version, equals('20240101000001'));
      expect(result.name, equals('create_users'));
      expect(result.direction, equals(MigrationDirection.down));
    });

    test('無効なファイル名の場合nullが返されること', () {
      expect(MigrationFile.parseFilename('invalid.sql'), isNull);
      expect(MigrationFile.parseFilename('no_direction.sql'), isNull);
      expect(MigrationFile.parseFilename('_.up.sql'), isNull);
    });
  });

  group('checksum', () {
    test('同じ内容では常に同じチェックサムが返されること', () {
      const content = 'CREATE TABLE users (id SERIAL PRIMARY KEY);';
      expect(
        MigrationFile.computeChecksum(content),
        equals(MigrationFile.computeChecksum(content)),
      );
    });

    test('異なる内容ではチェックサムが異なること', () {
      expect(
        MigrationFile.computeChecksum('CREATE TABLE users;'),
        isNot(equals(MigrationFile.computeChecksum('CREATE TABLE orders;'))),
      );
    });
  });

  group('MigrationError', () {
    test('メッセージを指定して生成できること', () {
      final err = MigrationError('test error');
      expect(err.message, equals('test error'));
    });

    test('connectionFailedエラーが生成されること', () {
      final err = MigrationError.connectionFailed('failed');
      expect(err.code, equals('CONNECTION_FAILED'));
    });

    test('checksumMismatchエラーが生成されること', () {
      final err = MigrationError.checksumMismatch('v1', 'abc', 'def');
      expect(err.code, equals('CHECKSUM_MISMATCH'));
    });

    test('directoryNotFoundエラーが生成されること', () {
      final err = MigrationError.directoryNotFound('/tmp');
      expect(err.code, equals('DIRECTORY_NOT_FOUND'));
    });
  });

  group('MigrationConfig', () {
    test('デフォルトのテーブル名が設定されること', () {
      const config = MigrationConfig(
        migrationsDir: '.',
        databaseUrl: 'memory://',
      );
      expect(config.tableName, equals('_migrations'));
    });

    test('カスタムテーブル名が設定できること', () {
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

    test('runUpで全マイグレーションが適用されること', () async {
      final report = await runner.runUp();
      expect(report.appliedCount, equals(3));
      expect(report.errors, isEmpty);
    });

    test('runUpが冪等であること', () async {
      await runner.runUp();
      final report = await runner.runUp();
      expect(report.appliedCount, equals(0));
    });

    test('runDownで1ステップがロールバックされること', () async {
      await runner.runUp();
      final report = await runner.runDown(1);
      expect(report.appliedCount, equals(1));

      final p = await runner.pending();
      expect(p.length, equals(1));
      expect(p[0].version, equals('20240201000001'));
    });

    test('runDownで複数ステップがロールバックされること', () async {
      await runner.runUp();
      final report = await runner.runDown(2);
      expect(report.appliedCount, equals(2));

      final p = await runner.pending();
      expect(p.length, equals(2));
    });

    test('runDownで適用済み数を超えた場合も正しく処理されること', () async {
      await runner.runUp();
      final report = await runner.runDown(10);
      expect(report.appliedCount, equals(3));
    });

    test('初期状態でstatusが全てpendingを示すこと', () async {
      final statuses = await runner.status();
      expect(statuses.length, equals(3));
      for (final s in statuses) {
        expect(s.appliedAt, isNull);
      }
    });

    test('runUp後にstatusが全て適用済みを示すこと', () async {
      await runner.runUp();
      final statuses = await runner.status();
      expect(statuses.length, equals(3));
      for (final s in statuses) {
        expect(s.appliedAt, isNotNull);
      }
    });

    test('pendingで未適用マイグレーションが全て返されること', () async {
      final p = await runner.pending();
      expect(p.length, equals(3));
      expect(p[0].version, equals('20240101000001'));
      expect(p[1].version, equals('20240101000002'));
      expect(p[2].version, equals('20240201000001'));
    });

    test('全適用後にpendingが空になること', () async {
      await runner.runUp();
      final p = await runner.pending();
      expect(p, isEmpty);
    });
  });
}
