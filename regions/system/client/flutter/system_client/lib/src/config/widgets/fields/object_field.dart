import 'dart:convert';

import 'package:flutter/material.dart';

import '../../config_types.dart';

/// オブジェクト型設定フィールドの入力ウィジェット
/// JSON 文字列として表示・編集し、パース結果を Map で返す
class ObjectField extends StatelessWidget {
  const ObjectField({
    super.key,
    required this.schema,
    required this.value,
    required this.errorText,
    required this.onValidationChanged,
    required this.onChanged,
  });

  final ConfigFieldSchema schema;
  /// 現在の設定値（ConfigValue sealed class で型安全に受け取る）
  final ConfigValue value;
  final String? errorText;
  final ValueChanged<String?> onValidationChanged;
  final ValueChanged<Map<String, dynamic>> onChanged;

  @override
  Widget build(BuildContext context) {
    /// ConfigValue から Map 値を取り出す（型不一致時はデフォルト値にフォールバック）
    final currentValue = value is MapConfigValue
        ? (value as MapConfigValue).toJson() as Map<String, dynamic>
        : _defaultMap();

    return TextFormField(
      initialValue: const JsonEncoder.withIndent('  ').convert(currentValue),
      maxLines: 6,
      decoration: InputDecoration(
        labelText: schema.label,
        helperText: schema.description,
        errorText: errorText,
      ),
      onChanged: (raw) {
        try {
          final decoded = jsonDecode(raw);
          if (decoded is Map<String, dynamic>) {
            onValidationChanged(null);
            onChanged(decoded);
          } else {
            onValidationChanged('JSON object is required');
          }
        } catch (_) {
          onValidationChanged('Invalid JSON');
        }
      },
    );
  }

  /// スキーマのデフォルト値から Map を取得する（存在しない場合は空マップ）
  Map<String, dynamic> _defaultMap() {
    final def = schema.defaultValue;
    if (def is MapConfigValue) {
      return def.toJson() as Map<String, dynamic>;
    }
    return const {};
  }
}
