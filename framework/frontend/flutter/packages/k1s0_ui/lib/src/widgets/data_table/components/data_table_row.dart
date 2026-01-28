/// K1s0 DataTable 行
library;

import 'package:flutter/material.dart';
import '../k1s0_column.dart';
import 'data_table_cell.dart';

/// DataTable 行ウィジェット
class K1s0DataTableRow<T> extends StatelessWidget {
  /// 行データ
  final T row;

  /// 行インデックス
  final int index;

  /// カラム定義
  final List<K1s0Column<T>> columns;

  /// 選択状態
  final bool isSelected;

  /// チェックボックス表示
  final bool showCheckbox;

  /// 選択クリック時コールバック
  final VoidCallback? onSelect;

  /// 行タップ時コールバック
  final VoidCallback? onTap;

  /// 行ダブルタップ時コールバック
  final VoidCallback? onDoubleTap;

  /// 行高さ
  final double? rowHeight;

  const K1s0DataTableRow({
    super.key,
    required this.row,
    required this.index,
    required this.columns,
    this.isSelected = false,
    this.showCheckbox = false,
    this.onSelect,
    this.onTap,
    this.onDoubleTap,
    this.rowHeight,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return InkWell(
      onTap: onTap,
      onDoubleTap: onDoubleTap,
      child: Container(
        height: rowHeight,
        decoration: BoxDecoration(
          color: isSelected
              ? theme.colorScheme.primaryContainer.withValues(alpha: 0.3)
              : index.isOdd
                  ? theme.colorScheme.surfaceContainerLowest
                  : null,
          border: Border(
            bottom: BorderSide(
              color: theme.dividerColor,
              width: 0.5,
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
                  value: isSelected,
                  onChanged: (_) => onSelect?.call(),
                ),
              ),
            // セル
            ...columns.map((column) => _buildCell(context, column)),
          ],
        ),
      ),
    );
  }

  Widget _buildCell(BuildContext context, K1s0Column<T> column) {
    return Expanded(
      flex: column.flex?.toInt() ?? 1,
      child: K1s0DataTableCell<T>(
        column: column,
        row: row,
        index: index,
      ),
    );
  }
}
