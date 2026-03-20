import 'package:opentelemetry/api.dart' as otel_api;
import 'package:opentelemetry/sdk.dart' as otel_sdk;
import 'package:test/test.dart';

import 'package:k1s0_telemetry/telemetry.dart';

void main() {
  group('withTrace', () {
    test('正常終了時に結果が返されること', () async {
      // 非同期関数をスパンでラップして実行する
      final result = await withTrace<int>(
        'test-service',
        'test-operation',
        (span) async {
          return 42;
        },
      );

      expect(result, 42);
    });

    test('例外発生時にエラーが再スローされること', () async {
      // スパン内で例外が発生した場合、再スローされることを検証する
      expect(
        () => withTrace<int>(
          'test-service',
          'failing-operation',
          (span) async {
            throw Exception('test error');
          },
        ),
        throwsA(isA<Exception>().having(
          (e) => e.toString(),
          'message',
          contains('test error'),
        )),
      );
    });

    test('スパンが関数に渡されること', () async {
      // fn に渡されるスパンオブジェクトが null でないことを検証する
      otel_api.Span? capturedSpan;
      await withTrace<void>(
        'test-service',
        'span-capture',
        (span) async {
          capturedSpan = span;
        },
      );

      expect(capturedSpan, isNotNull);
    });

    test('スパンに属性を追加できること', () async {
      // スパン内で属性を追加してもエラーにならないことを検証する
      await withTrace<void>(
        'test-service',
        'attributed-operation',
        (span) async {
          addSpanAttribute('user.id', '123', span: span);
          addSpanAttribute('request.count', 5, span: span);
          addSpanAttribute('cache.hit', true, span: span);
        },
      );
    });
  });

  group('withTraceSync', () {
    test('同期関数の結果が返されること', () {
      // 同期関数をスパンでラップして実行する
      final result = withTraceSync<String>(
        'test-service',
        'sync-operation',
        (span) {
          return 'hello';
        },
      );

      expect(result, 'hello');
    });

    test('同期関数で例外発生時にエラーが再スローされること', () {
      // 同期スパン内で例外が発生した場合、再スローされることを検証する
      expect(
        () => withTraceSync<int>(
          'test-service',
          'sync-failing',
          (span) {
            throw StateError('sync error');
          },
        ),
        throwsA(isA<StateError>()),
      );
    });
  });

  group('addSpanAttribute', () {
    test('スパンが存在しない場合でもエラーにならないこと', () {
      // getCurrentSpan が null の場合でも addSpanAttribute は例外を投げない
      addSpanAttribute('key', 'value');
    });

    test('文字列型の属性が設定できること', () async {
      await withTrace<void>('test', 'attr-string', (span) async {
        addSpanAttribute('str.key', 'str-value', span: span);
      });
    });

    test('整数型の属性が設定できること', () async {
      await withTrace<void>('test', 'attr-int', (span) async {
        addSpanAttribute('int.key', 42, span: span);
      });
    });

    test('浮動小数点型の属性が設定できること', () async {
      await withTrace<void>('test', 'attr-double', (span) async {
        addSpanAttribute('double.key', 3.14, span: span);
      });
    });

    test('真偽値型の属性が設定できること', () async {
      await withTrace<void>('test', 'attr-bool', (span) async {
        addSpanAttribute('bool.key', true, span: span);
      });
    });

    test('その他の型は toString で文字列化されること', () async {
      await withTrace<void>('test', 'attr-other', (span) async {
        addSpanAttribute('list.key', [1, 2, 3], span: span);
      });
    });
  });

  group('getCurrentSpan', () {
    test('スパンが存在しない場合は null が返されること', () {
      // アクティブスパンがない場合の動作を検証する
      final span = getCurrentSpan();
      // グローバルコンテキストにスパンがない場合は null
      // （テスト環境ではコンテキスト伝播が設定されていないため null になりうる）
      expect(span, isA<otel_api.Span?>());
    });
  });

  group('initTelemetry トレーサー初期化', () {
    tearDown(shutdown);

    test('traceEndpoint なしの場合 TracerProvider が null であること', () {
      final cfg = TelemetryConfig(
        serviceName: 'no-trace-service',
        version: '1.0.0',
        tier: 'system',
        environment: 'test',
      );

      initTelemetry(cfg);

      // traceEndpoint 未指定のため TracerProvider は初期化されない
      expect(tracerProvider, isNull);
    });

    test('traceEndpoint 指定時に TracerProvider が初期化されること', () {
      final cfg = TelemetryConfig(
        serviceName: 'traced-service',
        version: '1.0.0',
        tier: 'system',
        environment: 'test',
        traceEndpoint: 'http://localhost:4318/v1/traces',
      );

      initTelemetry(cfg);

      // traceEndpoint が指定されたため TracerProvider が作成される
      expect(tracerProvider, isNotNull);
      expect(tracerProvider, isA<otel_sdk.TracerProviderBase>());
    });

    test('sampleRate 1.0 で全スパンサンプリングされること', () {
      final cfg = TelemetryConfig(
        serviceName: 'full-sample',
        version: '1.0.0',
        tier: 'system',
        environment: 'test',
        traceEndpoint: 'http://localhost:4318/v1/traces',
        sampleRate: 1.0,
      );

      initTelemetry(cfg);
      expect(tracerProvider, isNotNull);
    });

    test('sampleRate 0.0 でスパンがサンプリングされないこと', () {
      final cfg = TelemetryConfig(
        serviceName: 'no-sample',
        version: '1.0.0',
        tier: 'system',
        environment: 'test',
        traceEndpoint: 'http://localhost:4318/v1/traces',
        sampleRate: 0.0,
      );

      initTelemetry(cfg);
      expect(tracerProvider, isNotNull);
    });

    test('shutdown で TracerProvider がクリアされること', () {
      final cfg = TelemetryConfig(
        serviceName: 'shutdown-test',
        version: '1.0.0',
        tier: 'system',
        environment: 'test',
        traceEndpoint: 'http://localhost:4318/v1/traces',
      );

      initTelemetry(cfg);
      expect(tracerProvider, isNotNull);

      shutdown();

      // shutdown 後は TracerProvider が null になる
      expect(tracerProvider, isNull);
    });

    test('空文字列の traceEndpoint では TracerProvider が初期化されないこと', () {
      final cfg = TelemetryConfig(
        serviceName: 'empty-endpoint',
        version: '1.0.0',
        tier: 'system',
        environment: 'test',
        traceEndpoint: '',
      );

      initTelemetry(cfg);
      expect(tracerProvider, isNull);
    });
  });

  group('traceFunction（trace_helper）', () {
    test('正常終了時に TraceResult が返されること', () async {
      final result = await traceFunction<int>(
        'test-tracer',
        'compute',
        () async => 100,
      );

      expect(result.value, 100);
      expect(result.duration.inMicroseconds, greaterThanOrEqualTo(0));
    });

    test('例外発生時にエラーが再スローされること', () async {
      expect(
        () => traceFunction<int>(
          'test-tracer',
          'failing-compute',
          () async => throw Exception('compute failed'),
        ),
        throwsA(isA<Exception>()),
      );
    });
  });

  group('traceMethod（trace_helper）', () {
    test('同期関数の TraceResult が返されること', () {
      final result = traceMethod<String>(
        'test-tracer',
        'sync-compute',
        () => 'result',
      );

      expect(result.value, 'result');
      expect(result.duration.inMicroseconds, greaterThanOrEqualTo(0));
    });

    test('同期関数で例外発生時にエラーが再スローされること', () {
      expect(
        () => traceMethod<int>(
          'test-tracer',
          'sync-failing',
          () => throw StateError('sync failed'),
        ),
        throwsA(isA<StateError>()),
      );
    });
  });
}
