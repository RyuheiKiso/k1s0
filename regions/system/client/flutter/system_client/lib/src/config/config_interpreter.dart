import 'dart:convert';

import 'package:dio/dio.dart';

import 'config_types.dart';

/// 設定データの解釈中にエラーが発生した場合にスローする例外
class ConfigInterpretationException implements Exception {
  final String message;
  const ConfigInterpretationException(this.message);

  @override
  String toString() => 'ConfigInterpretationException: $message';
}

/// サービス設定エントリ
/// API から取得した個別の設定キー・値・バージョンを保持する
class ServiceConfigEntry {
  const ServiceConfigEntry({
    required this.namespace,
    required this.key,
    required this.value,
    required this.version,
  });

  final String namespace;
  final String key;
  /// 設定値: ConfigValue sealed class で型安全に保持する
  final ConfigValue value;
  final int version;

  /// ネームスペースとキーを結合した一意な識別子を返す
  String get id => '$namespace::$key';

  /// JSON マップから ServiceConfigEntry を生成する
  factory ServiceConfigEntry.fromJson(Map<String, dynamic> json) =>
      ServiceConfigEntry(
        namespace: json['namespace'] as String,
        key: json['key'] as String,
        value: ConfigValue.fromJson(json['value']),
        version: (json['version'] as num?)?.toInt() ?? 0,
      );
}

/// サービス設定の取得結果
/// サービス名と設定エントリ一覧を保持する
class ServiceConfigResult {
  const ServiceConfigResult({
    required this.serviceName,
    required this.entries,
  });

  final String serviceName;
  final List<ServiceConfigEntry> entries;

  /// JSON マップから ServiceConfigResult を生成する
  factory ServiceConfigResult.fromJson(Map<String, dynamic> json) =>
      ServiceConfigResult(
        serviceName: json['service_name'] as String,
        entries: (json['entries'] as List<dynamic>? ?? [])
            .map((entry) =>
                ServiceConfigEntry.fromJson(entry as Map<String, dynamic>))
            .toList(),
      );
}

/// 設定フィールドの現在の状態
/// スキーマ・現在値・元の値・バージョン・変更状態を保持する
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
  /// 現在の設定値: ConfigValue sealed class で型安全に保持する
  final ConfigValue value;
  /// 元の設定値: 変更前の値を保持して差分検出に使用する
  final ConfigValue originalValue;
  final int version;
  final int originalVersion;
  final bool isDirty;
  final String? error;

  /// 一部のフィールドを更新した新しいインスタンスを返す
  ConfigFieldState copyWith({
    String? id,
    String? namespace,
    String? key,
    ConfigFieldSchema? schema,
    ConfigValue? value,
    ConfigValue? originalValue,
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

/// 設定カテゴリの状態
/// カテゴリスキーマとそのフィールド一覧を保持する
class ConfigCategoryState {
  const ConfigCategoryState({
    required this.schema,
    required this.fields,
  });

  final ConfigCategorySchema schema;
  final List<ConfigFieldState> fields;

  /// 一部のフィールドを更新した新しいインスタンスを返す
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

/// 設定エディタ全体のデータ
/// サービス名・カテゴリ一覧・変更フィールド数を保持する
class ConfigData {
  const ConfigData({
    required this.service,
    required this.categories,
    required this.dirtyCount,
  });

  final String service;
  final List<ConfigCategoryState> categories;
  final int dirtyCount;

  /// 一部のフィールドを更新した新しいインスタンスを返す
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

/// 設定スキーマと現在の設定値を統合してエディタ用データを構築する
class ConfigInterpreter {
  const ConfigInterpreter({required this.dio});

  final Dio dio;

  /// API からスキーマと設定値を取得し、エディタ用データに変換する
  Future<ConfigData> build(String serviceName) async {
    final results = await Future.wait([
      dio.get<Map<String, dynamic>>('/api/v1/config-schema/$serviceName'),
      dio.get<Map<String, dynamic>>('/api/v1/config/services/$serviceName'),
    ]);

    // null 強制演算子の代わりに明示的なエラーハンドリングを行う
    final schemaData = results[0].data;
    if (schemaData == null) {
      throw const ConfigInterpretationException('設定スキーマデータが null です');
    }

    final configData = results[1].data;
    if (configData == null) {
      throw const ConfigInterpretationException('設定データが null です');
    }

    final schema = ConfigEditorSchema.fromJson(schemaData);
    final serviceConfig = ServiceConfigResult.fromJson(configData);
    /// エントリを ID → エントリのマップに変換して高速ルックアップを可能にする
    final entryMap = {
      for (final entry in serviceConfig.entries) entry.id: entry,
    };

    /// 各カテゴリのフィールドに現在値またはデフォルト値を設定する
    final categories = schema.categories.map((category) {
      final fields = category.fields.map((field) {
        final existing = _findEntry(category.namespaces, field.key, entryMap);
        final namespace = existing?.namespace ??
            (category.namespaces.isNotEmpty
                ? category.namespaces.first
                : schema.namespacePrefix);
        /// 既存値がある場合はそれを使用し、なければデフォルト値にフォールバックする
        final currentValue = existing?.value ??
            field.defaultValue ??
            const StringConfigValue('');
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

/// 指定されたネームスペース一覧からキーに一致するエントリを検索する
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

/// ネームスペースとキーからフィールド ID を生成する
String buildFieldId(String namespace, String key) => '$namespace::$key';

/// ConfigValue に対してスキーマの型制約を検証し、エラーメッセージを返す
String? validateFieldValue(ConfigFieldSchema schema, ConfigValue value) {
  switch (schema.type) {
    case ConfigFieldType.integer:
      if (value is! NumberConfigValue || value.value is! int) {
        return 'Enter an integer';
      }
      return _validateNumberRange(schema, value.value);
    case ConfigFieldType.float:
      if (value is! NumberConfigValue) return 'Enter a number';
      return _validateNumberRange(schema, value.value);
    case ConfigFieldType.boolean:
      return value is BoolConfigValue ? null : 'Enter a boolean';
    case ConfigFieldType.enumType:
      if (value is! StringConfigValue) return 'Select a value';
      if (schema.options != null && !schema.options!.contains(value.value)) {
        return 'Select a valid option';
      }
      return null;
    case ConfigFieldType.object:
      return value is MapConfigValue ? null : 'Enter an object';
    case ConfigFieldType.array:
      return value is ListConfigValue ? null : 'Enter an array';
    case ConfigFieldType.string:
      if (value is! StringConfigValue) return 'Enter a string';
      if (schema.pattern != null && value.value.isNotEmpty) {
        // 不正な正規表現パターンによる FormatException を安全にキャッチする
        // 不正パターンはバリデーションエラーとして扱い、アプリのクラッシュを防ぐ
        try {
          if (!RegExp(schema.pattern!).hasMatch(value.value)) {
            return 'Must match ${schema.pattern}';
          }
        } on FormatException catch (e) {
          // 不正な正規表現パターンの場合はバリデーションエラーとして扱う
          return '不正な正規表現パターンです: ${e.message}';
        }
      }
      return null;
  }
}

/// 2つの ConfigValue が同値かどうかを JSON 表現で比較する
bool isSameValue(ConfigValue left, ConfigValue right) {
  return jsonEncode(left.toJson()) == jsonEncode(right.toJson());
}

/// フィールドの値を更新し、変更状態とバリデーション結果を再計算する
ConfigFieldState updateFieldState(ConfigFieldState field, ConfigValue nextValue) {
  return field.copyWith(
    value: nextValue,
    isDirty: !isSameValue(nextValue, field.originalValue),
    error: validateFieldValue(field.schema, nextValue),
  );
}

/// 全フィールドを元の値にリセットする
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

/// 変更されたフィールドの合計数をカウントする
int countDirtyFields(List<ConfigCategoryState> categories) {
  return categories.fold<int>(
    0,
    (sum, category) =>
        sum + category.fields.where((field) => field.isDirty).length,
  );
}

/// 数値の範囲制約を検証する
String? _validateNumberRange(ConfigFieldSchema schema, num value) {
  if (schema.min != null && value < schema.min!) {
    return 'Must be >= ${schema.min}';
  }
  if (schema.max != null && value > schema.max!) {
    return 'Must be <= ${schema.max}';
  }
  return null;
}
