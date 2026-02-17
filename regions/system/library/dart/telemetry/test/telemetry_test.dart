import 'package:logging/logging.dart';
import 'package:test/test.dart';

import 'package:k1s0_telemetry/telemetry.dart';

void main() {
  group('TelemetryConfig', () {
    test('should create config with required fields', () {
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

    test('should create config with all fields', () {
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
    test('should set Logger.root level to FINE for debug', () {
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

    test('should set Logger.root level to INFO for info', () {
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

    test('should set Logger.root level to WARNING for warn', () {
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

    test('should set Logger.root level to SEVERE for error', () {
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

    test('should default to INFO for unknown log level', () {
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

    test('should produce structured log output', () {
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

      final logger = createLogger('TestLogger');
      logger.info('Test message');

      expect(logEntries, isNotEmpty);
      expect(logEntries.last.message, 'Test message');
      expect(logEntries.last.loggerName, 'TestLogger');
    });
  });

  group('createLogger', () {
    test('should create a Logger with the given name', () {
      final logger = createLogger('MyService');
      expect(logger.name, 'MyService');
    });

    test('should create different loggers with different names', () {
      final logger1 = createLogger('ServiceA');
      final logger2 = createLogger('ServiceB');
      expect(logger1.name, isNot(logger2.name));
    });

    test('should return a functional logger', () {
      final logger = createLogger('FunctionalTest');
      expect(logger, isA<Logger>());
    });
  });
}
