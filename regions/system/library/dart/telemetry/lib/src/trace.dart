import 'package:opentelemetry/api.dart' as otel_api;

import 'init.dart';

/// withTrace は非同期関数をトレーススパンでラップして実行する汎用ヘルパー。
/// [tracerName] はスパンの計装名（通常はサービス名またはパッケージ名）。
/// [spanName] はオペレーション名。
/// [fn] はスパン内で実行する非同期関数。スパンオブジェクトが引数として渡される。
/// fn がエラーをスローした場合、スパンにエラーを記録してから再スローする。
/// TypeScript の withTrace に対応する。
Future<T> withTrace<T>(
  String tracerName,
  String spanName,
  Future<T> Function(otel_api.Span span) fn,
) async {
  // TracerProvider から Tracer を取得する（ローカル優先、なければグローバル）
  final provider = tracerProvider ?? otel_api.globalTracerProvider;
  final tracer = provider.getTracer(tracerName);

  // スパンを開始する
  final span = tracer.startSpan(spanName);

  try {
    final result = await fn(span);
    // 成功時にスパンステータスを OK に設定する
    span.setStatus(otel_api.StatusCode.ok);
    return result;
  } catch (e, stackTrace) {
    // エラーをスパンに記録してから再スローする
    span.recordException(e, stackTrace: stackTrace);
    span.setStatus(otel_api.StatusCode.error, e.toString());
    rethrow;
  } finally {
    // スパンを終了する（成功・失敗にかかわらず必ず呼ぶ）
    span.end();
  }
}

/// withTraceSync は同期関数をトレーススパンでラップして実行する汎用ヘルパー。
/// [tracerName] はスパンの計装名。
/// [spanName] はオペレーション名。
/// [fn] はスパン内で実行する同期関数。
T withTraceSync<T>(
  String tracerName,
  String spanName,
  T Function(otel_api.Span span) fn,
) {
  // TracerProvider から Tracer を取得する（ローカル優先、なければグローバル）
  final provider = tracerProvider ?? otel_api.globalTracerProvider;
  final tracer = provider.getTracer(tracerName);

  // スパンを開始する
  final span = tracer.startSpan(spanName);

  try {
    final result = fn(span);
    // 成功時にスパンステータスを OK に設定する
    span.setStatus(otel_api.StatusCode.ok);
    return result;
  } catch (e, stackTrace) {
    // エラーをスパンに記録してから再スローする
    span.recordException(e, stackTrace: stackTrace);
    span.setStatus(otel_api.StatusCode.error, e.toString());
    rethrow;
  } finally {
    // スパンを終了する
    span.end();
  }
}

/// getCurrentSpan は現在のコンテキストからアクティブなスパンを返す。
/// スパンが存在しない場合は null を返す。
/// TypeScript の getCurrentSpan に対応する。
otel_api.Span? getCurrentSpan() {
  // Context API からアクティブスパンを取得する（非推奨の .span ではなく spanFromContext を使用）
  final span = otel_api.spanFromContext(otel_api.Context.current);
  // spanFromContext は常に非 null を返すが、SpanContext が invalid なら実質無効スパン
  if (!span.spanContext.isValid) {
    return null;
  }
  return span;
}

/// addSpanAttribute は指定されたスパン（または現在のスパン）に属性を追加する。
/// [key] は属性キー。
/// [value] は属性値（String / int / double / bool に対応）。
/// [span] が指定されない場合は現在のアクティブスパンに追加する。
/// スパンが存在しない場合は何もしない。
/// TypeScript の addSpanAttribute に対応する。
void addSpanAttribute(String key, Object value, {otel_api.Span? span}) {
  // 対象スパンを決定する（引数優先、なければ現在のスパン）
  final targetSpan = span ?? getCurrentSpan();
  if (targetSpan == null) return;

  // 型に応じて適切な Attribute を設定する
  if (value is String) {
    targetSpan.setAttribute(otel_api.Attribute.fromString(key, value));
  } else if (value is int) {
    targetSpan.setAttribute(otel_api.Attribute.fromInt(key, value));
  } else if (value is double) {
    targetSpan.setAttribute(otel_api.Attribute.fromDouble(key, value));
  } else if (value is bool) {
    targetSpan.setAttribute(otel_api.Attribute.fromBoolean(key, value));
  } else {
    // その他の型は toString() で文字列化する
    targetSpan
        .setAttribute(otel_api.Attribute.fromString(key, value.toString()));
  }
}
