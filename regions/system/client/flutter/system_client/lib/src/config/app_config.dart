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
    final baseMap = _yamlToNative(loadYaml(baseYaml));

    /// 環境別オーバーレイ設定が存在する場合のみ読み込んでマージする
    Map<String, dynamic> merged;
    try {
      final overlayYaml =
          await rootBundle.loadString('config/config.$env.yaml');
      final overlayMap = _yamlToNative(loadYaml(overlayYaml));
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

    /// base_urlが未設定の場合はlocalhostフォールバックせず即座にエラーで失敗する
    final baseUrl = api['base_url'] as String?;
    if (baseUrl == null || baseUrl.isEmpty) {
      throw StateError(
        'api.base_url が設定されていません。環境別設定ファイル (config.$env.yaml) に base_url を指定してください。',
      );
    }

    return AppConfig(
      appName: app['name'] as String,
      version: app['version'] as String,
      env: app['env'] as String,
      api: ApiConfig(
        baseUrl: baseUrl,
        timeout: api['timeout'] as int,
        retryMaxAttempts: retry['max_attempts'] as int,
        retryBackoffMs: retry['backoff_ms'] as int,
      ),
      features: features,
    );
  }

  /// `YamlMap` を `Map<String, dynamic>` に、`YamlList` を `List<dynamic>` に再帰的に変換する
  /// yaml パッケージの型を Dart の標準コレクション型に統一する
  static dynamic _yamlToNative(dynamic yaml) {
    if (yaml is YamlMap) {
      return Map<String, dynamic>.fromEntries(
        yaml.entries
            .map((e) => MapEntry(e.key.toString(), _yamlToNative(e.value))),
      );
    }
    // YamlList はリストとして返す（Map への誤変換を防止）
    if (yaml is YamlList) {
      return yaml.map((v) => _yamlToNative(v)).toList();
    }
    if (yaml is Map) {
      return Map<String, dynamic>.fromEntries(
        yaml.entries
            .map((e) => MapEntry(e.key.toString(), _yamlToNative(e.value))),
      );
    }
    return yaml;
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
