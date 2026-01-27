import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_observability/src/error/error_info.dart';
import 'package:k1s0_observability/src/error/error_tracker.dart';
import 'package:k1s0_observability/src/logging/logger.dart';
import 'package:k1s0_observability/src/logging/log_sink.dart';
import 'package:k1s0_observability/src/tracing/trace_context.dart';
import 'package:mocktail/mocktail.dart';

class MockLogger extends Mock implements Logger {}

class MockLogSink extends Mock implements LogSink {}

void main() {
  group('ErrorSeverity', () {
    test('contains all expected severities', () {
      expect(ErrorSeverity.values, contains(ErrorSeverity.low));
      expect(ErrorSeverity.values, contains(ErrorSeverity.medium));
      expect(ErrorSeverity.values, contains(ErrorSeverity.high));
      expect(ErrorSeverity.values, contains(ErrorSeverity.fatal));
    });
  });

  group('ErrorTracker', () {
    late ErrorTracker tracker;

    setUp(() {
      tracker = ErrorTracker();
    });

    test('creates with default values', () {
      expect(tracker.logger, isNull);
      expect(tracker.onError, isNull);
      expect(tracker.reportToRemote, false);
      expect(tracker.captureStackTrace, true);
    });

    test('creates with custom values', () {
      void onError(ErrorInfo error, ErrorSeverity severity) {}

      final customTracker = ErrorTracker(
        onError: onError,
        reportToRemote: true,
        captureStackTrace: false,
      );

      expect(customTracker.onError, isNotNull);
      expect(customTracker.reportToRemote, true);
      expect(customTracker.captureStackTrace, false);
    });

    test('captureError stores error in recentErrors', () {
      tracker.captureError(Exception('Test error'));

      expect(tracker.recentErrors, hasLength(1));
      expect(tracker.recentErrors.first.type, 'Exception');
    });

    test('captureError with custom severity', () {
      ErrorSeverity? capturedSeverity;
      final trackerWithCallback = ErrorTracker(
        onError: (error, severity) => capturedSeverity = severity,
      );

      trackerWithCallback.captureError(
        Exception('Test'),
        severity: ErrorSeverity.high,
      );

      expect(capturedSeverity, ErrorSeverity.high);
    });

    test('captureError with code', () {
      tracker.captureError(
        Exception('Test'),
        code: 'TEST_ERROR',
      );

      expect(tracker.recentErrors.first.code, 'TEST_ERROR');
    });

    test('captureError with trace context', () {
      final context = TraceContext.create();

      tracker.captureError(
        Exception('Test'),
        traceContext: context,
      );

      expect(tracker.recentErrors.first.traceId, context.traceId);
    });

    test('captureError with additional context', () {
      tracker.captureError(
        Exception('Test'),
        context: {'userId': '123'},
      );

      expect(tracker.recentErrors.first.context['userId'], '123');
    });

    test('captureException stores exception in recentErrors', () {
      tracker.captureException(
        Exception('Test exception'),
        StackTrace.current,
      );

      expect(tracker.recentErrors, hasLength(1));
    });

    test('captureException with message', () {
      tracker.captureException(
        Exception('Test'),
        StackTrace.current,
        message: 'Custom message',
      );

      expect(tracker.recentErrors.first.context['message'], 'Custom message');
    });

    test('captureMessage stores message in recentErrors', () {
      tracker.captureMessage('Something happened');

      expect(tracker.recentErrors, hasLength(1));
      expect(tracker.recentErrors.first.type, 'Message');
      expect(tracker.recentErrors.first.message, 'Something happened');
    });

    test('captureMessage with code and context', () {
      tracker.captureMessage(
        'Event occurred',
        code: 'EVENT_CODE',
        context: {'key': 'value'},
      );

      expect(tracker.recentErrors.first.code, 'EVENT_CODE');
      expect(tracker.recentErrors.first.context['key'], 'value');
    });

    test('recentErrors limit is enforced', () {
      for (var i = 0; i < 150; i++) {
        tracker.captureError(Exception('Error $i'));
      }

      expect(tracker.recentErrors.length, ErrorTracker.maxRecentErrors);
    });

    test('recentErrors is unmodifiable', () {
      tracker.captureError(Exception('Test'));

      expect(
        () => tracker.recentErrors.add(
          ErrorInfo(
            type: 'Test',
            message: 'test',
            timestamp: DateTime.now().toIso8601String(),
          ),
        ),
        throwsUnsupportedError,
      );
    });

    test('clearErrors removes all errors', () {
      tracker.captureError(Exception('Error 1'));
      tracker.captureError(Exception('Error 2'));

      tracker.clearErrors();

      expect(tracker.recentErrors, isEmpty);
    });

    test('onError callback is called for each error', () {
      final capturedErrors = <ErrorInfo>[];
      final trackerWithCallback = ErrorTracker(
        onError: (error, severity) => capturedErrors.add(error),
      );

      trackerWithCallback.captureError(Exception('Error 1'));
      trackerWithCallback.captureError(Exception('Error 2'));

      expect(capturedErrors, hasLength(2));
    });

    test('captureStackTrace false does not include stack trace', () {
      final noStackTracker = ErrorTracker(captureStackTrace: false);

      noStackTracker.captureError(
        Exception('Test'),
        stackTrace: StackTrace.current,
      );

      expect(noStackTracker.recentErrors.first.stackTrace, isNull);
    });
  });

  group('createErrorTrackerWithLogger', () {
    test('creates tracker with logger', () {
      final mockSink = MockLogSink();
      final logger = Logger(
        serviceName: 'test',
        env: 'test',
        sink: mockSink,
      );

      final tracker = createErrorTrackerWithLogger(logger);

      expect(tracker.logger, logger);
    });
  });

  group('createErrorTrackerWithCallback', () {
    test('creates tracker with callback', () {
      var callbackCalled = false;
      final tracker = createErrorTrackerWithCallback(
        (error, severity) => callbackCalled = true,
      );

      tracker.captureError(Exception('Test'));

      expect(callbackCalled, true);
    });
  });
}
