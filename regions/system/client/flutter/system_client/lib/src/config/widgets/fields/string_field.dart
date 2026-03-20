import 'package:flutter/material.dart';

import '../../config_types.dart';

/// 文字列型設定フィールドの入力ウィジェット
/// ConfigValue から文字列値を取り出して TextFormField で表示・編集する
class StringField extends StatelessWidget {
  const StringField({
    super.key,
    required this.schema,
    required this.value,
    required this.errorText,
    required this.onChanged,
  });

  final ConfigFieldSchema schema;
  /// 現在の設定値（ConfigValue sealed class で型安全に受け取る）
  final ConfigValue value;
  final String? errorText;
  final ValueChanged<String> onChanged;

  @override
  Widget build(BuildContext context) {
    /// ConfigValue から文字列値を取り出す（型不一致時はデフォルト値にフォールバック）
    final currentValue = value is StringConfigValue
        ? (value as StringConfigValue).value
        : _defaultString();

    return TextFormField(
      initialValue: currentValue,
      decoration: InputDecoration(
        labelText: schema.label,
        helperText: schema.description,
        suffixText: schema.unit,
        errorText: errorText,
      ),
      onChanged: onChanged,
    );
  }

  /// スキーマのデフォルト値から文字列を取得する（存在しない場合は空文字列）
  String _defaultString() {
    final def = schema.defaultValue;
    if (def is StringConfigValue) return def.value;
    return '';
  }
}
