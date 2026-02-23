import 'trace_context.dart';
import 'baggage.dart';

void injectContext(
  Map<String, String> headers,
  TraceContext ctx, [
  Baggage? baggage,
]) {
  headers['traceparent'] = ctx.toTraceparent();
  if (baggage != null) {
    final header = baggage.toHeader();
    if (header.isNotEmpty) {
      headers['baggage'] = header;
    }
  }
}

({TraceContext? context, Baggage baggage}) extractContext(
  Map<String, String> headers,
) {
  final traceparent = headers['traceparent'];
  final ctx = traceparent != null
      ? TraceContext.fromTraceparent(traceparent)
      : null;

  final baggageHeader = headers['baggage'];
  final baggage = baggageHeader != null
      ? Baggage.fromHeader(baggageHeader)
      : Baggage();

  return (context: ctx, baggage: baggage);
}
