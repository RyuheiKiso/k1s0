import 'package:logging/logging.dart';
import 'package:test/test.dart';

import 'package:k1s0_telemetry/telemetry.dart';

void main() {
  group('TelemetryConfig', () {
    test('必須フィールドで設定が作成されること', () {
      final cfg = TelemetryConfig(
        serviceName: 'test-service',
        version: '1.0.0',
        tier: 'system',
        environment: 'dev',
      );

      expect(cfg.serviceName, 'test-service');
      expect(cfg.version, '1.0.0');
      expect(cfg.tier, 'system');
      expect(cfg.environment, 'dev');
      expect(cfg.traceEndpoint, isNull);
      expect(cfg.sampleRate, 1.0);
      expect(cfg.logLevel, 'info');
    });

    test('全フィールドで設定が作成されること', () {
      final cfg = TelemetryConfig(
        serviceName: 'order-server',
        version: '2.0.0',
        tier: 'service',
        environment: 'prod',
        traceEndpoint: 'otel-collector:4317',
        sampleRate: 0.1,
        logLevel: 'warn',
      );

      expect(cfg.serviceName, 'order-server');
      expect(cfg.version, '2.0.0');
      expect(cfg.tier, 'service');
      expect(cfg.environment, 'prod');
      expect(cfg.traceEndpoint, 'otel-collector:4317');
      expect(cfg.sampleRate, 0.1);
      expect(cfg.logLevel, 'warn');
    });
  });

  group('initTelemetry', () {
    test('debugログレベルでLogger.rootがFINEに設定されること', () {
      final cfg = TelemetryConfig(
        serviceName: 'test-service',
        version: '1.0.0',
        tier: 'system',
        environment: 'dev',
        logLevel: 'debug',
      );

      initTelemetry(cfg);
      expect(Logger.root.level, Level.FINE);
    });

    test('infoログレベルでLogger.rootがINFOに設定されること', () {
      final cfg = TelemetryConfig(
        serviceName: 'test-service',
        version: '1.0.0',
        tier: 'system',
        environment: 'staging',
        logLevel: 'info',
      );

      initTelemetry(cfg);
      expect(Logger.root.level, Level.INFO);
    });

    test('warnログレベルでLogger.rootがWARNINGに設定されること', () {
      final cfg = TelemetryConfig(
        serviceName: 'test-service',
        version: '1.0.0',
        tier: 'system',
        environment: 'prod',
        logLevel: 'warn',
      );

      initTelemetry(cfg);
      expect(Logger.root.level, Level.WARNING);
    });

    test('errorログレベルでLogger.rootがSEVEREに設定されること', () {
      final cfg = TelemetryConfig(
        serviceName: 'test-service',
        version: '1.0.0',
        tier: 'system',
        environment: 'prod',
        logLevel: 'error',
      );

      initTelemetry(cfg);
      expect(Logger.root.level, Level.SEVERE);
    });

    test('未知のログレベルでINFOにデフォルト設定されること', () {
      final cfg = TelemetryConfig(
        serviceName: 'test-service',
        version: '1.0.0',
        tier: 'system',
        environment: 'dev',
        logLevel: 'unknown',
      );

      initTelemetry(cfg);
      expect(Logger.root.level, Level.INFO);
    });

    test('構造化ログが出力されること', () {
      final logEntries = <LogRecord>[];
      Logger.root.onRecord.listen(logEntries.add);

      final cfg = TelemetryConfig(
        serviceName: 'test-service',
        version: '1.0.0',
        tier: 'system',
        environment: 'dev',
        logLevel: 'debug',
      );

      initTelemetry(cfg);

      final logger = createLogger(cfg);
      logger.info('Test message');

      expect(logEntries, isNotEmpty);
      expect(logEntries.last.message, 'Test message');
      expect(logEntries.last.loggerName, 'test-service');
    });
  });

  group('createLogger', () {
    test('設定のサービス名でLoggerが作成されること', () {
      final cfg = TelemetryConfig(
        serviceName: 'MyService',
        version: '1.0.0',
        tier: 'system',
        environment: 'dev',
      );
      final logger = createLogger(cfg);
      expect(logger.name, 'MyService');
    });

    test('異なる設定で異なるLoggerが作成されること', () {
      final cfg1 = TelemetryConfig(
        serviceName: 'ServiceA',
        version: '1.0.0',
        tier: 'system',
        environment: 'dev',
      );
      final cfg2 = TelemetryConfig(
        serviceName: 'ServiceB',
        version: '1.0.0',
        tier: 'system',
        environment: 'dev',
      );
      final logger1 = createLogger(cfg1);
      final logger2 = createLogger(cfg2);
      expect(logger1.name, isNot(logger2.name));
    });

    test('機能するLoggerが返されること', () {
      final cfg = TelemetryConfig(
        serviceName: 'FunctionalTest',
        version: '1.0.0',
        tier: 'system',
        environment: 'dev',
      );
      final logger = createLogger(cfg);
      expect(logger, isA<Logger>());
    });
  });
}
