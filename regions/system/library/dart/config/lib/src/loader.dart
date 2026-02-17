import 'dart:io';
import 'package:yaml/yaml.dart';
import 'types.dart';
import 'merge.dart';

/// YAML を読み込み Config を返す。envPath があればマージする。
Config loadConfig(String basePath, [String? envPath]) {
  final baseContent = File(basePath).readAsStringSync();
  var yamlMap = yamlToMap(loadYaml(baseContent)) as Map<String, dynamic>;

  if (envPath != null) {
    final envContent = File(envPath).readAsStringSync();
    final envMap = yamlToMap(loadYaml(envContent)) as Map<String, dynamic>;
    yamlMap = deepMerge(yamlMap, envMap);
  }

  return Config.fromYaml(yamlMap);
}

/// 設定値のバリデーション。不正値は ConfigValidationError を投げる。
void validateConfig(Config config) {
  if (config.app.name.isEmpty) {
    throw ConfigValidationError('app.name is required');
  }
  if (config.app.version.isEmpty) {
    throw ConfigValidationError('app.version is required');
  }
  if (!['system', 'business', 'service'].contains(config.app.tier)) {
    throw ConfigValidationError(
        'app.tier must be system, business, or service');
  }
  if (!['dev', 'staging', 'prod'].contains(config.app.environment)) {
    throw ConfigValidationError(
        'app.environment must be dev, staging, or prod');
  }
  if (config.server.host.isEmpty) {
    throw ConfigValidationError('server.host is required');
  }
  if (config.server.port <= 0 || config.server.port > 65535) {
    throw ConfigValidationError('server.port must be between 1 and 65535');
  }
  if (config.auth.jwt.issuer.isEmpty) {
    throw ConfigValidationError('auth.jwt.issuer is required');
  }
  if (config.auth.jwt.audience.isEmpty) {
    throw ConfigValidationError('auth.jwt.audience is required');
  }
}

/// 設定バリデーションエラー。
class ConfigValidationError implements Exception {
  final String message;
  ConfigValidationError(this.message);
  @override
  String toString() => 'ConfigValidationError: $message';
}
