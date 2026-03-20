import 'dart:convert';

import 'package:flutter/material.dart';

import '../config_interpreter.dart';
import '../config_types.dart';
import 'fields/array_field.dart';
import 'fields/boolean_field.dart';
import 'fields/enum_field.dart';
import 'fields/float_field.dart';
import 'fields/integer_field.dart';
import 'fields/object_field.dart';
import 'fields/string_field.dart';

/// 設定フィールド一覧ウィジェット
/// カテゴリ内の全フィールドをリスト表示し、変更・バリデーション・リセットを処理する
class ConfigFieldList extends StatelessWidget {
  const ConfigFieldList({
    super.key,
    required this.category,
    required this.onFieldChanged,
    required this.onFieldValidationChanged,
    required this.onResetToDefault,
  });

  final ConfigCategoryState category;
  /// フィールド値変更時のコールバック（ConfigValue で型安全に値を渡す）
  final void Function(String key, ConfigValue value) onFieldChanged;
  final void Function(String key, String? error) onFieldValidationChanged;
  final void Function(String key) onResetToDefault;

  @override
  Widget build(BuildContext context) {
    return ListView.separated(
      padding: const EdgeInsets.all(16),
      itemCount: category.fields.length,
      separatorBuilder: (_, _) => const SizedBox(height: 16),
      itemBuilder: (context, index) {
        final field = category.fields[index];
        final fieldWidget = _buildFieldWidget(field);

        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Align(
              alignment: Alignment.centerRight,
              child: field.isDirty
                  ? TextButton(
                      onPressed: () => onResetToDefault(field.key),
                      child: const Text('Reset to default'),
                    )
                  : const SizedBox.shrink(),
            ),
            fieldWidget,
          ],
        );
      },
    );
  }

  /// フィールドの型に応じた入力ウィジェットを構築する
  Widget _buildFieldWidget(ConfigFieldState field) {
    /// ValueKey に値の JSON 表現を含めて値変更時に再構築を保証する
    switch (field.schema.type) {
      case ConfigFieldType.integer:
        return IntegerField(
          key: ValueKey(
              '${field.id}:${field.version}:${jsonEncode(field.value.toJson())}'),
          schema: field.schema,
          value: field.value,
          errorText: field.error,
          onValidationChanged: (error) =>
              onFieldValidationChanged(field.key, error),
          onChanged: (value) =>
              onFieldChanged(field.key, NumberConfigValue(value)),
        );
      case ConfigFieldType.float:
        return FloatField(
          key: ValueKey(
              '${field.id}:${field.version}:${jsonEncode(field.value.toJson())}'),
          schema: field.schema,
          value: field.value,
          errorText: field.error,
          onValidationChanged: (error) =>
              onFieldValidationChanged(field.key, error),
          onChanged: (value) =>
              onFieldChanged(field.key, NumberConfigValue(value)),
        );
      case ConfigFieldType.boolean:
        return BooleanField(
          key: ValueKey(
              '${field.id}:${field.version}:${jsonEncode(field.value.toJson())}'),
          schema: field.schema,
          value: field.value,
          errorText: field.error,
          onChanged: (value) =>
              onFieldChanged(field.key, BoolConfigValue(value)),
        );
      case ConfigFieldType.enumType:
        return EnumField(
          key: ValueKey(
              '${field.id}:${field.version}:${jsonEncode(field.value.toJson())}'),
          schema: field.schema,
          value: field.value,
          errorText: field.error,
          onChanged: (value) =>
              onFieldChanged(field.key, StringConfigValue(value)),
        );
      case ConfigFieldType.object:
        return ObjectField(
          key: ValueKey(
              '${field.id}:${field.version}:${jsonEncode(field.value.toJson())}'),
          schema: field.schema,
          value: field.value,
          errorText: field.error,
          onValidationChanged: (error) =>
              onFieldValidationChanged(field.key, error),
          onChanged: (value) =>
              onFieldChanged(field.key, MapConfigValue(
                value.map((k, v) => MapEntry(k, ConfigValue.fromJson(v))),
              )),
        );
      case ConfigFieldType.array:
        return ArrayField(
          key: ValueKey(
              '${field.id}:${field.version}:${jsonEncode(field.value.toJson())}'),
          schema: field.schema,
          value: field.value,
          errorText: field.error,
          onChanged: (value) =>
              onFieldChanged(field.key, ListConfigValue(
                value.map((v) => StringConfigValue(v)).toList(),
              )),
        );
      case ConfigFieldType.string:
        return StringField(
          key: ValueKey(
              '${field.id}:${field.version}:${jsonEncode(field.value.toJson())}'),
          schema: field.schema,
          value: field.value,
          errorText: field.error,
          onChanged: (value) =>
              onFieldChanged(field.key, StringConfigValue(value)),
        );
    }
  }
}
