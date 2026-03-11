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
    return ComponentConfig(
      name: yaml['name'] as String,
      type: yaml['type'] as String,
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
    final yaml = loadYaml(yamlContent);
    if (yaml is! Map || yaml['components'] is! List) {
      throw ComponentError(
        component: 'config',
        operation: 'parse',
        message: 'components field is required and must be a list',
      );
    }
    final components = (yaml['components'] as List)
        .map((c) => ComponentConfig.fromYaml(c as Map))
        .toList();
    return ComponentsConfig(components: components);
  }

  factory ComponentsConfig.fromFile(String path) {
    final content = File(path).readAsStringSync();
    return ComponentsConfig.fromYaml(content);
  }
}
