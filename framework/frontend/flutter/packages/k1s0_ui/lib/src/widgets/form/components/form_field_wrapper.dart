/// K1s0 Form フィールドラッパー
library;

import 'package:flutter/material.dart';

/// フォームフィールドをラップするウィジェット
class K1s0FormFieldWrapper extends StatelessWidget {
  /// ラベル
  final String? label;

  /// 必須マーク表示
  final bool required;

  /// ヘルプテキスト
  final String? helperText;

  /// エラーメッセージ
  final String? errorText;

  /// 子ウィジェット
  final Widget child;

  /// 下部マージン
  final double bottomMargin;

  const K1s0FormFieldWrapper({
    super.key,
    this.label,
    this.required = false,
    this.helperText,
    this.errorText,
    required this.child,
    this.bottomMargin = 16,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Padding(
      padding: EdgeInsets.only(bottom: bottomMargin),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (label != null) ...[
            Row(
              children: [
                Text(
                  label!,
                  style: theme.textTheme.bodyMedium?.copyWith(
                    fontWeight: FontWeight.w500,
                    color: errorText != null
                        ? theme.colorScheme.error
                        : null,
                  ),
                ),
                if (required)
                  Text(
                    ' *',
                    style: theme.textTheme.bodyMedium?.copyWith(
                      color: theme.colorScheme.error,
                    ),
                  ),
              ],
            ),
            const SizedBox(height: 8),
          ],
          child,
          if (errorText != null || helperText != null)
            Padding(
              padding: const EdgeInsets.only(top: 4, left: 12),
              child: Text(
                errorText ?? helperText!,
                style: theme.textTheme.bodySmall?.copyWith(
                  color: errorText != null
                      ? theme.colorScheme.error
                      : theme.hintColor,
                ),
              ),
            ),
        ],
      ),
    );
  }
}
