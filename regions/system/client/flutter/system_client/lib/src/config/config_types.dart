enum ConfigFieldType {
  string,
  integer,
  float,
  boolean,
  enumType,
  object,
  array;

  static ConfigFieldType fromString(String value) => switch (value) {
        'string' => ConfigFieldType.string,
        'integer' => ConfigFieldType.integer,
        'float' => ConfigFieldType.float,
        'boolean' => ConfigFieldType.boolean,
        'enum' => ConfigFieldType.enumType,
        'object' => ConfigFieldType.object,
        'array' => ConfigFieldType.array,
        _ => ConfigFieldType.string,
      };
}

class ConfigFieldSchema {
  const ConfigFieldSchema({
    required this.key,
    required this.label,
    this.description,
    required this.type,
    this.min,
    this.max,
    this.options,
    this.pattern,
    this.unit,
    this.defaultValue,
  });

  final String key;
  final String label;
  final String? description;
  final ConfigFieldType type;
  final num? min;
  final num? max;
  final List<String>? options;
  final String? pattern;
  final String? unit;
  final dynamic defaultValue;

  factory ConfigFieldSchema.fromJson(Map<String, dynamic> json) =>
      ConfigFieldSchema(
        key: json['key'] as String,
        label: json['label'] as String,
        description: json['description'] as String?,
        type: ConfigFieldType.fromString(json['type'] as String),
        min: json['min'] as num?,
        max: json['max'] as num?,
        options: (json['options'] as List<dynamic>?)?.cast<String>(),
        pattern: json['pattern'] as String?,
        unit: json['unit'] as String?,
        defaultValue: json['default'],
      );
}

class ConfigCategorySchema {
  const ConfigCategorySchema({
    required this.id,
    required this.label,
    this.icon,
    required this.namespaces,
    required this.fields,
  });

  final String id;
  final String label;
  final String? icon;
  final List<String> namespaces;
  final List<ConfigFieldSchema> fields;

  factory ConfigCategorySchema.fromJson(Map<String, dynamic> json) =>
      ConfigCategorySchema(
        id: json['id'] as String,
        label: json['label'] as String,
        icon: json['icon'] as String?,
        namespaces: (json['namespaces'] as List<dynamic>).cast<String>(),
        fields: (json['fields'] as List<dynamic>)
            .map((f) =>
                ConfigFieldSchema.fromJson(f as Map<String, dynamic>))
            .toList(),
      );
}

class ConfigEditorSchema {
  const ConfigEditorSchema({
    required this.service,
    required this.namespacePrefix,
    required this.categories,
  });

  final String service;
  final String namespacePrefix;
  final List<ConfigCategorySchema> categories;

  factory ConfigEditorSchema.fromJson(Map<String, dynamic> json) =>
      ConfigEditorSchema(
        service: json['service'] as String,
        namespacePrefix: json['namespace_prefix'] as String,
        categories: (json['categories'] as List<dynamic>)
            .map((c) =>
                ConfigCategorySchema.fromJson(c as Map<String, dynamic>))
            .toList(),
      );
}
