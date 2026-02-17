import 'package:logging/logging.dart';

/// createLogger は指定された名前で Logger インスタンスを生成する。
///
/// initTelemetry で Logger.root の設定が完了している前提で使用する。
///
/// ```dart
/// final logger = createLogger('OrderService');
/// logger.info('Order created');
/// logger.warning('Slow query detected');
/// logger.severe('Failed to process payment');
/// ```
Logger createLogger(String name) => Logger(name);
