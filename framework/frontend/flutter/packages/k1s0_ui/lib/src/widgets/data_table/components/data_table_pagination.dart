/// K1s0 DataTable ページネーション
library;

import 'package:flutter/material.dart';

/// DataTable ページネーションウィジェット
class K1s0DataTablePagination extends StatelessWidget {
  /// 現在のページ（0始まり）
  final int page;

  /// ページサイズ
  final int pageSize;

  /// 総行数
  final int totalRows;

  /// ページサイズ選択肢
  final List<int> pageSizeOptions;

  /// ページ変更時コールバック
  final void Function(int page)? onPageChange;

  /// ページサイズ変更時コールバック
  final void Function(int pageSize)? onPageSizeChange;

  const K1s0DataTablePagination({
    super.key,
    required this.page,
    required this.pageSize,
    required this.totalRows,
    this.pageSizeOptions = const [10, 20, 50, 100],
    this.onPageChange,
    this.onPageSizeChange,
  });

  int get totalPages => (totalRows / pageSize).ceil();
  int get startRow => page * pageSize + 1;
  int get endRow => ((page + 1) * pageSize).clamp(1, totalRows);

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      decoration: BoxDecoration(
        color: theme.colorScheme.surface,
        border: Border(
          top: BorderSide(
            color: theme.dividerColor,
            width: 1,
          ),
        ),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          // ページサイズ選択
          Row(
            children: [
              Text(
                '表示件数:',
                style: theme.textTheme.bodyMedium,
              ),
              const SizedBox(width: 8),
              DropdownButton<int>(
                value: pageSize,
                underline: const SizedBox.shrink(),
                items: pageSizeOptions.map((size) {
                  return DropdownMenuItem(
                    value: size,
                    child: Text('$size'),
                  );
                }).toList(),
                onChanged: onPageSizeChange != null
                    ? (value) {
                        if (value != null) {
                          onPageSizeChange!(value);
                        }
                      }
                    : null,
              ),
            ],
          ),

          // 行数表示
          Text(
            totalRows > 0
                ? '$startRow - $endRow / $totalRows'
                : '0件',
            style: theme.textTheme.bodyMedium,
          ),

          // ページ移動ボタン
          Row(
            children: [
              // 最初のページ
              IconButton(
                icon: const Icon(Icons.first_page),
                onPressed: page > 0
                    ? () => onPageChange?.call(0)
                    : null,
                tooltip: '最初のページ',
              ),
              // 前のページ
              IconButton(
                icon: const Icon(Icons.chevron_left),
                onPressed: page > 0
                    ? () => onPageChange?.call(page - 1)
                    : null,
                tooltip: '前のページ',
              ),
              // ページ番号
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 8),
                child: Text(
                  '${page + 1} / $totalPages',
                  style: theme.textTheme.bodyMedium,
                ),
              ),
              // 次のページ
              IconButton(
                icon: const Icon(Icons.chevron_right),
                onPressed: page < totalPages - 1
                    ? () => onPageChange?.call(page + 1)
                    : null,
                tooltip: '次のページ',
              ),
              // 最後のページ
              IconButton(
                icon: const Icon(Icons.last_page),
                onPressed: page < totalPages - 1
                    ? () => onPageChange?.call(totalPages - 1)
                    : null,
                tooltip: '最後のページ',
              ),
            ],
          ),
        ],
      ),
    );
  }
}
