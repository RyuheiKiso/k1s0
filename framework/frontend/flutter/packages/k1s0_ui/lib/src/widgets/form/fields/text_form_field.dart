/// K1s0 Form テキストフィールド
library;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

/// テキスト入力フィールド
class K1s0TextFormField extends StatelessWidget {
  /// 現在の値
  final String? value;

  /// 値変更時コールバック
  final void Function(String value)? onChanged;

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

  /// 読み取り専用
  final bool readOnly;

  /// 無効化
  final bool disabled;

  /// キーボードタイプ
  final TextInputType? keyboardType;

  /// 入力フォーマッタ
  final List<TextInputFormatter>? inputFormatters;

  /// パスワード入力
  final bool obscureText;

  /// 複数行
  final bool multiline;

  /// 行数
  final int? maxLines;

  /// 最小行数
  final int? minLines;

  /// 最大文字数
  final int? maxLength;

  /// プレフィックスアイコン
  final IconData? prefixIcon;

  /// サフィックスアイコン
  final IconData? suffixIcon;

  /// サフィックスアイコンタップ時コールバック
  final VoidCallback? onSuffixIconTap;

  /// フォーカス時コールバック
  final VoidCallback? onFocus;

  /// フォーカス解除時コールバック
  final VoidCallback? onBlur;

  /// オートフォーカス
  final bool autofocus;

  const K1s0TextFormField({
    super.key,
    this.value,
    this.onChanged,
    this.label,
    this.placeholder,
    this.helperText,
    this.errorText,
    this.required = false,
    this.readOnly = false,
    this.disabled = false,
    this.keyboardType,
    this.inputFormatters,
    this.obscureText = false,
    this.multiline = false,
    this.maxLines,
    this.minLines,
    this.maxLength,
    this.prefixIcon,
    this.suffixIcon,
    this.onSuffixIconTap,
    this.onFocus,
    this.onBlur,
    this.autofocus = false,
  });

  @override
  Widget build(BuildContext context) {
    return TextFormField(
      initialValue: value,
      onChanged: disabled ? null : onChanged,
      decoration: InputDecoration(
        labelText: label != null ? (required ? '$label *' : label) : null,
        hintText: placeholder,
        helperText: helperText,
        errorText: errorText,
        prefixIcon: prefixIcon != null ? Icon(prefixIcon) : null,
        suffixIcon: suffixIcon != null
            ? IconButton(
                icon: Icon(suffixIcon),
                onPressed: onSuffixIconTap,
              )
            : null,
        counterText: maxLength != null ? null : '',
      ),
      readOnly: readOnly,
      enabled: !disabled,
      keyboardType: multiline ? TextInputType.multiline : keyboardType,
      inputFormatters: inputFormatters,
      obscureText: obscureText,
      maxLines: multiline ? (maxLines ?? 5) : 1,
      minLines: multiline ? (minLines ?? 3) : null,
      maxLength: maxLength,
      autofocus: autofocus,
      onTap: onFocus,
      onEditingComplete: onBlur,
    );
  }
}
