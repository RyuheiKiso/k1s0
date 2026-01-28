/// K1s0 Form 日付フィールド
library;

import 'package:flutter/material.dart';

/// 日付選択フィールド
class K1s0DateFormField extends StatelessWidget {
  /// 現在の値
  final DateTime? value;

  /// 値変更時コールバック
  final void Function(DateTime? value)? onChanged;

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

  /// 読み取り専用
  final bool readOnly;

  /// 最小日付
  final DateTime? firstDate;

  /// 最大日付
  final DateTime? lastDate;

  /// 日付フォーマット
  final String Function(DateTime date)? formatDate;

  const K1s0DateFormField({
    super.key,
    this.value,
    this.onChanged,
    this.label,
    this.placeholder,
    this.helperText,
    this.errorText,
    this.required = false,
    this.disabled = false,
    this.readOnly = false,
    this.firstDate,
    this.lastDate,
    this.formatDate,
  });

  String _defaultFormatDate(DateTime date) {
    return '${date.year}/${date.month.toString().padLeft(2, '0')}/${date.day.toString().padLeft(2, '0')}';
  }

  Future<void> _showDatePicker(BuildContext context) async {
    final now = DateTime.now();
    final selectedDate = await showDatePicker(
      context: context,
      initialDate: value ?? now,
      firstDate: firstDate ?? DateTime(1900),
      lastDate: lastDate ?? DateTime(2100),
    );

    if (selectedDate != null) {
      onChanged?.call(selectedDate);
    }
  }

  @override
  Widget build(BuildContext context) {
    final displayValue = value != null
        ? (formatDate ?? _defaultFormatDate)(value!)
        : '';

    return TextFormField(
      readOnly: true,
      controller: TextEditingController(text: displayValue),
      decoration: InputDecoration(
        labelText: label != null ? (required ? '$label *' : label) : null,
        hintText: placeholder ?? '日付を選択',
        helperText: helperText,
        errorText: errorText,
        suffixIcon: IconButton(
          icon: const Icon(Icons.calendar_today),
          onPressed: disabled || readOnly
              ? null
              : () => _showDatePicker(context),
        ),
      ),
      enabled: !disabled,
      onTap: disabled || readOnly ? null : () => _showDatePicker(context),
    );
  }
}
