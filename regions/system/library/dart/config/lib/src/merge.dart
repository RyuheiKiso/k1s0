import 'package:yaml/yaml.dart';

/// YamlMap / YamlList を通常の Map / List に変換する。
/// leaf 値はそのまま返す。
dynamic yamlToMap(dynamic yaml) {
  if (yaml is YamlMap) {
    return yaml.map((k, v) => MapEntry(k.toString(), yamlToMap(v)));
  }
  if (yaml is YamlList) {
    return yaml.map(yamlToMap).toList();
  }
  return yaml;
}

/// base マップに overlay マップを再帰的にマージする。
Map<String, dynamic> deepMerge(
    Map<String, dynamic> base, Map<String, dynamic> overlay) {
  final result = Map<String, dynamic>.from(base);
  for (final key in overlay.keys) {
    if (result.containsKey(key) &&
        result[key] is Map &&
        overlay[key] is Map) {
      result[key] = deepMerge(
        result[key] as Map<String, dynamic>,
        overlay[key] as Map<String, dynamic>,
      );
    } else {
      result[key] = overlay[key];
    }
  }
  return result;
}
