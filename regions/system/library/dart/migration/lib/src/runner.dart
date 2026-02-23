import 'model.dart';
import 'config.dart';

abstract class MigrationRunner {
  Future<MigrationReport> runUp();
  Future<MigrationReport> runDown(int steps);
  Future<List<MigrationStatus>> status();
  Future<List<PendingMigration>> pending();
}

class InMemoryMigrationRunner implements MigrationRunner {
  final MigrationConfig config;
  final List<_MigrationEntry> _upMigrations;
  final Map<String, _MigrationEntry> _downMigrations;
  final List<MigrationStatus> _applied = [];

  InMemoryMigrationRunner({
    required this.config,
    required List<({String version, String name, String content})> ups,
    required List<({String version, String name, String content})> downs,
  })  : _upMigrations = ups
            .map((u) =>
                _MigrationEntry(version: u.version, name: u.name, content: u.content))
            .toList()
          ..sort((a, b) => a.version.compareTo(b.version)),
        _downMigrations = {
          for (final d in downs)
            d.version:
                _MigrationEntry(version: d.version, name: d.name, content: d.content),
        };

  @override
  Future<MigrationReport> runUp() async {
    final stopwatch = Stopwatch()..start();
    final appliedVersions = _applied.map((s) => s.version).toSet();
    var count = 0;

    for (final mf in _upMigrations) {
      if (appliedVersions.contains(mf.version)) continue;
      _applied.add(MigrationStatus(
        version: mf.version,
        name: mf.name,
        appliedAt: DateTime.now().toUtc(),
        checksum: MigrationFile.computeChecksum(mf.content),
      ));
      count++;
    }

    stopwatch.stop();
    return MigrationReport(
      appliedCount: count,
      elapsed: stopwatch.elapsed,
    );
  }

  @override
  Future<MigrationReport> runDown(int steps) async {
    final stopwatch = Stopwatch()..start();
    var count = 0;

    for (var i = 0; i < steps; i++) {
      if (_applied.isEmpty) break;
      _applied.removeLast();
      count++;
    }

    stopwatch.stop();
    return MigrationReport(
      appliedCount: count,
      elapsed: stopwatch.elapsed,
    );
  }

  @override
  Future<List<MigrationStatus>> status() async {
    final appliedMap = {for (final s in _applied) s.version: s};

    return _upMigrations.map((mf) {
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
    final appliedVersions = _applied.map((s) => s.version).toSet();
    return _upMigrations
        .where((mf) => !appliedVersions.contains(mf.version))
        .map((mf) => PendingMigration(version: mf.version, name: mf.name))
        .toList();
  }
}

class _MigrationEntry {
  final String version;
  final String name;
  final String content;

  const _MigrationEntry({
    required this.version,
    required this.name,
    required this.content,
  });
}
