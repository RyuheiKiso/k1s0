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

/// 設定値のバリデーション。Config.validate() に委譲する。
/// 後方互換性のためにスタンドアロン関数として残す。
void validateConfig(Config config) {
  config.validate();
}
