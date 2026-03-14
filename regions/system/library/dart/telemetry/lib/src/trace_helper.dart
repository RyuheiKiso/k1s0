import 'package:logging/logging.dart';

// _logger はトレースヘルパーの内部ロガー。
final _logger = Logger('telemetry.trace');

/// TraceResult はトレース実行の結果を保持する汎用クラス。
class TraceResult<T> {
  final T value;
  final Duration duration;

  const TraceResult({required this.value, required this.duration});
}

/// traceFunction は非同期関数の実行時間を計測してログに記録するヘルパー。
/// [tracerName] はログの計装名（通常はサービス名またはパッケージ名）。
/// [spanName] はオペレーション名。
/// [fn] は計測対象の非同期関数。
/// エラー発生時はスタックトレース付きでログを記録し、例外を再スローする。
Future<TraceResult<T>> traceFunction<T>(
  String tracerName,
  String spanName,
  Future<T> Function() fn,
) async {
  final start = DateTime.now();
  _logger.info('[$tracerName] $spanName: start');

  try {
    final result = await fn();
    final duration = DateTime.now().difference(start);
    _logger.info('[$tracerName] $spanName: ok [${duration.inMilliseconds}ms]');
    return TraceResult(value: result, duration: duration);
  } catch (e, stackTrace) {
    final duration = DateTime.now().difference(start);
    // エラーをログに記録してから再スローする（呼び出し元で処理できるようにする）
    _logger.severe(
      '[$tracerName] $spanName: error [${duration.inMilliseconds}ms]',
      e,
      stackTrace,
    );
    rethrow;
  }
}

/// traceMethod は同期関数の実行時間を計測してログに記録するヘルパー。
/// [tracerName] はログの計装名。
/// [spanName] はオペレーション名。
/// [fn] は計測対象の同期関数。
TraceResult<T> traceMethod<T>(
  String tracerName,
  String spanName,
  T Function() fn,
) {
  final start = DateTime.now();
  _logger.info('[$tracerName] $spanName: start');

  try {
    final result = fn();
    final duration = DateTime.now().difference(start);
    _logger.info('[$tracerName] $spanName: ok [${duration.inMilliseconds}ms]');
    return TraceResult(value: result, duration: duration);
  } catch (e, stackTrace) {
    final duration = DateTime.now().difference(start);
    _logger.severe(
      '[$tracerName] $spanName: error [${duration.inMilliseconds}ms]',
      e,
      stackTrace,
    );
    rethrow;
  }
}
