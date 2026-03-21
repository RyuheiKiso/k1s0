/// 設定値の型安全な表現: dynamic の代わりに sealed class を使用する
sealed class ConfigValue {
  const ConfigValue();

  /// JSON互換の値に変換する（API送信やシリアライズ用）
  dynamic toJson();

  /// JSON互換の値から ConfigValue インスタンスを生成する
  static ConfigValue fromJson(dynamic value) {
    if (value is String) return StringConfigValue(value);
    if (value is bool) return BoolConfigValue(value);
    if (value is num) return NumberConfigValue(value);
    if (value is List) {
      return ListConfigValue(
        value.map((v) => ConfigValue.fromJson(v)).toList(),
      );
    }
    if (value is Map<String, dynamic>) {
      return MapConfigValue(
        value.map((k, v) => MapEntry(k, ConfigValue.fromJson(v))),
      );
    }
    // null またはサポート外の型は文字列として扱う
    return StringConfigValue(value?.toString() ?? '');
  }
}

/// 文字列型の設定値
class StringConfigValue extends ConfigValue {
  final String value;
  const StringConfigValue(this.value);

  @override
  dynamic toJson() => value;

  @override
  bool operator ==(Object other) =>
      other is StringConfigValue && other.value == value;

  @override
  int get hashCode => value.hashCode;
}

/// 数値型の設定値
class NumberConfigValue extends ConfigValue {
  final num value;
  const NumberConfigValue(this.value);

  @override
  dynamic toJson() => value;

  @override
  bool operator ==(Object other) =>
      other is NumberConfigValue && other.value == value;

  @override
  int get hashCode => value.hashCode;
}

/// 真偽値型の設定値
class BoolConfigValue extends ConfigValue {
  final bool value;
  const BoolConfigValue(this.value);

  @override
  dynamic toJson() => value;

  @override
  bool operator ==(Object other) =>
      other is BoolConfigValue && other.value == value;

  @override
  int get hashCode => value.hashCode;
}

/// リスト型の設定値
class ListConfigValue extends ConfigValue {
  final List<ConfigValue> values;
  const ListConfigValue(this.values);

  @override
  dynamic toJson() => values.map((v) => v.toJson()).toList();

  @override
  bool operator ==(Object other) {
    if (other is! ListConfigValue) return false;
    if (other.values.length != values.length) return false;
    for (var i = 0; i < values.length; i++) {
      if (values[i] != other.values[i]) return false;
    }
    return true;
  }

  @override
  int get hashCode => Object.hashAll(values);
}

/// マップ型の設定値
class MapConfigValue extends ConfigValue {
  final Map<String, ConfigValue> entries;
  const MapConfigValue(this.entries);

  @override
  dynamic toJson() =>
      entries.map((k, v) => MapEntry(k, v.toJson()));

  @override
  bool operator ==(Object other) {
    if (other is! MapConfigValue) return false;
    if (other.entries.length != entries.length) return false;
    for (final key in entries.keys) {
      if (!other.entries.containsKey(key)) return false;
      if (entries[key] != other.entries[key]) return false;
    }
    return true;
  }

  @override
  int get hashCode => Object.hashAll(entries.entries.map((e) => Object.hash(e.key, e.value)));
}

/// 設定フィールドの型を表す列挙型
/// YAML スキーマで定義される型名と対応する
enum ConfigFieldType {
  string,
  integer,
  float,
  boolean,
  enumType,
  object,
  array;

  /// 文字列から ConfigFieldType に変換する（不明な型は string にフォールバック）
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

/// 設定フィールドのスキーマ定義
/// 各フィールドの型・制約・デフォルト値を保持する
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
  /// デフォルト値: ConfigValue sealed class で型安全に保持する
  final ConfigValue? defaultValue;

  /// JSON マップから ConfigFieldSchema を生成する
  factory ConfigFieldSchema.fromJson(Map<String, dynamic> json) {
    final defaultJson = json['default'];
    return ConfigFieldSchema(
      key: json['key'] as String,
      label: json['label'] as String,
      description: json['description'] as String?,
      type: ConfigFieldType.fromString(json['type'] as String),
      min: json['min'] as num?,
      max: json['max'] as num?,
      options: (json['options'] as List<dynamic>?)?.cast<String>(),
      pattern: json['pattern'] as String?,
      unit: json['unit'] as String?,
      defaultValue:
          defaultJson != null ? ConfigValue.fromJson(defaultJson) : null,
    );
  }
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
