import 'package:k1s0_correlation/correlation.dart';
import 'package:test/test.dart';

void main() {
  group('CorrelationId', () {
    test('空でないIDを生成すること', () {
      final id = CorrelationId.generate();
      expect(id.value, isNotEmpty);
      expect(id.isEmpty, isFalse);
    });

    test('ユニークなIDを生成すること', () {
      final id1 = CorrelationId.generate();
      final id2 = CorrelationId.generate();
      expect(id1, isNot(equals(id2)));
    });

    test('バリデーションなしで任意の文字列をパースできること', () {
      final id = CorrelationId.parse('custom-id-123');
      expect(id.toString(), 'custom-id-123');
    });

    test('空文字列の場合にisEmptyがtrueを返すこと', () {
      final id = CorrelationId.parse('');
      expect(id.isEmpty, isTrue);
    });
  });

  group('TraceId', () {
    test('32文字のIDを生成すること', () {
      final id = TraceId.generate();
      expect(id.value.length, 32);
    });

    test('小文字16進数のみのIDを生成すること', () {
      final id = TraceId.generate();
      expect(id.value, matches(RegExp(r'^[0-9a-f]{32}$')));
    });

    test('ユニークなIDを生成すること', () {
      final id1 = TraceId.generate();
      final id2 = TraceId.generate();
      expect(id1, isNot(equals(id2)));
    });

    test('有効な32文字小文字16進数をパースできること', () {
      const valid = '4bf92f3577b34da6a3ce929d0e0e4736';
      final id = TraceId.parse(valid);
      expect(id.toString(), valid);
    });

    test('長さが不正な場合に例外をスローすること', () {
      expect(() => TraceId.parse('short'), throwsArgumentError);
    });

    test('大文字16進数の場合に例外をスローすること', () {
      expect(
        () => TraceId.parse('4BF92F3577B34DA6A3CE929D0E0E4736'),
        throwsArgumentError,
      );
    });
  });

  group('CorrelationContext', () {
    test('空でないIDを持つコンテキストを生成すること', () {
      final ctx = CorrelationContext.generate();
      expect(ctx.correlationId.isEmpty, isFalse);
      expect(ctx.traceId.isEmpty, isFalse);
    });
  });

  group('toHeaders', () {
    test('コンテキストをヘッダーマップに変換できること', () {
      final ctx = CorrelationContext(
        correlationId: CorrelationId.parse('test-id'),
        traceId: TraceId.parse('4bf92f3577b34da6a3ce929d0e0e4736'),
      );
      final headers = toHeaders(ctx);
      expect(headers[headerCorrelationId], 'test-id');
      expect(headers[headerTraceId], '4bf92f3577b34da6a3ce929d0e0e4736');
    });

    test('空のCorrelationIDはヘッダーに含まれないこと', () {
      final ctx = CorrelationContext(
        correlationId: CorrelationId.parse(''),
        traceId: TraceId.generate(),
      );
      final headers = toHeaders(ctx);
      expect(headers.containsKey(headerCorrelationId), isFalse);
    });
  });

  group('fromHeaders', () {
    test('既存のヘッダーから値を抽出できること', () {
      final headers = {
        headerCorrelationId: 'existing-id',
        headerTraceId: '4bf92f3577b34da6a3ce929d0e0e4736',
      };
      final ctx = fromHeaders(headers);
      expect(ctx.correlationId.toString(), 'existing-id');
      expect(ctx.traceId.toString(), '4bf92f3577b34da6a3ce929d0e0e4736');
    });

    test('CorrelationIDが欠落している場合に自動生成すること', () {
      final ctx = fromHeaders({});
      expect(ctx.correlationId.isEmpty, isFalse);
    });

    test('TraceIDが欠落している場合に自動生成すること', () {
      final ctx = fromHeaders({});
      expect(ctx.traceId.isEmpty, isFalse);
      expect(ctx.traceId.value.length, 32);
    });

    test('TraceIDが不正な場合に自動生成すること', () {
      final headers = {headerTraceId: 'invalid'};
      final ctx = fromHeaders(headers);
      expect(ctx.traceId.value.length, 32);
      expect(ctx.traceId.value, matches(RegExp(r'^[0-9a-f]{32}$')));
    });
  });
}
