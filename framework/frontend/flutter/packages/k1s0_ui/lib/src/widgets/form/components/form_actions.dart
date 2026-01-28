/// K1s0 Form アクション
library;

import 'package:flutter/material.dart';

/// フォームのアクションボタン群
class K1s0FormActions extends StatelessWidget {
  /// 送信ボタンラベル
  final String submitLabel;

  /// キャンセルボタンラベル
  final String? cancelLabel;

  /// リセットボタンラベル
  final String? resetLabel;

  /// キャンセルボタン表示
  final bool showCancel;

  /// リセットボタン表示
  final bool showReset;

  /// 送信時コールバック
  final VoidCallback? onSubmit;

  /// キャンセル時コールバック
  final VoidCallback? onCancel;

  /// リセット時コールバック
  final VoidCallback? onReset;

  /// ローディング状態
  final bool loading;

  /// 無効化状態
  final bool disabled;

  const K1s0FormActions({
    super.key,
    this.submitLabel = '送信',
    this.cancelLabel,
    this.resetLabel,
    this.showCancel = false,
    this.showReset = false,
    this.onSubmit,
    this.onCancel,
    this.onReset,
    this.loading = false,
    this.disabled = false,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(top: 24),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.end,
        children: [
          if (showReset) ...[
            TextButton(
              onPressed: disabled || loading ? null : onReset,
              child: Text(resetLabel ?? 'リセット'),
            ),
            const SizedBox(width: 8),
          ],
          if (showCancel) ...[
            OutlinedButton(
              onPressed: loading ? null : onCancel,
              child: Text(cancelLabel ?? 'キャンセル'),
            ),
            const SizedBox(width: 8),
          ],
          FilledButton(
            onPressed: disabled || loading ? null : onSubmit,
            child: loading
                ? const SizedBox(
                    width: 20,
                    height: 20,
                    child: CircularProgressIndicator(
                      strokeWidth: 2,
                      color: Colors.white,
                    ),
                  )
                : Text(submitLabel),
          ),
        ],
      ),
    );
  }
}
