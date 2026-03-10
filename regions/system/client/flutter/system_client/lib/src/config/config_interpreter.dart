import 'dart:convert';

import 'package:dio/dio.dart';

import 'config_types.dart';

class ServiceConfigEntry {
  const ServiceConfigEntry({
    required this.namespace,
    required this.key,
    required this.value,
    required this.version,
  });

  final String namespace;
  final String key;
  final dynamic value;
  final int version;

  String get id => '$namespace::$key';

  factory ServiceConfigEntry.fromJson(Map<String, dynamic> json) =>
      ServiceConfigEntry(
        namespace: json['namespace'] as String,
        key: json['key'] as String,
        value: json['value'],
        version: (json['version'] as num?)?.toInt() ?? 0,
      );
}

class ServiceConfigResult {
  const ServiceConfigResult({
    required this.serviceName,
    required this.entries,
  });

  final String serviceName;
  final List<ServiceConfigEntry> entries;

  factory ServiceConfigResult.fromJson(Map<String, dynamic> json) =>
      ServiceConfigResult(
        serviceName: json['service_name'] as String,
        entries: (json['entries'] as List<dynamic>? ?? [])
            .map((entry) =>
                ServiceConfigEntry.fromJson(entry as Map<String, dynamic>))
            .toList(),
      );
}

class ConfigFieldState {
  const ConfigFieldState({
    required this.id,
    required this.namespace,
    required this.key,
    required this.schema,
    required this.value,
    required this.originalValue,
    required this.version,
    required this.originalVersion,
    required this.isDirty,
    this.error,
  });

  final String id;
  final String namespace;
  final String key;
  final ConfigFieldSchema schema;
  final dynamic value;
  final dynamic originalValue;
  final int version;
  final int originalVersion;
  final bool isDirty;
  final String? error;

  ConfigFieldState copyWith({
    String? id,
    String? namespace,
    String? key,
    ConfigFieldSchema? schema,
    dynamic value,
    dynamic originalValue,
    int? version,
    int? originalVersion,
    bool? isDirty,
    String? error,
    bool clearError = false,
  }) {
    return ConfigFieldState(
      id: id ?? this.id,
      namespace: namespace ?? this.namespace,
      key: key ?? this.key,
      schema: schema ?? this.schema,
      value: value ?? this.value,
      originalValue: originalValue ?? this.originalValue,
      version: version ?? this.version,
      originalVersion: originalVersion ?? this.originalVersion,
      isDirty: isDirty ?? this.isDirty,
      error: clearError ? null : (error ?? this.error),
    );
  }
}

class ConfigCategoryState {
  const ConfigCategoryState({
    required this.schema,
    required this.fields,
  });

  final ConfigCategorySchema schema;
  final List<ConfigFieldState> fields;

  ConfigCategoryState copyWith({
    ConfigCategorySchema? schema,
    List<ConfigFieldState>? fields,
  }) {
    return ConfigCategoryState(
      schema: schema ?? this.schema,
      fields: fields ?? this.fields,
    );
  }
}

class ConfigData {
  const ConfigData({
    required this.service,
    required this.categories,
    required this.dirtyCount,
  });

  final String service;
  final List<ConfigCategoryState> categories;
  final int dirtyCount;

  ConfigData copyWith({
    String? service,
    List<ConfigCategoryState>? categories,
    int? dirtyCount,
  }) {
    return ConfigData(
      service: service ?? this.service,
      categories: categories ?? this.categories,
      dirtyCount: dirtyCount ?? this.dirtyCount,
    );
  }
}

class ConfigInterpreter {
  const ConfigInterpreter({required this.dio});

  final Dio dio;

  Future<ConfigData> build(String serviceName) async {
    final results = await Future.wait([
      dio.get<Map<String, dynamic>>('/api/v1/config-schema/$serviceName'),
      dio.get<Map<String, dynamic>>('/api/v1/config/services/$serviceName'),
    ]);

    final schema = ConfigEditorSchema.fromJson(results[0].data!);
    final serviceConfig = ServiceConfigResult.fromJson(results[1].data!);
    final entryMap = {
      for (final entry in serviceConfig.entries) entry.id: entry,
    };

    final categories = schema.categories.map((category) {
      final fields = category.fields.map((field) {
        final existing = _findEntry(category.namespaces, field.key, entryMap);
        final namespace = existing?.namespace ??
            (category.namespaces.isNotEmpty
                ? category.namespaces.first
                : schema.namespacePrefix);
        final currentValue = existing?.value ?? field.defaultValue;
        return ConfigFieldState(
          id: buildFieldId(namespace, field.key),
          namespace: namespace,
          key: field.key,
          schema: field,
          value: currentValue,
          originalValue: currentValue,
          version: existing?.version ?? 0,
          originalVersion: existing?.version ?? 0,
          isDirty: false,
          error: validateFieldValue(field, currentValue),
        );
      }).toList();

      return ConfigCategoryState(schema: category, fields: fields);
    }).toList();

    return ConfigData(
      service: serviceConfig.serviceName,
      categories: categories,
      dirtyCount: 0,
    );
  }
}

ServiceConfigEntry? _findEntry(
  List<String> namespaces,
  String key,
  Map<String, ServiceConfigEntry> entries,
) {
  for (final namespace in namespaces) {
    final entry = entries[buildFieldId(namespace, key)];
    if (entry != null) {
      return entry;
    }
  }
  return null;
}

String buildFieldId(String namespace, String key) => '$namespace::$key';

String? validateFieldValue(ConfigFieldSchema schema, dynamic value) {
  switch (schema.type) {
    case ConfigFieldType.integer:
      if (value is! int) return 'Enter an integer';
      return _validateNumberRange(schema, value);
    case ConfigFieldType.float:
      if (value is! num) return 'Enter a number';
      return _validateNumberRange(schema, value);
    case ConfigFieldType.boolean:
      return value is bool ? null : 'Enter a boolean';
    case ConfigFieldType.enumType:
      if (value is! String) return 'Select a value';
      if (schema.options != null && !schema.options!.contains(value)) {
        return 'Select a valid option';
      }
      return null;
    case ConfigFieldType.object:
      return value is Map<String, dynamic> ? null : 'Enter an object';
    case ConfigFieldType.array:
      return value is List ? null : 'Enter an array';
    case ConfigFieldType.string:
      if (value is! String) return 'Enter a string';
      if (schema.pattern != null &&
          value.isNotEmpty &&
          !RegExp(schema.pattern!).hasMatch(value)) {
        return 'Must match ${schema.pattern}';
      }
      return null;
  }
}

bool isSameValue(dynamic left, dynamic right) {
  return jsonEncode(left) == jsonEncode(right);
}

ConfigFieldState updateFieldState(ConfigFieldState field, dynamic nextValue) {
  return field.copyWith(
    value: nextValue,
    isDirty: !isSameValue(nextValue, field.originalValue),
    error: validateFieldValue(field.schema, nextValue),
  );
}

ConfigData resetConfigData(ConfigData data) {
  final categories = data.categories.map((category) {
    return category.copyWith(
      fields: category.fields.map((field) {
        return field.copyWith(
          value: field.originalValue,
          version: field.originalVersion,
          isDirty: false,
          error: validateFieldValue(field.schema, field.originalValue),
        );
      }).toList(),
    );
  }).toList();

  return data.copyWith(categories: categories, dirtyCount: 0);
}

int countDirtyFields(List<ConfigCategoryState> categories) {
  return categories.fold<int>(
    0,
    (sum, category) =>
        sum + category.fields.where((field) => field.isDirty).length,
  );
}

String? _validateNumberRange(ConfigFieldSchema schema, num value) {
  if (schema.min != null && value < schema.min!) {
    return 'Must be >= ${schema.min}';
  }
  if (schema.max != null && value > schema.max!) {
    return 'Must be <= ${schema.max}';
  }
  return null;
}
