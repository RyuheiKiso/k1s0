import 'package:test/test.dart';
import 'package:k1s0_tracing/tracing.dart';

void main() {
  group('TraceContext', () {
    test('toTraceparentが正しい形式で出力されること', () {
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

    test('toTraceparentがフラグをゼロパディングすること', () {
      final ctx = TraceContext(
        traceId: '0' * 32,
        parentId: '0' * 16,
        flags: 0,
      );
      expect(ctx.toTraceparent(), endsWith('-00'));
    });

    test('fromTraceparentが有効な文字列を解析すること', () {
      final input = '00-${'a' * 32}-${'b' * 16}-01';
      final ctx = TraceContext.fromTraceparent(input);
      expect(ctx, isNotNull);
      expect(ctx!.traceId, equals('a' * 32));
      expect(ctx.parentId, equals('b' * 16));
      expect(ctx.flags, equals(1));
    });

    test('fromTraceparentが無効なバージョンでnullを返すこと', () {
      final input = '01-${'a' * 32}-${'b' * 16}-01';
      expect(TraceContext.fromTraceparent(input), isNull);
    });

    test('fromTraceparentがパート数不正でnullを返すこと', () {
      expect(TraceContext.fromTraceparent('00-abc'), isNull);
    });

    test('fromTraceparentがtraceIdの長さ不正でnullを返すこと', () {
      final input = '00-abc-${'b' * 16}-01';
      expect(TraceContext.fromTraceparent(input), isNull);
    });

    test('fromTraceparentがparentIdの長さ不正でnullを返すこと', () {
      final input = '00-${'a' * 32}-abc-01';
      expect(TraceContext.fromTraceparent(input), isNull);
    });

    test('fromTraceparentがフラグの長さ不正でnullを返すこと', () {
      final input = '00-${'a' * 32}-${'b' * 16}-1';
      expect(TraceContext.fromTraceparent(input), isNull);
    });

    test('シリアライズとデシリアライズが往復で一致すること', () {
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
    test('値のセットと取得ができること', () {
      final baggage = Baggage();
      baggage.set('key1', 'value1');
      expect(baggage.get('key1'), equals('value1'));
    });

    test('存在しないキーでnullが返されること', () {
      final baggage = Baggage();
      expect(baggage.get('missing'), isNull);
    });

    test('toHeaderが正しい形式で出力されること', () {
      final baggage = Baggage();
      baggage.set('k1', 'v1');
      baggage.set('k2', 'v2');
      final header = baggage.toHeader();
      expect(header, contains('k1=v1'));
      expect(header, contains('k2=v2'));
    });

    test('fromHeaderが正しく解析されること', () {
      final baggage = Baggage.fromHeader('k1=v1,k2=v2');
      expect(baggage.get('k1'), equals('v1'));
      expect(baggage.get('k2'), equals('v2'));
    });

    test('fromHeaderが空文字列を処理できること', () {
      final baggage = Baggage.fromHeader('');
      expect(baggage.entries, isEmpty);
    });

    test('entriesが変更不可なマップを返すこと', () {
      final baggage = Baggage();
      baggage.set('a', 'b');
      expect(baggage.entries, equals({'a': 'b'}));
    });

    test('シリアライズとデシリアライズが往復で一致すること', () {
      final original = Baggage();
      original.set('tenant', 'acme');
      original.set('region', 'us-east');
      final parsed = Baggage.fromHeader(original.toHeader());
      expect(parsed.get('tenant'), equals('acme'));
      expect(parsed.get('region'), equals('us-east'));
    });
  });

  group('Propagation', () {
    test('injectContextがtraceparentヘッダーを追加すること', () {
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

    test('injectContextがbaggageヘッダーを追加すること', () {
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

    test('injectContextが空のbaggageをスキップすること', () {
      final headers = <String, String>{};
      final ctx = TraceContext(
        traceId: 'a' * 32,
        parentId: 'b' * 16,
        flags: 1,
      );
      injectContext(headers, ctx, Baggage());
      expect(headers.containsKey('baggage'), isFalse);
    });

    test('extractContextがヘッダーを解析すること', () {
      final headers = {
        'traceparent': '00-${'a' * 32}-${'b' * 16}-01',
        'baggage': 'key=val',
      };
      final result = extractContext(headers);
      expect(result.context, isNotNull);
      expect(result.context!.traceId, equals('a' * 32));
      expect(result.baggage.get('key'), equals('val'));
    });

    test('traceparentが存在しない場合にextractContextがnullを返すこと', () {
      final result = extractContext({});
      expect(result.context, isNull);
      expect(result.baggage.entries, isEmpty);
    });

    test('injectとextractの往復で値が一致すること', () {
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
