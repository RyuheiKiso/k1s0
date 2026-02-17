import 'dart:async';
import 'dart:io';

import 'package:logging/logging.dart';
import 'package:shelf/shelf.dart';
import 'package:test/test.dart';

import 'package:k1s0_telemetry/telemetry.dart';

void main() {
  late Logger logger;
  late List<LogRecord> logEntries;
  late StreamSubscription<LogRecord> subscription;

  setUp(() {
    logger = createLogger('MiddlewareTest');
    logEntries = [];
    Logger.root.level = Level.ALL;
    subscription = Logger.root.onRecord.listen((record) {
      logEntries.add(record);
    });
  });

  tearDown(() async {
    await subscription.cancel();
  });

  group('TelemetryMiddleware', () {
    test('should log incoming requests', () async {
      final middleware = TelemetryMiddleware(logger: logger);
      final handler = const Pipeline()
          .addMiddleware(middleware.middleware)
          .addHandler(
            (request) => Response.ok('hello'),
          );

      final request = Request(
        'GET',
        Uri.parse('http://localhost/api/users'),
      );
      await handler(request);

      final infoLogs =
          logEntries.where((r) => r.level == Level.INFO).toList();
      expect(infoLogs, isNotEmpty);

      final logMessage = infoLogs.last.message;
      expect(logMessage, contains('GET'));
      expect(logMessage, contains('/api/users'));
      expect(logMessage, contains('200'));
    });

    test('should measure response time', () async {
      final middleware = TelemetryMiddleware(logger: logger);
      final handler = const Pipeline()
          .addMiddleware(middleware.middleware)
          .addHandler(
            (request) async {
              await Future.delayed(Duration(milliseconds: 50));
              return Response.ok('slow');
            },
          );

      final request = Request(
        'GET',
        Uri.parse('http://localhost/api/slow'),
      );
      await handler(request);

      final infoLogs =
          logEntries.where((r) => r.level == Level.INFO).toList();
      expect(infoLogs, isNotEmpty);

      final logMessage = infoLogs.last.message;
      // ログメッセージに duration 情報が含まれることを確認
      expect(logMessage, contains('ms'));
    });

    test('should handle error responses', () async {
      final middleware = TelemetryMiddleware(logger: logger);
      final handler = const Pipeline()
          .addMiddleware(middleware.middleware)
          .addHandler(
            (request) => Response.internalServerError(body: 'error'),
          );

      final request = Request(
        'POST',
        Uri.parse('http://localhost/api/fail'),
      );
      final response = await handler(request);

      expect(response.statusCode, HttpStatus.internalServerError);

      final warningLogs =
          logEntries.where((r) => r.level == Level.WARNING).toList();
      expect(warningLogs, isNotEmpty);

      final logMessage = warningLogs.last.message;
      expect(logMessage, contains('POST'));
      expect(logMessage, contains('/api/fail'));
      expect(logMessage, contains('500'));
    });

    test('should propagate trace-id header', () async {
      final middleware = TelemetryMiddleware(logger: logger);
      String? capturedTraceId;

      final handler = const Pipeline()
          .addMiddleware(middleware.middleware)
          .addHandler(
            (request) {
              capturedTraceId = request.headers['x-trace-id'];
              return Response.ok('traced');
            },
          );

      final request = Request(
        'GET',
        Uri.parse('http://localhost/api/trace'),
        headers: {'x-trace-id': 'abc-123-def'},
      );
      final response = await handler(request);

      expect(capturedTraceId, 'abc-123-def');
      expect(response.headers['x-trace-id'], 'abc-123-def');
    });

    test('should generate trace-id when not provided', () async {
      final middleware = TelemetryMiddleware(logger: logger);
      String? capturedTraceId;

      final handler = const Pipeline()
          .addMiddleware(middleware.middleware)
          .addHandler(
            (request) {
              capturedTraceId = request.headers['x-trace-id'];
              return Response.ok('auto-traced');
            },
          );

      final request = Request(
        'GET',
        Uri.parse('http://localhost/api/auto-trace'),
      );
      final response = await handler(request);

      expect(capturedTraceId, isNotNull);
      expect(capturedTraceId, isNotEmpty);
      expect(response.headers['x-trace-id'], capturedTraceId);
    });

    test('should handle exceptions in inner handler', () async {
      final middleware = TelemetryMiddleware(logger: logger);
      final handler = const Pipeline()
          .addMiddleware(middleware.middleware)
          .addHandler(
            (request) => throw Exception('unexpected error'),
          );

      final request = Request(
        'GET',
        Uri.parse('http://localhost/api/crash'),
      );
      final response = await handler(request);

      expect(response.statusCode, HttpStatus.internalServerError);

      final severeLogs =
          logEntries.where((r) => r.level == Level.SEVERE).toList();
      expect(severeLogs, isNotEmpty);
      expect(severeLogs.last.message, contains('unexpected error'));
    });
  });
}
