import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_observability/src/tracing/span.dart';
import 'package:k1s0_observability/src/tracing/trace_context.dart';

void main() {
  group('SpanStatus', () {
    test('contains all expected statuses', () {
      expect(SpanStatus.values, contains(SpanStatus.ok));
      expect(SpanStatus.values, contains(SpanStatus.error));
      expect(SpanStatus.values, contains(SpanStatus.unset));
    });
  });

  group('SpanInfo', () {
    test('creates with required fields', () {
      final span = SpanInfo(
        traceId: 'trace-123',
        spanId: 'span-456',
        name: 'test-span',
        startTime: 1000,
      );

      expect(span.traceId, 'trace-123');
      expect(span.spanId, 'span-456');
      expect(span.name, 'test-span');
      expect(span.startTime, 1000);
      expect(span.parentSpanId, isNull);
      expect(span.endTime, isNull);
      expect(span.status, SpanStatus.unset);
      expect(span.statusMessage, isNull);
      expect(span.attributes, isEmpty);
    });

    test('creates with all fields', () {
      final span = SpanInfo(
        traceId: 'trace',
        spanId: 'span',
        name: 'span',
        startTime: 1000,
        parentSpanId: 'parent',
        endTime: 2000,
        status: SpanStatus.ok,
        statusMessage: 'Success',
        attributes: {'key': 'value'},
      );

      expect(span.parentSpanId, 'parent');
      expect(span.endTime, 2000);
      expect(span.status, SpanStatus.ok);
      expect(span.statusMessage, 'Success');
      expect(span.attributes['key'], 'value');
    });

    test('durationMs returns null when not ended', () {
      final span = SpanInfo(
        traceId: 'trace',
        spanId: 'span',
        name: 'span',
        startTime: 1000,
      );

      expect(span.durationMs, isNull);
    });

    test('durationMs returns correct duration', () {
      final span = SpanInfo(
        traceId: 'trace',
        spanId: 'span',
        name: 'span',
        startTime: 1000,
        endTime: 1500,
      );

      expect(span.durationMs, 500);
    });

    test('hasEnded returns correct value', () {
      final notEnded = SpanInfo(
        traceId: 'trace',
        spanId: 'span',
        name: 'span',
        startTime: 1000,
      );
      final ended = SpanInfo(
        traceId: 'trace',
        spanId: 'span',
        name: 'span',
        startTime: 1000,
        endTime: 2000,
      );

      expect(notEnded.hasEnded, false);
      expect(ended.hasEnded, true);
    });

    test('hasError returns correct value', () {
      final noError = SpanInfo(
        traceId: 'trace',
        spanId: 'span',
        name: 'span',
        startTime: 1000,
        status: SpanStatus.ok,
      );
      final withError = SpanInfo(
        traceId: 'trace',
        spanId: 'span',
        name: 'span',
        startTime: 1000,
        status: SpanStatus.error,
      );

      expect(noError.hasError, false);
      expect(withError.hasError, true);
    });
  });

  group('ActiveSpan', () {
    late TraceContext context;
    late ActiveSpan span;

    setUp(() {
      context = TraceContext.create();
      span = ActiveSpan(
        context: context,
        name: 'test-operation',
      );
    });

    test('creates with correct initial values', () {
      expect(span.name, 'test-operation');
      expect(span.traceId, context.traceId);
      expect(span.spanId, context.spanId);
      expect(span.parentSpanId, context.parentSpanId);
      expect(span.hasEnded, false);
    });

    test('setAttribute adds attribute', () {
      span.setAttribute('key', 'value');

      final info = span.end();

      expect(info.attributes['key'], 'value');
    });

    test('setAttributes adds multiple attributes', () {
      span.setAttributes({
        'key1': 'value1',
        'key2': 42,
      });

      final info = span.end();

      expect(info.attributes['key1'], 'value1');
      expect(info.attributes['key2'], 42);
    });

    test('setAttribute does nothing after end', () {
      span.end();
      span.setAttribute('key', 'value');

      final info = span.end();

      expect(info.attributes.containsKey('key'), false);
    });

    test('setOk sets status to ok', () {
      span.setOk('Success message');

      final info = span.end();

      expect(info.status, SpanStatus.ok);
      expect(info.statusMessage, 'Success message');
    });

    test('setError sets status to error', () {
      span.setError('Error message');

      final info = span.end();

      expect(info.status, SpanStatus.error);
      expect(info.statusMessage, 'Error message');
    });

    test('setError with exception adds error attributes', () {
      final error = Exception('Test error');
      span.setError('Error occurred', error, StackTrace.current);

      final info = span.end();

      expect(info.attributes['error.type'], 'Exception');
      expect(info.attributes['error.message'], contains('Test error'));
      expect(info.attributes.containsKey('error.stack_trace'), true);
    });

    test('end returns SpanInfo', () {
      final info = span.end();

      expect(info, isA<SpanInfo>());
      expect(info.traceId, context.traceId);
      expect(info.spanId, context.spanId);
      expect(info.name, 'test-operation');
      expect(info.endTime, isNotNull);
    });

    test('end sets default status to ok', () {
      final info = span.end();

      expect(info.status, SpanStatus.ok);
    });

    test('end preserves explicit status', () {
      span.setError('Error');

      final info = span.end();

      expect(info.status, SpanStatus.error);
    });

    test('end marks span as ended', () {
      expect(span.hasEnded, false);

      span.end();

      expect(span.hasEnded, true);
    });

    test('end returns same info on multiple calls', () {
      final info1 = span.end();
      final info2 = span.end();

      expect(info1.startTime, info2.startTime);
      expect(info1.endTime, info2.endTime);
    });

    test('onEnd callback is called', () {
      SpanInfo? capturedInfo;
      final spanWithCallback = ActiveSpan(
        context: context,
        name: 'callback-test',
        onEnd: (info) => capturedInfo = info,
      );

      spanWithCallback.end();

      expect(capturedInfo, isNotNull);
      expect(capturedInfo!.name, 'callback-test');
    });

    test('onEnd callback is only called once', () {
      var callCount = 0;
      final spanWithCallback = ActiveSpan(
        context: context,
        name: 'callback-test',
        onEnd: (_) => callCount++,
      );

      spanWithCallback.end();
      spanWithCallback.end();

      expect(callCount, 1);
    });

    test('creates with initial attributes', () {
      final spanWithAttrs = ActiveSpan(
        context: context,
        name: 'attrs-test',
        attributes: {'initial': 'value'},
      );

      final info = spanWithAttrs.end();

      expect(info.attributes['initial'], 'value');
    });
  });
}
