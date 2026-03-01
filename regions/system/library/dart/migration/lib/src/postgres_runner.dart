import 'package:postgres/postgres.dart';

import 'config.dart';
import 'model.dart';
import 'runner.dart';

class PostgresMigrationRunner implements MigrationRunner {
  final Connection _conn;
  final MigrationConfig config;
  final List<({String version, String name, String content})> _ups;
  final List<({String version, String name, String content})> _downs;

  PostgresMigrationRunner({
    required Connection connection,
    required this.config,
    required List<({String version, String name, String content})> ups,
    required List<({String version, String name, String content})> downs,
  })  : _conn = connection,
        _ups = List.of(ups)
          ..sort((a, b) => a.version.compareTo(b.version)),
        _downs = downs;

  Future<void> _ensureTable() async {
    await _conn.execute('''
      CREATE TABLE IF NOT EXISTS ${config.tableName} (
        version TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        checksum TEXT NOT NULL
      )
    ''');
  }

  Future<Set<String>> _appliedVersions() async {
    final rows = await _conn.execute(
      'SELECT version FROM ${config.tableName} ORDER BY version ASC',
    );
    return rows.map((r) => r[0] as String).toSet();
  }

  @override
  Future<MigrationReport> runUp() async {
    await _ensureTable();
    final stopwatch = Stopwatch()..start();
    final applied = await _appliedVersions();
    var count = 0;
    final errors = <String>[];

    for (final mf in _ups) {
      if (applied.contains(mf.version)) continue;
      try {
        await _conn.execute(mf.content);
        final checksum = MigrationFile.computeChecksum(mf.content);
        await _conn.execute(
          Sql.named(
              'INSERT INTO ${config.tableName} (version, name, applied_at, checksum) '
              'VALUES (@version, @name, NOW(), @checksum)'),
          parameters: {
            'version': mf.version,
            'name': mf.name,
            'checksum': checksum,
          },
        );
        count++;
      } catch (e) {
        errors.add('Migration ${mf.version} failed: $e');
        stopwatch.stop();
        return MigrationReport(
          appliedCount: count,
          elapsed: stopwatch.elapsed,
          errors: errors,
        );
      }
    }

    stopwatch.stop();
    return MigrationReport(
      appliedCount: count,
      elapsed: stopwatch.elapsed,
      errors: errors,
    );
  }

  @override
  Future<MigrationReport> runDown(int steps) async {
    await _ensureTable();
    final stopwatch = Stopwatch()..start();
    final applied = await _appliedVersions();
    final appliedSorted = applied.toList()..sort((a, b) => b.compareTo(a));
    final downsMap = {for (final d in _downs) d.version: d};
    var count = 0;
    final errors = <String>[];

    for (var i = 0; i < steps && i < appliedSorted.length; i++) {
      final version = appliedSorted[i];
      final down = downsMap[version];
      if (down == null) break;
      try {
        await _conn.execute(down.content);
        await _conn.execute(
          Sql.named('DELETE FROM ${config.tableName} WHERE version = @version'),
          parameters: {'version': version},
        );
        count++;
      } catch (e) {
        errors.add('Rollback $version failed: $e');
        break;
      }
    }

    stopwatch.stop();
    return MigrationReport(
      appliedCount: count,
      elapsed: stopwatch.elapsed,
      errors: errors,
    );
  }

  @override
  Future<List<MigrationStatus>> status() async {
    await _ensureTable();
    final rows = await _conn.execute(
      'SELECT version, name, applied_at, checksum FROM ${config.tableName}',
    );
    final appliedMap = {
      for (final r in rows)
        r[0] as String: (
          appliedAt: r[2] as DateTime,
          checksum: r[3] as String,
        )
    };

    return _ups.map((mf) {
      final cs = MigrationFile.computeChecksum(mf.content);
      final applied = appliedMap[mf.version];
      return MigrationStatus(
        version: mf.version,
        name: mf.name,
        appliedAt: applied?.appliedAt,
        checksum: cs,
      );
    }).toList();
  }

  @override
  Future<List<PendingMigration>> pending() async {
    await _ensureTable();
    final applied = await _appliedVersions();
    return _ups
        .where((mf) => !applied.contains(mf.version))
        .map((mf) => PendingMigration(version: mf.version, name: mf.name))
        .toList();
  }
}
