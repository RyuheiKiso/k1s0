import 'package:flutter/material.dart';

import '../../config_types.dart';

/// 列挙型設定フィールドの入力ウィジェット
/// ドロップダウンで選択肢から値を選択する
class EnumField extends StatelessWidget {
  const EnumField({
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
    final options = schema.options ?? [];

    return DropdownButtonFormField<String>(
      initialValue: options.contains(currentValue) ? currentValue : null,
      decoration: InputDecoration(
        labelText: schema.label,
        helperText: schema.description,
        errorText: errorText,
      ),
      items: options
          .map((option) => DropdownMenuItem(value: option, child: Text(option)))
          .toList(),
      onChanged: (value) {
        if (value != null) {
          onChanged(value);
        }
      },
    );
  }

  /// スキーマのデフォルト値から文字列を取得する（存在しない場合は null）
  String? _defaultString() {
    final def = schema.defaultValue;
    if (def is StringConfigValue) return def.value;
    return null;
  }
}
