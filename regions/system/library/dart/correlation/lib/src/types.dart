import 'package:uuid/uuid.dart';

const _uuid = Uuid();

/// CorrelationId は分散トレーシングのリクエスト相関 ID。
/// UUID v4 文字列ラッパー（バリデーションなし）。
class CorrelationId {
  final String value;

  const CorrelationId._(this.value);

  factory CorrelationId.generate() {
    return CorrelationId._(_uuid.v4());
  }

  factory CorrelationId.parse(String s) {
    return CorrelationId._(s);
  }

  bool get isEmpty => value.isEmpty;

  @override
  String toString() => value;

  @override
  bool operator ==(Object other) =>
      other is CorrelationId && other.value == value;

  @override
  int get hashCode => value.hashCode;
}

/// TraceId は OpenTelemetry 互換の 32 文字小文字 hex トレース ID。
class TraceId {
  final String value;

  const TraceId._(this.value);

  factory TraceId.generate() {
    final raw = _uuid.v4().replaceAll('-', '');
    return TraceId._(raw);
  }

  factory TraceId.parse(String s) {
    if (s.length != 32) {
      throw ArgumentError(
        'Invalid trace id length: expected 32, got ${s.length}',
      );
    }
    final validHex = RegExp(r'^[0-9a-f]{32}$');
    if (!validHex.hasMatch(s)) {
      throw ArgumentError(
        'Invalid trace id: must be 32 lowercase hex characters',
      );
    }
    return TraceId._(s);
  }

  bool get isEmpty => value.isEmpty;

  @override
  String toString() => value;

  @override
  bool operator ==(Object other) => other is TraceId && other.value == value;

  @override
  int get hashCode => value.hashCode;
}

/// CorrelationContext は CorrelationId と TraceId を保持するコンテキスト。
class CorrelationContext {
  final CorrelationId correlationId;
  final TraceId traceId;

  const CorrelationContext({
    required this.correlationId,
    required this.traceId,
  });

  factory CorrelationContext.generate() {
    return CorrelationContext(
      correlationId: CorrelationId.generate(),
      traceId: TraceId.generate(),
    );
  }
}
