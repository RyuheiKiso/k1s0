import 'package:flutter/material.dart';

import '../../config_types.dart';

/// 整数型設定フィールドの入力ウィジェット
/// テキスト入力とスライダーで整数値を編集する
class IntegerField extends StatelessWidget {
  const IntegerField({
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
  final ValueChanged<int> onChanged;

  @override
  Widget build(BuildContext context) {
    /// ConfigValue から整数値を取り出す（型不一致時はデフォルト値にフォールバック）
    final currentValue = value is NumberConfigValue
        ? (value as NumberConfigValue).value.toInt()
        : _defaultInt();

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        TextFormField(
          initialValue: currentValue.toString(),
          keyboardType: TextInputType.number,
          decoration: InputDecoration(
            labelText: schema.label,
            helperText: schema.description,
            suffixText: schema.unit,
            errorText: errorText,
          ),
          onChanged: (raw) {
            final parsed = int.tryParse(raw);
            if (parsed != null) {
              onValidationChanged(null);
              onChanged(parsed);
              return;
            }
            onValidationChanged('Enter an integer');
          },
        ),
        /// 最小値と最大値が設定されている場合はスライダーを表示する
        if (schema.min != null && schema.max != null)
          Slider(
            value: currentValue.toDouble().clamp(
                  schema.min!.toDouble(),
                  schema.max!.toDouble(),
                ),
            min: schema.min!.toDouble(),
            max: schema.max!.toDouble(),
            divisions: (schema.max! - schema.min!).toInt(),
            label: currentValue.toString(),
            onChanged: (value) => onChanged(value.round()),
          ),
      ],
    );
  }

  /// スキーマのデフォルト値から整数を取得する（存在しない場合は 0）
  int _defaultInt() {
    final def = schema.defaultValue;
    if (def is NumberConfigValue) return def.value.toInt();
    return 0;
  }
}
