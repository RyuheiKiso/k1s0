import 'package:flutter/material.dart';

import '../../config_types.dart';

/// 浮動小数点型設定フィールドの入力ウィジェット
/// テキスト入力とスライダーで数値を編集する
class FloatField extends StatelessWidget {
  const FloatField({
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
  final ValueChanged<double> onChanged;

  @override
  Widget build(BuildContext context) {
    /// ConfigValue から数値を取り出す（型不一致時はデフォルト値にフォールバック）
    final currentValue = value is NumberConfigValue
        ? (value as NumberConfigValue).value.toDouble()
        : _defaultDouble();

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        TextFormField(
          initialValue: currentValue.toString(),
          keyboardType: const TextInputType.numberWithOptions(decimal: true),
          decoration: InputDecoration(
            labelText: schema.label,
            helperText: schema.description,
            suffixText: schema.unit,
            errorText: errorText,
          ),
          onChanged: (raw) {
            final parsed = double.tryParse(raw);
            if (parsed != null) {
              onValidationChanged(null);
              onChanged(parsed);
              return;
            }
            onValidationChanged('Enter a number');
          },
        ),
        /// 最小値と最大値が設定されている場合はスライダーを表示する
        if (schema.min != null && schema.max != null)
          Slider(
            value: currentValue.clamp(
              schema.min!.toDouble(),
              schema.max!.toDouble(),
            ),
            min: schema.min!.toDouble(),
            max: schema.max!.toDouble(),
            label: currentValue.toString(),
            onChanged: onChanged,
          ),
      ],
    );
  }

  /// スキーマのデフォルト値から double を取得する（存在しない場合は 0）
  double _defaultDouble() {
    final def = schema.defaultValue;
    if (def is NumberConfigValue) return def.value.toDouble();
    return 0;
  }
}
