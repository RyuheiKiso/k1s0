/// K1s0 DataTable 空状態
library;

import 'package:flutter/material.dart';

/// DataTable 空状態ウィジェット
class K1s0DataTableEmpty extends StatelessWidget {
  /// メッセージ
  final String message;

  /// アイコン
  final IconData icon;

  /// アクションボタン
  final Widget? action;

  const K1s0DataTableEmpty({
    super.key,
    this.message = 'データがありません',
    this.icon = Icons.inbox_outlined,
    this.action,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              icon,
              size: 64,
              color: theme.colorScheme.onSurfaceVariant.withValues(alpha: 0.5),
            ),
            const SizedBox(height: 16),
            Text(
              message,
              style: theme.textTheme.bodyLarge?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
              textAlign: TextAlign.center,
            ),
            if (action != null) ...[
              const SizedBox(height: 16),
              action!,
            ],
          ],
        ),
      ),
    );
  }
}
