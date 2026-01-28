/// K1s0 Form ラジオフィールド
library;

import 'package:flutter/material.dart';
import '../k1s0_form_schema.dart';
import '../components/form_field_wrapper.dart';

/// ラジオボタングループフィールド
class K1s0RadioFormField<T> extends StatelessWidget {
  /// 現在の値
  final T? value;

  /// 値変更時コールバック
  final void Function(T? value)? onChanged;

  /// 選択肢
  final List<K1s0FieldOption> options;

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

  /// 横並び
  final bool horizontal;

  const K1s0RadioFormField({
    super.key,
    this.value,
    this.onChanged,
    required this.options,
    this.label,
    this.helperText,
    this.errorText,
    this.required = false,
    this.disabled = false,
    this.horizontal = false,
  });

  @override
  Widget build(BuildContext context) {
    final radioButtons = options.map((option) {
      return RadioListTile<T>(
        title: Text(option.label),
        value: option.value as T,
        groupValue: value,
        onChanged: disabled || option.disabled ? null : onChanged,
        contentPadding: EdgeInsets.zero,
        dense: true,
      );
    }).toList();

    return K1s0FormFieldWrapper(
      label: label,
      required: required,
      helperText: helperText,
      errorText: errorText,
      child: horizontal
          ? Wrap(
              spacing: 16,
              children: radioButtons,
            )
          : Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: radioButtons,
            ),
    );
  }
}
