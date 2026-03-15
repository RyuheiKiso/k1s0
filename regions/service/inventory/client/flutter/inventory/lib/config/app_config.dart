import 'package:flutter/services.dart';
import 'package:yaml/yaml.dart';

/// API接続設定を保持するクラス
/// ベースURL、タイムアウト、リトライ設定をまとめて管理する
class ApiConfig {
  /// APIのベースURL
  final String baseUrl;

  /// リクエストタイムアウト（秒）
  final int timeout;

  /// リトライ最大回数
  final int retryMaxAttempts;

  /// リトライ時のバックオフ時間（ミリ秒）
  final int retryBackoffMs;

  const ApiConfig({
    required this.baseUrl,
    required this.timeout,
    required this.retryMaxAttempts,
    required this.retryBackoffMs,
  });
}

/// アプリケーション全体の設定を保持するクラス
/// YAMLファイルから読み込んだ設定を型安全に保持する
class AppConfig {
  /// アプリケーション名
  final String appName;

  /// アプリケーションバージョン
  final String version;

  /// 実行環境（development, staging, production）
  final String env;

  /// API接続設定
  final ApiConfig api;

  /// フィーチャーフラグ
  final Map<String, bool> features;

  const AppConfig({
    required this.appName,
    required this.version,
    required this.env,
    required this.api,
    required this.features,
  });

  /// 指定された環境の設定をYAMLアセットから読み込む
  /// ベース設定と環境別設定をディープマージして返す
  static Future<AppConfig> load(String env) async {
    /// ベース設定ファイルを読み込む
    final baseYaml = await rootBundle.loadString('config/config.yaml');
    final baseMap = _yamlToMap(loadYaml(baseYaml));

    /// 環境別オーバーレイ設定が存在する場合のみ読み込んでマージする
    Map<String, dynamic> merged;
    try {
      final overlayYaml = await rootBundle.loadString('config/config.$env.yaml');
      final overlayMap = _yamlToMap(loadYaml(overlayYaml));
      merged = _deepMerge(baseMap, overlayMap);
    } catch (_) {
      merged = baseMap;
    }

    final app = merged['app'] as Map<String, dynamic>;
    final api = merged['api'] as Map<String, dynamic>;
    final retry = api['retry'] as Map<String, dynamic>;
    final features = (merged['features'] as Map<String, dynamic>?)
            ?.map((k, v) => MapEntry(k, v as bool)) ??
        {};

    return AppConfig(
      appName: app['name'] as String,
      version: app['version'] as String,
      env: app['env'] as String,
      api: ApiConfig(
        baseUrl: api['base_url'] as String,
        timeout: api['timeout'] as int,
        retryMaxAttempts: retry['max_attempts'] as int,
        retryBackoffMs: retry['backoff_ms'] as int,
      ),
      features: features,
    );
  }

  /// YamlMapをMap<String, dynamic>に再帰的に変換する
  /// yamlパッケージの型をDartの標準Map型に統一する
  static Map<String, dynamic> _yamlToMap(dynamic yaml) {
    if (yaml is YamlMap) {
      return yaml.map((k, v) => MapEntry(k.toString(), _yamlToMap(v)));
    }
    if (yaml is YamlList) {
      return yaml.asMap().map((k, v) => MapEntry(k.toString(), _yamlToMap(v)));
    }
    return yaml is Map
        ? yaml.map((k, v) => MapEntry(k.toString(), _yamlToMap(v)))
        : yaml;
  }

  /// 2つのMapを再帰的にディープマージする
  /// overlay側の値でbase側の値を上書きし、ネストされたMapは再帰的にマージする
  static Map<String, dynamic> _deepMerge(
    Map<String, dynamic> base,
    Map<String, dynamic> overlay,
  ) {
    final result = Map<String, dynamic>.from(base);
    for (final key in overlay.keys) {
      if (result.containsKey(key) &&
          result[key] is Map<String, dynamic> &&
          overlay[key] is Map<String, dynamic>) {
        result[key] = _deepMerge(
          result[key] as Map<String, dynamic>,
          overlay[key] as Map<String, dynamic>,
        );
      } else {
        result[key] = overlay[key];
      }
    }
    return result;
  }
}
