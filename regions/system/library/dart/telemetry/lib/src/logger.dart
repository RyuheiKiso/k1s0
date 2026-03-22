import 'package:logging/logging.dart';

import 'init.dart';

/// createLogger は TelemetryConfig から Logger インスタンスを生成する。
///
/// initTelemetry で Logger.root の設定が完了している前提で使用する。
/// config の serviceName をロガー名として使用する。
///
/// ```dart
/// final cfg = TelemetryConfig(
///   serviceName: 'TaskService',
///   version: '1.0.0',
///   tier: 'service',
///   environment: 'dev',
/// );
/// final logger = createLogger(cfg);
/// logger.info('Task created');
/// logger.warning('Slow query detected');
/// logger.severe('Failed to assign task');
/// ```
Logger createLogger(TelemetryConfig config) => Logger(config.serviceName);
