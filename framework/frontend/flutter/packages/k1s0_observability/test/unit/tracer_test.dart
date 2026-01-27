import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_observability/src/tracing/span.dart';
import 'package:k1s0_observability/src/tracing/trace_context.dart';
import 'package:k1s0_observability/src/tracing/tracer.dart';
import 'package:mocktail/mocktail.dart';

class MockSpanExporter extends Mock implements SpanExporter {}

class FakeSpanInfoList extends Fake implements List<SpanInfo> {}

void main() {
  late MockSpanExporter mockExporter;
  late Tracer tracer;

  setUpAll(() {
    registerFallbackValue(<SpanInfo>[]);
  });

  setUp(() {
    mockExporter = MockSpanExporter();
    when(() => mockExporter.export(any())).thenAnswer((_) async {});
    tracer = Tracer(
      serviceName: 'test-service',
      exporter: mockExporter,
      samplingRate: 1.0,
    );
  });

  group('Tracer', () {
    test('creates with required parameters', () {
      final simpleTracer = Tracer(serviceName: 'simple');

      expect(simpleTracer.serviceName, 'simple');
      expect(simpleTracer.exporter, isNull);
      expect(simpleTracer.samplingRate, 1.0);
    });

    test('creates with custom sampling rate', () {
      final sampledTracer = Tracer(
        serviceName: 'sampled',
        samplingRate: 0.5,
      );

      expect(sampledTracer.samplingRate, 0.5);
    });

    test('startSpan creates active span', () {
      final span = tracer.startSpan('test-operation');

      expect(span, isA<ActiveSpan>());
      expect(span.name, 'test-operation');
      expect(span.traceId, isNotEmpty);
      expect(span.spanId, isNotEmpty);
    });

    test('startSpan sets service name attribute', () {
      final span = tracer.startSpan('operation');

      span.end();

      verify(() => mockExporter.export(any(
            that: isA<List<SpanInfo>>().having(
              (list) => list.first.attributes['service.name'],
              'service.name',
              'test-service',
            ),
          ))).called(1);
    });

    test('startSpan with custom attributes', () {
      final span = tracer.startSpan(
        'operation',
        attributes: {'custom': 'value'},
      );

      span.end();

      verify(() => mockExporter.export(any(
            that: isA<List<SpanInfo>>().having(
              (list) => list.first.attributes['custom'],
              'custom',
              'value',
            ),
          ))).called(1);
    });

    test('startSpan creates child span when parent provided', () {
      final parentContext = TraceContext.create();
      final span = tracer.startSpan('child', parent: parentContext);

      expect(span.traceId, parentContext.traceId);
      expect(span.parentSpanId, parentContext.spanId);
    });

    test('currentContext returns or creates context', () {
      final context = tracer.currentContext;

      expect(context, isA<TraceContext>());
      expect(context.traceId, isNotEmpty);
    });

    test('currentContext can be set', () {
      final customContext = TraceContext.create();
      tracer.currentContext = customContext;

      expect(tracer.currentContext.traceId, customContext.traceId);
    });

    test('trace executes function and ends span', () async {
      var executed = false;

      await tracer.trace('operation', (span) async {
        executed = true;
        return 'result';
      });

      expect(executed, true);
      verify(() => mockExporter.export(any())).called(1);
    });

    test('trace returns function result', () async {
      final result = await tracer.trace('operation', (span) async {
        return 42;
      });

      expect(result, 42);
    });

    test('trace sets OK status on success', () async {
      await tracer.trace('operation', (span) async {
        return 'success';
      });

      verify(() => mockExporter.export(any(
            that: isA<List<SpanInfo>>().having(
              (list) => list.first.status,
              'status',
              SpanStatus.ok,
            ),
          ))).called(1);
    });

    test('trace sets error status on exception', () async {
      try {
        await tracer.trace('operation', (span) async {
          throw Exception('Test error');
        });
      } catch (_) {}

      verify(() => mockExporter.export(any(
            that: isA<List<SpanInfo>>().having(
              (list) => list.first.status,
              'status',
              SpanStatus.error,
            ),
          ))).called(1);
    });

    test('trace rethrows exceptions', () async {
      await expectLater(
        tracer.trace('operation', (span) async {
          throw Exception('Test error');
        }),
        throwsException,
      );
    });

    test('traceSync executes sync function', () {
      var executed = false;

      tracer.traceSync('operation', (span) {
        executed = true;
        return 'result';
      });

      expect(executed, true);
    });

    test('traceSync returns function result', () {
      final result = tracer.traceSync('operation', (span) => 42);

      expect(result, 42);
    });

    test('shutdown calls exporter shutdown', () async {
      when(() => mockExporter.shutdown()).thenAnswer((_) async {});

      await tracer.shutdown();

      verify(() => mockExporter.shutdown()).called(1);
    });
  });

  group('ConsoleSpanExporter', () {
    test('export prints spans', () async {
      final exporter = ConsoleSpanExporter();
      final span = SpanInfo(
        traceId: 'trace123456789012345678901234',
        spanId: 'span1234',
        name: 'test-span',
        startTime: DateTime.now().millisecondsSinceEpoch,
        endTime: DateTime.now().millisecondsSinceEpoch + 100,
        status: SpanStatus.ok,
      );

      // Should not throw
      await exporter.export([span]);
    });

    test('shutdown completes', () async {
      final exporter = ConsoleSpanExporter();

      await exporter.shutdown();
    });
  });

  group('BufferedSpanExporter', () {
    late MockSpanExporter delegateExporter;
    late BufferedSpanExporter bufferedExporter;

    setUp(() {
      delegateExporter = MockSpanExporter();
      when(() => delegateExporter.export(any())).thenAnswer((_) async {});
      when(() => delegateExporter.shutdown()).thenAnswer((_) async {});

      bufferedExporter = BufferedSpanExporter(
        delegate: delegateExporter,
        batchSize: 2,
        flushInterval: const Duration(seconds: 60),
      );
    });

    tearDown(() async {
      await bufferedExporter.shutdown();
    });

    test('buffers spans until batch size', () async {
      final span1 = SpanInfo(
        traceId: 'trace1',
        spanId: 'span1',
        name: 'span1',
        startTime: 0,
      );

      await bufferedExporter.export([span1]);

      // Should not flush yet (batch size is 2)
      verifyNever(() => delegateExporter.export(any()));
    });

    test('flushes when batch size reached', () async {
      final spans = [
        SpanInfo(traceId: 'trace1', spanId: 'span1', name: 'span1', startTime: 0),
        SpanInfo(traceId: 'trace2', spanId: 'span2', name: 'span2', startTime: 0),
      ];

      await bufferedExporter.export(spans);

      verify(() => delegateExporter.export(any())).called(1);
    });

    test('shutdown flushes remaining spans', () async {
      final span = SpanInfo(
        traceId: 'trace1',
        spanId: 'span1',
        name: 'span1',
        startTime: 0,
      );

      await bufferedExporter.export([span]);
      await bufferedExporter.shutdown();

      verify(() => delegateExporter.export(any())).called(1);
      verify(() => delegateExporter.shutdown()).called(1);
    });
  });

  group('createConsoleTracer', () {
    test('creates tracer with console exporter', () {
      final consoleTracer = createConsoleTracer(
        serviceName: 'console-service',
      );

      expect(consoleTracer.serviceName, 'console-service');
      expect(consoleTracer.exporter, isA<ConsoleSpanExporter>());
    });

    test('creates tracer with custom sampling rate', () {
      final consoleTracer = createConsoleTracer(
        serviceName: 'service',
        samplingRate: 0.1,
      );

      expect(consoleTracer.samplingRate, 0.1);
    });
  });
}
