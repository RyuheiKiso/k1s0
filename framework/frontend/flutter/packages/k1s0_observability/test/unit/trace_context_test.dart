import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_observability/src/tracing/trace_context.dart';

void main() {
  group('TraceContext', () {
    test('create generates new trace and span IDs', () {
      final context = TraceContext.create();

      expect(context.traceId, isNotEmpty);
      expect(context.spanId, isNotEmpty);
      expect(context.parentSpanId, isNull);
      expect(context.requestId, isNull);
    });

    test('create with custom IDs', () {
      final context = TraceContext.create(
        traceId: 'custom-trace',
        spanId: 'custom-span',
        requestId: 'custom-request',
      );

      expect(context.traceId, 'custom-trace');
      expect(context.spanId, 'custom-span');
      expect(context.requestId, 'custom-request');
    });

    test('create with baggage', () {
      final context = TraceContext.create(
        baggage: {'key': 'value'},
      );

      expect(context.baggage['key'], 'value');
    });

    test('fromTraceparent parses valid header', () {
      final context = TraceContext.fromTraceparent(
        '00-0123456789abcdef0123456789abcdef-0123456789abcdef-01',
      );

      expect(context.traceId, '0123456789abcdef0123456789abcdef');
      expect(context.spanId, '0123456789abcdef');
    });

    test('fromTraceparent throws for invalid format', () {
      expect(
        () => TraceContext.fromTraceparent('invalid'),
        throwsFormatException,
      );
    });

    test('tryFromTraceparent returns null for invalid format', () {
      final context = TraceContext.tryFromTraceparent('invalid');

      expect(context, isNull);
    });

    test('tryFromTraceparent returns null for null input', () {
      final context = TraceContext.tryFromTraceparent(null);

      expect(context, isNull);
    });

    test('tryFromTraceparent returns context for valid format', () {
      final context = TraceContext.tryFromTraceparent(
        '00-0123456789abcdef0123456789abcdef-0123456789abcdef-01',
      );

      expect(context, isNotNull);
      expect(context!.traceId, '0123456789abcdef0123456789abcdef');
    });

    test('createChild creates context with same trace ID', () {
      final parent = TraceContext.create();
      final child = parent.createChild(name: 'child-span');

      expect(child.traceId, parent.traceId);
      expect(child.parentSpanId, parent.spanId);
      expect(child.spanId, isNot(parent.spanId));
    });

    test('createChild inherits requestId', () {
      final parent = TraceContext.create(requestId: 'request-123');
      final child = parent.createChild();

      expect(child.requestId, 'request-123');
    });

    test('createChild inherits baggage', () {
      final parent = TraceContext.create(baggage: {'key': 'value'});
      final child = parent.createChild();

      expect(child.baggage['key'], 'value');
    });

    test('copyWith creates new context with updated values', () {
      final original = TraceContext.create();
      final copied = original.copyWith(requestId: 'new-request');

      expect(copied.traceId, original.traceId);
      expect(copied.spanId, original.spanId);
      expect(copied.requestId, 'new-request');
    });

    test('withBaggage adds baggage item', () {
      final context = TraceContext.create();
      final withBaggage = context.withBaggage('newKey', 'newValue');

      expect(withBaggage.baggage['newKey'], 'newValue');
      expect(context.baggage.containsKey('newKey'), false);
    });

    test('toTraceparent generates valid header', () {
      const context = TraceContext(
        traceId: '0123456789abcdef0123456789abcdef',
        spanId: '0123456789abcdef',
      );

      final header = context.toTraceparent();

      expect(header, '00-0123456789abcdef0123456789abcdef-0123456789abcdef-01');
    });

    test('toHeaders includes all relevant headers', () {
      const context = TraceContext(
        traceId: 'trace123',
        spanId: 'span456',
        requestId: 'request789',
      );

      final headers = context.toHeaders();

      expect(headers['traceparent'], '00-trace123-span456-01');
      expect(headers['x-trace-id'], 'trace123');
      expect(headers['x-span-id'], 'span456');
      expect(headers['x-request-id'], 'request789');
    });

    test('toHeaders excludes request ID when null', () {
      const context = TraceContext(
        traceId: 'trace123',
        spanId: 'span456',
      );

      final headers = context.toHeaders();

      expect(headers.containsKey('x-request-id'), false);
    });

    test('toString returns readable format', () {
      const context = TraceContext(
        traceId: 'trace123',
        spanId: 'span456',
        parentSpanId: 'parent789',
      );

      final str = context.toString();

      expect(str, contains('trace123'));
      expect(str, contains('span456'));
      expect(str, contains('parent789'));
    });

    test('equality based on traceId and spanId', () {
      const context1 = TraceContext(
        traceId: 'trace',
        spanId: 'span',
      );
      const context2 = TraceContext(
        traceId: 'trace',
        spanId: 'span',
        requestId: 'different',
      );
      const context3 = TraceContext(
        traceId: 'trace',
        spanId: 'different',
      );

      expect(context1 == context2, true);
      expect(context1 == context3, false);
    });

    test('hashCode based on traceId and spanId', () {
      const context1 = TraceContext(
        traceId: 'trace',
        spanId: 'span',
      );
      const context2 = TraceContext(
        traceId: 'trace',
        spanId: 'span',
      );

      expect(context1.hashCode, context2.hashCode);
    });
  });
}
