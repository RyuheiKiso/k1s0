import 'dart:io';
import 'package:yaml/yaml.dart';
import 'errors.dart';

class ComponentConfig {
  final String name;
  final String type;
  final String? version;
  final Map<String, String> metadata;

  const ComponentConfig({
    required this.name,
    required this.type,
    this.version,
    this.metadata = const {},
  });

  factory ComponentConfig.fromYaml(Map<dynamic, dynamic> yaml) {
    // 必須フィールドの存在と型を検証する。
    final name = yaml['name'];
    if (name is! String) {
      throw ComponentError(
        component: 'config',
        operation: 'parse',
        message: 'name is required and must be a string, got ${name?.runtimeType}',
      );
    }
    final type = yaml['type'];
    if (type is! String) {
      throw ComponentError(
        component: 'config',
        operation: 'parse',
        message: 'type is required and must be a string, got ${type?.runtimeType}',
      );
    }
    return ComponentConfig(
      name: name,
      type: type,
      version: yaml['version'] as String?,
      metadata: (yaml['metadata'] as Map<dynamic, dynamic>?)
          ?.map((k, v) => MapEntry(k.toString(), v.toString())) ?? {},
    );
  }
}

class ComponentsConfig {
  final List<ComponentConfig> components;

  const ComponentsConfig({required this.components});

  factory ComponentsConfig.fromYaml(String yamlContent) {
    // YAML 構文エラーを ComponentError にラップする。
    final dynamic yaml;
    try {
      yaml = loadYaml(yamlContent);
    } on YamlException catch (e) {
      throw ComponentError(
        component: 'config',
        operation: 'parse',
        message: 'invalid YAML syntax: $e',
      );
    }

    if (yaml is! Map || yaml['components'] is! List) {
      throw const ComponentError(
        component: 'config',
        operation: 'parse',
        message: 'components field is required and must be a list',
      );
    }

    // 各要素の型を検証し、不正な要素は ComponentError として報告する。
    final components = <ComponentConfig>[];
    final list = yaml['components'] as List;
    for (var i = 0; i < list.length; i++) {
      final item = list[i];
      if (item is! Map) {
        throw ComponentError(
          component: 'config',
          operation: 'parse',
          message: 'components[$i] must be a map, got ${item.runtimeType}',
        );
      }
      try {
        components.add(ComponentConfig.fromYaml(item));
      } on TypeError catch (e) {
        throw ComponentError(
          component: 'config',
          operation: 'parse',
          message: 'components[$i] has invalid fields: $e',
        );
      }
    }

    return ComponentsConfig(components: components);
  }

  factory ComponentsConfig.fromFile(String path) {
    final content = File(path).readAsStringSync();
    return ComponentsConfig.fromYaml(content);
  }
}
