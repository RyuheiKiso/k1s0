import 'package:flutter/material.dart';

import '../../config_types.dart';

/// 真偽値型設定フィールドの入力ウィジェット
/// SwitchListTile で ON/OFF を切り替える
class BooleanField extends StatelessWidget {
  const BooleanField({
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
  final ValueChanged<bool> onChanged;

  @override
  Widget build(BuildContext context) {
    /// ConfigValue から真偽値を取り出す（型不一致時はデフォルト値にフォールバック）
    final currentValue = value is BoolConfigValue
        ? (value as BoolConfigValue).value
        : _defaultBool();

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SwitchListTile(
          title: Text(schema.label),
          subtitle:
              schema.description != null ? Text(schema.description!) : null,
          value: currentValue,
          onChanged: onChanged,
        ),
        if (errorText != null)
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Text(
              errorText!,
              style: TextStyle(color: Theme.of(context).colorScheme.error),
            ),
          ),
      ],
    );
  }

  /// スキーマのデフォルト値から bool を取得する（存在しない場合は false）
  bool _defaultBool() {
    final def = schema.defaultValue;
    if (def is BoolConfigValue) return def.value;
    return false;
  }
}
