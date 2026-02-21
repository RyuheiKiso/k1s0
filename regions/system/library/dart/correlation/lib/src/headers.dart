import 'types.dart';

/// HTTP ヘッダー定数
const headerCorrelationId = 'X-Correlation-Id';
const headerTraceId = 'X-Trace-Id';

/// toHeaders は CorrelationContext を HTTP ヘッダーマップに変換する。
Map<String, String> toHeaders(CorrelationContext ctx) {
  final headers = <String, String>{};
  if (!ctx.correlationId.isEmpty) {
    headers[headerCorrelationId] = ctx.correlationId.toString();
  }
  if (!ctx.traceId.isEmpty) {
    headers[headerTraceId] = ctx.traceId.toString();
  }
  return headers;
}

/// fromHeaders は HTTP ヘッダーマップから CorrelationContext を生成する。
/// ヘッダーが存在しない場合は自動生成する。
CorrelationContext fromHeaders(Map<String, String> headers) {
  CorrelationId correlationId;
  TraceId traceId;

  final corrHeader = headers[headerCorrelationId];
  if (corrHeader != null && corrHeader.isNotEmpty) {
    correlationId = CorrelationId.parse(corrHeader);
  } else {
    correlationId = CorrelationId.generate();
  }

  final traceHeader = headers[headerTraceId];
  if (traceHeader != null && traceHeader.isNotEmpty) {
    try {
      traceId = TraceId.parse(traceHeader);
    } catch (_) {
      traceId = TraceId.generate();
    }
  } else {
    traceId = TraceId.generate();
  }

  return CorrelationContext(correlationId: correlationId, traceId: traceId);
}
