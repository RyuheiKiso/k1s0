import 'dart:convert';
import 'package:logging/logging.dart';

/// TelemetryConfig は telemetry ライブラリの初期化設定を保持する。
class TelemetryConfig {
  final String serviceName;
  final String version;
  final String tier;
  final String environment;
  final String? traceEndpoint;
  final double sampleRate;
  final String logLevel;
  final String logFormat;

  TelemetryConfig({
    required this.serviceName,
    required this.version,
    required this.tier,
    required this.environment,
    this.traceEndpoint,
    this.sampleRate = 1.0,
    this.logLevel = 'info',
    this.logFormat = 'json',
  });
}

/// initTelemetry は構造化ログの初期化を行う。
/// Logger.root のレベルを設定し、JSON 形式のログ出力を構成する。
void initTelemetry(TelemetryConfig cfg) {
  Logger.root.level = _parseLevel(cfg.logLevel);
  Logger.root.onRecord.listen((record) {
    if (cfg.logFormat == 'text') {
      // ignore: avoid_print
      print(
          '${record.time} [${record.level.name}] ${record.loggerName}: ${record.message}');
      if (record.error != null) {
        // ignore: avoid_print
        print('  Error: ${record.error}');
      }
    } else {
      final entry = {
        'timestamp': record.time.toUtc().toIso8601String(),
        'level': record.level.name.toLowerCase(),
        'message': record.message,
        'service': cfg.serviceName,
        'version': cfg.version,
        'tier': cfg.tier,
        'environment': cfg.environment,
        'logger': record.loggerName,
      };
      if (record.error != null) entry['error'] = record.error.toString();
      // ignore: avoid_print
      print(jsonEncode(entry));
    }
  });
}

/// shutdown は Logger.root のリスナーをクリアする。
/// Go の Provider.Shutdown / Rust の shutdown に対応する。
void shutdown() {
  Logger.root.clearListeners();
}

/// _parseLevel はログレベル文字列を logging パッケージの Level に変換する。
Level _parseLevel(String level) {
  switch (level) {
    case 'debug':
      return Level.FINE;
    case 'info':
      return Level.INFO;
    case 'warn':
      return Level.WARNING;
    case 'error':
      return Level.SEVERE;
    default:
      return Level.INFO;
  }
}
