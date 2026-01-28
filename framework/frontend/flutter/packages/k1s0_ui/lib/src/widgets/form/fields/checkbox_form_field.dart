/// K1s0 Form チェックボックスフィールド
library;

import 'package:flutter/material.dart';
import '../components/form_field_wrapper.dart';

/// チェックボックスフィールド
class K1s0CheckboxFormField extends StatelessWidget {
  /// 現在の値
  final bool value;

  /// 値変更時コールバック
  final void Function(bool value)? onChanged;

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

  const K1s0CheckboxFormField({
    super.key,
    this.value = false,
    this.onChanged,
    this.label,
    this.helperText,
    this.errorText,
    this.required = false,
    this.disabled = false,
  });

  @override
  Widget build(BuildContext context) {
    return K1s0FormFieldWrapper(
      helperText: helperText,
      errorText: errorText,
      child: CheckboxListTile(
        title: label != null
            ? Text.rich(
                TextSpan(
                  children: [
                    TextSpan(text: label),
                    if (required)
                      TextSpan(
                        text: ' *',
                        style: TextStyle(
                          color: Theme.of(context).colorScheme.error,
                        ),
                      ),
                  ],
                ),
              )
            : null,
        value: value,
        onChanged: disabled ? null : (v) => onChanged?.call(v ?? false),
        controlAffinity: ListTileControlAffinity.leading,
        contentPadding: EdgeInsets.zero,
        dense: true,
      ),
    );
  }
}
