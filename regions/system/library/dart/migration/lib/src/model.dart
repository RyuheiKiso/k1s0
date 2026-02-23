import 'dart:convert';
import 'package:crypto/crypto.dart';

class MigrationReport {
  final int appliedCount;
  final Duration elapsed;
  final List<String> errors;

  const MigrationReport({
    required this.appliedCount,
    required this.elapsed,
    this.errors = const [],
  });
}

class MigrationStatus {
  final String version;
  final String name;
  final DateTime? appliedAt;
  final String checksum;

  const MigrationStatus({
    required this.version,
    required this.name,
    this.appliedAt,
    required this.checksum,
  });
}

class PendingMigration {
  final String version;
  final String name;

  const PendingMigration({
    required this.version,
    required this.name,
  });
}

enum MigrationDirection { up, down }

class MigrationFile {
  final String version;
  final String name;
  final MigrationDirection direction;
  final String content;

  const MigrationFile({
    required this.version,
    required this.name,
    required this.direction,
    required this.content,
  });

  static ({String version, String name, MigrationDirection direction})?
      parseFilename(String filename) {
    if (!filename.endsWith('.sql')) return null;
    final stem = filename.substring(0, filename.length - 4);

    MigrationDirection direction;
    String rest;

    if (stem.endsWith('.up')) {
      direction = MigrationDirection.up;
      rest = stem.substring(0, stem.length - 3);
    } else if (stem.endsWith('.down')) {
      direction = MigrationDirection.down;
      rest = stem.substring(0, stem.length - 5);
    } else {
      return null;
    }

    final idx = rest.indexOf('_');
    if (idx <= 0 || idx >= rest.length - 1) return null;

    final version = rest.substring(0, idx);
    final name = rest.substring(idx + 1);

    return (version: version, name: name, direction: direction);
  }

  static String computeChecksum(String content) {
    final bytes = utf8.encode(content);
    return sha256.convert(bytes).toString();
  }
}
