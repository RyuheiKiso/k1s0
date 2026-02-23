import 'package:test/test.dart';
import 'package:k1s0_tracing/tracing.dart';

void main() {
  group('TraceContext', () {
    test('toTraceparent formats correctly', () {
      final ctx = TraceContext(
        traceId: 'a' * 32,
        parentId: 'b' * 16,
        flags: 1,
      );
      expect(
        ctx.toTraceparent(),
        equals('00-${'a' * 32}-${'b' * 16}-01'),
      );
    });

    test('toTraceparent pads flags', () {
      final ctx = TraceContext(
        traceId: '0' * 32,
        parentId: '0' * 16,
        flags: 0,
      );
      expect(ctx.toTraceparent(), endsWith('-00'));
    });

    test('fromTraceparent parses valid string', () {
      final input = '00-${'a' * 32}-${'b' * 16}-01';
      final ctx = TraceContext.fromTraceparent(input);
      expect(ctx, isNotNull);
      expect(ctx!.traceId, equals('a' * 32));
      expect(ctx.parentId, equals('b' * 16));
      expect(ctx.flags, equals(1));
    });

    test('fromTraceparent returns null for invalid version', () {
      final input = '01-${'a' * 32}-${'b' * 16}-01';
      expect(TraceContext.fromTraceparent(input), isNull);
    });

    test('fromTraceparent returns null for wrong part count', () {
      expect(TraceContext.fromTraceparent('00-abc'), isNull);
    });

    test('fromTraceparent returns null for wrong traceId length', () {
      final input = '00-abc-${'b' * 16}-01';
      expect(TraceContext.fromTraceparent(input), isNull);
    });

    test('fromTraceparent returns null for wrong parentId length', () {
      final input = '00-${'a' * 32}-abc-01';
      expect(TraceContext.fromTraceparent(input), isNull);
    });

    test('fromTraceparent returns null for wrong flags length', () {
      final input = '00-${'a' * 32}-${'b' * 16}-1';
      expect(TraceContext.fromTraceparent(input), isNull);
    });

    test('roundtrip', () {
      final original = TraceContext(
        traceId: 'abcd1234' * 4,
        parentId: 'ef567890' * 2,
        flags: 1,
      );
      final parsed = TraceContext.fromTraceparent(original.toTraceparent());
      expect(parsed!.traceId, equals(original.traceId));
      expect(parsed.parentId, equals(original.parentId));
      expect(parsed.flags, equals(original.flags));
    });
  });

  group('Baggage', () {
    test('set and get', () {
      final baggage = Baggage();
      baggage.set('key1', 'value1');
      expect(baggage.get('key1'), equals('value1'));
    });

    test('get returns null for missing key', () {
      final baggage = Baggage();
      expect(baggage.get('missing'), isNull);
    });

    test('toHeader formats correctly', () {
      final baggage = Baggage();
      baggage.set('k1', 'v1');
      baggage.set('k2', 'v2');
      final header = baggage.toHeader();
      expect(header, contains('k1=v1'));
      expect(header, contains('k2=v2'));
    });

    test('fromHeader parses correctly', () {
      final baggage = Baggage.fromHeader('k1=v1,k2=v2');
      expect(baggage.get('k1'), equals('v1'));
      expect(baggage.get('k2'), equals('v2'));
    });

    test('fromHeader handles empty string', () {
      final baggage = Baggage.fromHeader('');
      expect(baggage.entries, isEmpty);
    });

    test('entries returns unmodifiable map', () {
      final baggage = Baggage();
      baggage.set('a', 'b');
      expect(baggage.entries, equals({'a': 'b'}));
    });

    test('roundtrip', () {
      final original = Baggage();
      original.set('tenant', 'acme');
      original.set('region', 'us-east');
      final parsed = Baggage.fromHeader(original.toHeader());
      expect(parsed.get('tenant'), equals('acme'));
      expect(parsed.get('region'), equals('us-east'));
    });
  });

  group('Propagation', () {
    test('injectContext adds traceparent header', () {
      final headers = <String, String>{};
      final ctx = TraceContext(
        traceId: 'a' * 32,
        parentId: 'b' * 16,
        flags: 1,
      );
      injectContext(headers, ctx);
      expect(headers['traceparent'], isNotNull);
      expect(headers.containsKey('baggage'), isFalse);
    });

    test('injectContext adds baggage header', () {
      final headers = <String, String>{};
      final ctx = TraceContext(
        traceId: 'a' * 32,
        parentId: 'b' * 16,
        flags: 1,
      );
      final baggage = Baggage();
      baggage.set('tenant', 'acme');
      injectContext(headers, ctx, baggage);
      expect(headers['traceparent'], isNotNull);
      expect(headers['baggage'], equals('tenant=acme'));
    });

    test('injectContext skips empty baggage', () {
      final headers = <String, String>{};
      final ctx = TraceContext(
        traceId: 'a' * 32,
        parentId: 'b' * 16,
        flags: 1,
      );
      injectContext(headers, ctx, Baggage());
      expect(headers.containsKey('baggage'), isFalse);
    });

    test('extractContext parses headers', () {
      final headers = {
        'traceparent': '00-${'a' * 32}-${'b' * 16}-01',
        'baggage': 'key=val',
      };
      final result = extractContext(headers);
      expect(result.context, isNotNull);
      expect(result.context!.traceId, equals('a' * 32));
      expect(result.baggage.get('key'), equals('val'));
    });

    test('extractContext with missing traceparent', () {
      final result = extractContext({});
      expect(result.context, isNull);
      expect(result.baggage.entries, isEmpty);
    });

    test('roundtrip inject and extract', () {
      final ctx = TraceContext(
        traceId: 'c' * 32,
        parentId: 'd' * 16,
        flags: 0,
      );
      final baggage = Baggage();
      baggage.set('env', 'prod');

      final headers = <String, String>{};
      injectContext(headers, ctx, baggage);
      final result = extractContext(headers);

      expect(result.context!.traceId, equals(ctx.traceId));
      expect(result.context!.parentId, equals(ctx.parentId));
      expect(result.context!.flags, equals(ctx.flags));
      expect(result.baggage.get('env'), equals('prod'));
    });
  });
}
