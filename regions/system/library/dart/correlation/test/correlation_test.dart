import 'package:k1s0_correlation/correlation.dart';
import 'package:test/test.dart';

void main() {
  group('CorrelationId', () {
    test('generates a non-empty id', () {
      final id = CorrelationId.generate();
      expect(id.value, isNotEmpty);
      expect(id.isEmpty, isFalse);
    });

    test('generates unique ids', () {
      final id1 = CorrelationId.generate();
      final id2 = CorrelationId.generate();
      expect(id1, isNot(equals(id2)));
    });

    test('parses any string without validation', () {
      final id = CorrelationId.parse('custom-id-123');
      expect(id.toString(), 'custom-id-123');
    });

    test('isEmpty returns true for empty string', () {
      final id = CorrelationId.parse('');
      expect(id.isEmpty, isTrue);
    });
  });

  group('TraceId', () {
    test('generates a 32-character id', () {
      final id = TraceId.generate();
      expect(id.value.length, 32);
    });

    test('generates lowercase hex only', () {
      final id = TraceId.generate();
      expect(id.value, matches(RegExp(r'^[0-9a-f]{32}$')));
    });

    test('generates unique ids', () {
      final id1 = TraceId.generate();
      final id2 = TraceId.generate();
      expect(id1, isNot(equals(id2)));
    });

    test('parses valid 32-char lowercase hex', () {
      const valid = '4bf92f3577b34da6a3ce929d0e0e4736';
      final id = TraceId.parse(valid);
      expect(id.toString(), valid);
    });

    test('throws on wrong length', () {
      expect(() => TraceId.parse('short'), throwsArgumentError);
    });

    test('throws on uppercase hex', () {
      expect(
        () => TraceId.parse('4BF92F3577B34DA6A3CE929D0E0E4736'),
        throwsArgumentError,
      );
    });
  });

  group('CorrelationContext', () {
    test('generates context with non-empty ids', () {
      final ctx = CorrelationContext.generate();
      expect(ctx.correlationId.isEmpty, isFalse);
      expect(ctx.traceId.isEmpty, isFalse);
    });
  });

  group('toHeaders', () {
    test('converts context to header map', () {
      final ctx = CorrelationContext(
        correlationId: CorrelationId.parse('test-id'),
        traceId: TraceId.parse('4bf92f3577b34da6a3ce929d0e0e4736'),
      );
      final headers = toHeaders(ctx);
      expect(headers[headerCorrelationId], 'test-id');
      expect(headers[headerTraceId], '4bf92f3577b34da6a3ce929d0e0e4736');
    });

    test('omits empty correlation id from headers', () {
      final ctx = CorrelationContext(
        correlationId: CorrelationId.parse(''),
        traceId: TraceId.generate(),
      );
      final headers = toHeaders(ctx);
      expect(headers.containsKey(headerCorrelationId), isFalse);
    });
  });

  group('fromHeaders', () {
    test('extracts existing headers', () {
      final headers = {
        headerCorrelationId: 'existing-id',
        headerTraceId: '4bf92f3577b34da6a3ce929d0e0e4736',
      };
      final ctx = fromHeaders(headers);
      expect(ctx.correlationId.toString(), 'existing-id');
      expect(ctx.traceId.toString(), '4bf92f3577b34da6a3ce929d0e0e4736');
    });

    test('auto-generates missing correlation id', () {
      final ctx = fromHeaders({});
      expect(ctx.correlationId.isEmpty, isFalse);
    });

    test('auto-generates missing trace id', () {
      final ctx = fromHeaders({});
      expect(ctx.traceId.isEmpty, isFalse);
      expect(ctx.traceId.value.length, 32);
    });

    test('auto-generates trace id when invalid', () {
      final headers = {headerTraceId: 'invalid'};
      final ctx = fromHeaders(headers);
      expect(ctx.traceId.value.length, 32);
      expect(ctx.traceId.value, matches(RegExp(r'^[0-9a-f]{32}$')));
    });
  });
}
