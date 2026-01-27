import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_observability/src/logging/log_entry.dart';
import 'package:k1s0_observability/src/logging/log_level.dart';
import 'package:k1s0_observability/src/logging/log_sink.dart';
import 'package:k1s0_observability/src/logging/logger.dart';
import 'package:mocktail/mocktail.dart';

class MockLogSink extends Mock implements LogSink {}

class FakeLogEntry extends Fake implements LogEntry {}

void main() {
  late MockLogSink mockSink;
  late Logger logger;

  setUpAll(() {
    registerFallbackValue(FakeLogEntry());
  });

  setUp(() {
    mockSink = MockLogSink();
    logger = Logger(
      serviceName: 'test-service',
      env: 'test',
      sink: mockSink,
      minLevel: LogLevel.debug,
    );
  });

  group('Logger', () {
    test('creates with required parameters', () {
      expect(logger.serviceName, 'test-service');
      expect(logger.env, 'test');
      expect(logger.minLevel, LogLevel.debug);
    });

    test('creates with default minLevel', () {
      final defaultLogger = Logger(
        serviceName: 'service',
        env: 'dev',
        sink: mockSink,
      );

      expect(defaultLogger.minLevel, LogLevel.debug);
    });

    test('debug logs debug level entry', () {
      logger.debug('Debug message');

      verify(() => mockSink.write(any(
            that: isA<LogEntry>().having(
              (e) => e.level,
              'level',
              LogLevel.debug,
            ),
          ))).called(1);
    });

    test('info logs info level entry', () {
      logger.info('Info message');

      verify(() => mockSink.write(any(
            that: isA<LogEntry>().having(
              (e) => e.level,
              'level',
              LogLevel.info,
            ),
          ))).called(1);
    });

    test('warn logs warn level entry', () {
      logger.warn('Warning message');

      verify(() => mockSink.write(any(
            that: isA<LogEntry>().having(
              (e) => e.level,
              'level',
              LogLevel.warn,
            ),
          ))).called(1);
    });

    test('error logs error level entry', () {
      logger.error('Error message');

      verify(() => mockSink.write(any(
            that: isA<LogEntry>().having(
              (e) => e.level,
              'level',
              LogLevel.error,
            ),
          ))).called(1);
    });

    test('logs include message', () {
      logger.info('Test message content');

      verify(() => mockSink.write(any(
            that: isA<LogEntry>().having(
              (e) => e.message,
              'message',
              'Test message content',
            ),
          ))).called(1);
    });

    test('logs include service name', () {
      logger.info('Test');

      verify(() => mockSink.write(any(
            that: isA<LogEntry>().having(
              (e) => e.serviceName,
              'serviceName',
              'test-service',
            ),
          ))).called(1);
    });

    test('logs include env', () {
      logger.info('Test');

      verify(() => mockSink.write(any(
            that: isA<LogEntry>().having(
              (e) => e.env,
              'env',
              'test',
            ),
          ))).called(1);
    });

    test('logs include extra data', () {
      logger.info('Test', {'key': 'value'});

      verify(() => mockSink.write(any(
            that: isA<LogEntry>().having(
              (e) => e.extra['key'],
              'extra key',
              'value',
            ),
          ))).called(1);
    });

    test('error logs include error info', () {
      logger.error(
        'Error occurred',
        error: Exception('Test exception'),
        stackTrace: StackTrace.current,
      );

      verify(() => mockSink.write(any(
            that: isA<LogEntry>().having(
              (e) => e.errorInfo,
              'errorInfo',
              isNotNull,
            ),
          ))).called(1);
    });

    test('respects minLevel and does not log below threshold', () {
      final warnLogger = Logger(
        serviceName: 'service',
        env: 'prod',
        sink: mockSink,
        minLevel: LogLevel.warn,
      );

      warnLogger.debug('Should not log');
      warnLogger.info('Should not log');

      verifyNever(() => mockSink.write(any()));
    });

    test('logs at or above minLevel', () {
      final warnLogger = Logger(
        serviceName: 'service',
        env: 'prod',
        sink: mockSink,
        minLevel: LogLevel.warn,
      );

      warnLogger.warn('Should log');
      warnLogger.error('Should log');

      verify(() => mockSink.write(any())).called(2);
    });

    test('child creates logger with same service name and new span', () {
      final childLogger = logger.child(spanName: 'operation');

      expect(childLogger.serviceName, logger.serviceName);
      expect(childLogger.env, logger.env);
      expect(childLogger.traceContext.traceId, logger.traceContext.traceId);
      expect(
        childLogger.traceContext.parentSpanId,
        logger.traceContext.spanId,
      );
    });

    test('flush delegates to sink', () async {
      when(() => mockSink.flush()).thenAnswer((_) async {});

      await logger.flush();

      verify(() => mockSink.flush()).called(1);
    });

    test('dispose calls sink dispose', () {
      logger.dispose();

      verify(() => mockSink.dispose()).called(1);
    });
  });

  group('createConsoleLogger', () {
    test('creates logger with console sink', () {
      final consoleLogger = createConsoleLogger(
        serviceName: 'console-service',
        env: 'dev',
      );

      expect(consoleLogger.serviceName, 'console-service');
      expect(consoleLogger.env, 'dev');
    });

    test('creates logger with custom minLevel', () {
      final consoleLogger = createConsoleLogger(
        serviceName: 'service',
        env: 'prod',
        minLevel: LogLevel.warn,
      );

      expect(consoleLogger.minLevel, LogLevel.warn);
    });
  });
}
