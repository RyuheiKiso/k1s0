/// K1s0 Form ドロップダウンフィールド
library;

import 'package:flutter/material.dart';
import '../k1s0_form_schema.dart';

/// ドロップダウン選択フィールド
class K1s0DropdownFormField<T> extends StatelessWidget {
  /// 現在の値
  final T? value;

  /// 値変更時コールバック
  final void Function(T? value)? onChanged;

  /// 選択肢
  final List<K1s0FieldOption> options;

  /// ラベル
  final String? label;

  /// プレースホルダー
  final String? placeholder;

  /// ヘルプテキスト
  final String? helperText;

  /// エラーメッセージ
  final String? errorText;

  /// 必須
  final bool required;

  /// 無効化
  final bool disabled;

  const K1s0DropdownFormField({
    super.key,
    this.value,
    this.onChanged,
    required this.options,
    this.label,
    this.placeholder,
    this.helperText,
    this.errorText,
    this.required = false,
    this.disabled = false,
  });

  @override
  Widget build(BuildContext context) {
    return DropdownButtonFormField<T>(
      value: value,
      onChanged: disabled ? null : onChanged,
      decoration: InputDecoration(
        labelText: label != null ? (required ? '$label *' : label) : null,
        hintText: placeholder,
        helperText: helperText,
        errorText: errorText,
      ),
      items: options.map((option) {
        return DropdownMenuItem<T>(
          value: option.value as T,
          enabled: !option.disabled,
          child: Text(option.label),
        );
      }).toList(),
    );
  }
}
