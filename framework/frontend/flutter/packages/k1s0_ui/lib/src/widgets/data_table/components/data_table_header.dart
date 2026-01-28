/// K1s0 DataTable ヘッダー
library;

import 'package:flutter/material.dart';
import '../k1s0_column.dart';
import '../k1s0_sort_model.dart';

/// DataTable ヘッダー行
class K1s0DataTableHeader<T> extends StatelessWidget {
  /// カラム定義
  final List<K1s0Column<T>> columns;

  /// ソートモデル
  final List<K1s0SortItem> sortModel;

  /// ソートクリック時コールバック
  final void Function(String field)? onSort;

  /// チェックボックス表示
  final bool showCheckbox;

  /// 全選択状態
  final bool isAllSelected;

  /// 一部選択状態
  final bool isIndeterminate;

  /// 全選択クリック時コールバック
  final VoidCallback? onSelectAll;

  /// ヘッダー背景色
  final Color? backgroundColor;

  const K1s0DataTableHeader({
    super.key,
    required this.columns,
    this.sortModel = const [],
    this.onSort,
    this.showCheckbox = false,
    this.isAllSelected = false,
    this.isIndeterminate = false,
    this.onSelectAll,
    this.backgroundColor,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Container(
      decoration: BoxDecoration(
        color: backgroundColor ?? theme.colorScheme.surfaceContainerHighest,
        border: Border(
          bottom: BorderSide(
            color: theme.dividerColor,
            width: 1,
          ),
        ),
      ),
      child: Row(
        children: [
          // チェックボックス
          if (showCheckbox)
            SizedBox(
              width: 56,
              child: Checkbox(
                value: isAllSelected,
                tristate: true,
                onChanged: (_) => onSelectAll?.call(),
              ),
            ),
          // カラム
          ...columns.map((column) => _buildHeaderCell(context, column)),
        ],
      ),
    );
  }

  Widget _buildHeaderCell(BuildContext context, K1s0Column<T> column) {
    final theme = Theme.of(context);
    final sortItem = sortModel.firstWhere(
      (item) => item.field == column.field,
      orElse: () => K1s0SortItem(field: '', sort: K1s0SortOrder.asc),
    );
    final isSorted = sortModel.any((item) => item.field == column.field);

    Widget child = Text(
      column.headerName,
      style: theme.textTheme.titleSmall?.copyWith(
        fontWeight: FontWeight.bold,
      ),
      textAlign: column.align,
    );

    // ソートインジケーター
    if (column.sortable) {
      child = Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Flexible(child: child),
          if (isSorted)
            Padding(
              padding: const EdgeInsets.only(left: 4),
              child: Icon(
                sortItem.sort == K1s0SortOrder.asc
                    ? Icons.arrow_upward
                    : Icons.arrow_downward,
                size: 16,
                color: theme.colorScheme.primary,
              ),
            ),
        ],
      );
    }

    return Expanded(
      flex: column.flex?.toInt() ?? 1,
      child: InkWell(
        onTap: column.sortable ? () => onSort?.call(column.field) : null,
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
          width: column.width,
          child: child,
        ),
      ),
    );
  }
}
