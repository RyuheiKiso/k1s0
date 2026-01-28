/// K1s0 Form スライダーフィールド
library;

import 'package:flutter/material.dart';
import '../components/form_field_wrapper.dart';

/// スライダーフィールド
class K1s0SliderFormField extends StatelessWidget {
  /// 現在の値
  final double value;

  /// 値変更時コールバック
  final void Function(double value)? onChanged;

  /// ラベル
  final String? label;

  /// ヘルプテキスト
  final String? helperText;

  /// エラーメッセージ
  final String? errorText;

  /// 必須
  final bool required;

  /// 無効化
  final bool disabled;

  /// 最小値
  final double min;

  /// 最大値
  final double max;

  /// 分割数
  final int? divisions;

  /// 値表示
  final bool showValue;

  /// 値フォーマット
  final String Function(double value)? formatValue;

  const K1s0SliderFormField({
    super.key,
    this.value = 0,
    this.onChanged,
    this.label,
    this.helperText,
    this.errorText,
    this.required = false,
    this.disabled = false,
    this.min = 0,
    this.max = 100,
    this.divisions,
    this.showValue = true,
    this.formatValue,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return K1s0FormFieldWrapper(
      label: label,
      required: required,
      helperText: helperText,
      errorText: errorText,
      child: Row(
        children: [
          Expanded(
            child: Slider(
              value: value.clamp(min, max),
              min: min,
              max: max,
              divisions: divisions,
              onChanged: disabled ? null : onChanged,
              label: formatValue?.call(value) ?? value.toStringAsFixed(0),
            ),
          ),
          if (showValue)
            SizedBox(
              width: 50,
              child: Text(
                formatValue?.call(value) ?? value.toStringAsFixed(0),
                style: theme.textTheme.bodyMedium,
                textAlign: TextAlign.center,
              ),
            ),
        ],
      ),
    );
  }
}
