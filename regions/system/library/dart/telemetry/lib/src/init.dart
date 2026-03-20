import 'dart:convert';

import 'package:logging/logging.dart';
import 'package:opentelemetry/api.dart' as otel_api;
import 'package:opentelemetry/sdk.dart' as otel_sdk;

/// TelemetryConfig は telemetry ライブラリの初期化設定を保持する。
class TelemetryConfig {
  /// サービス名。ログやトレースの識別に使用する。
  final String serviceName;

  /// サービスバージョン。
  final String version;

  /// ティア（system / business / service）。
  final String tier;

  /// デプロイ環境（dev / staging / prod）。
  final String environment;

  /// OTLP トレースエンドポイント。指定時にトレーサーを初期化する。
  final String? traceEndpoint;

  /// サンプリングレート（0.0〜1.0）。デフォルトは全スパン送信。
  final double sampleRate;

  /// ログレベル文字列（debug / info / warn / error）。
  final String logLevel;

  /// ログフォーマット（json / text）。
  final String logFormat;

  TelemetryConfig({
    required this.serviceName,
    required this.version,
    required this.tier,
    required this.environment,
    this.traceEndpoint,
    this.sampleRate = 1.0,
    this.logLevel = 'info',
    this.logFormat = 'json',
  });
}

/// _tracerProvider はモジュールレベルで保持する TracerProvider。
/// shutdown 時にフラッシュ・停止するために保持する。
otel_sdk.TracerProviderBase? _tracerProvider;

/// tracerProvider は現在の TracerProvider を返す。
/// initTelemetry で初期化されていない場合は null。
otel_sdk.TracerProviderBase? get tracerProvider => _tracerProvider;

/// initTelemetry は構造化ログと OpenTelemetry トレーサーの初期化を行う。
/// Logger.root のレベルを設定し、JSON 形式のログ出力を構成する。
/// traceEndpoint が指定されている場合、OTLP エクスポータ付き TracerProvider を作成する。
void initTelemetry(TelemetryConfig cfg) {
  // --- ログ初期化 ---
  Logger.root.level = _parseLevel(cfg.logLevel);
  Logger.root.onRecord.listen((record) {
    if (cfg.logFormat == 'text') {
      // ignore: avoid_print
      print(
          '${record.time} [${record.level.name}] ${record.loggerName}: ${record.message}');
      if (record.error != null) {
        // ignore: avoid_print
        print('  Error: ${record.error}');
      }
    } else {
      final entry = {
        'timestamp': record.time.toUtc().toIso8601String(),
        'level': record.level.name.toLowerCase(),
        'message': record.message,
        'service': cfg.serviceName,
        'version': cfg.version,
        'tier': cfg.tier,
        'environment': cfg.environment,
        'logger': record.loggerName,
      };
      if (record.error != null) entry['error'] = record.error.toString();
      // ignore: avoid_print
      print(jsonEncode(entry));
    }
  });

  // --- トレーサー初期化 ---
  // traceEndpoint が指定されている場合のみ TracerProvider を構成する。
  if (cfg.traceEndpoint != null && cfg.traceEndpoint!.isNotEmpty) {
    _initTracer(cfg);
  }
}

/// _initTracer は OTLP エクスポータ付きの TracerProvider を構成して登録する。
/// リソース属性にサービス名・バージョン・ティア・環境を設定する。
void _initTracer(TelemetryConfig cfg) {
  // OTLP HTTP エクスポータを作成（CollectorExporter は OTLP/HTTP プロトコルを使用する）
  final exporter = otel_sdk.CollectorExporter(
    Uri.parse(cfg.traceEndpoint!),
  );

  // サンプラーを設定する。sampleRate に基づいて AlwaysOn / AlwaysOff / ParentBased を選択する。
  final otel_sdk.Sampler sampler;
  if (cfg.sampleRate >= 1.0) {
    // 全スパンをサンプリング
    sampler = otel_sdk.AlwaysOnSampler();
  } else if (cfg.sampleRate <= 0.0) {
    // スパンをサンプリングしない
    sampler = otel_sdk.AlwaysOffSampler();
  } else {
    // 親スパンの状態に基づくサンプリング。ルートスパンは常にサンプリングし、
    // リモート親がサンプリング済みならサンプリングを継続する。
    sampler = otel_sdk.ParentBasedSampler(otel_sdk.AlwaysOnSampler());
  }

  // リソース属性を構成する（サービス名・バージョン・ティア・環境）
  final resource = otel_sdk.Resource([
    otel_api.Attribute.fromString('service.name', cfg.serviceName),
    otel_api.Attribute.fromString('service.version', cfg.version),
    otel_api.Attribute.fromString('tier', cfg.tier),
    otel_api.Attribute.fromString('environment', cfg.environment),
  ]);

  // BatchSpanProcessor で効率的にスパンをエクスポートする
  final processor = otel_sdk.BatchSpanProcessor(exporter);

  // TracerProvider を作成する
  _tracerProvider = otel_sdk.TracerProviderBase(
    processors: [processor],
    resource: resource,
    sampler: sampler,
  );

  // グローバル TracerProvider として登録する。
  // registerGlobalTracerProvider は一度しか呼べないため、
  // 既に登録済みの場合は StateError を無視する（テスト環境や再初期化時）。
  try {
    otel_api.registerGlobalTracerProvider(_tracerProvider!);
  } on StateError {
    // 既にグローバル TracerProvider が登録済みの場合は無視する。
    // _tracerProvider フィールドは更新済みなので、
    // このライブラリ内のコードは新しい TracerProvider を使用する。
  }
}

/// shutdown は Logger.root のリスナーをクリアし、TracerProvider をシャットダウンする。
/// Go の Provider.Shutdown / Rust の shutdown / TS の shutdown に対応する。
void shutdown() {
  // ログリスナーをクリア
  Logger.root.clearListeners();

  // TracerProvider が存在する場合はシャットダウンしてリソースを解放する
  if (_tracerProvider != null) {
    _tracerProvider!.shutdown();
    _tracerProvider = null;
  }
}

/// _parseLevel はログレベル文字列を logging パッケージの Level に変換する。
Level _parseLevel(String level) {
  switch (level) {
    case 'debug':
      return Level.FINE;
    case 'info':
      return Level.INFO;
    case 'warn':
      return Level.WARNING;
    case 'error':
      return Level.SEVERE;
    default:
      return Level.INFO;
  }
}
