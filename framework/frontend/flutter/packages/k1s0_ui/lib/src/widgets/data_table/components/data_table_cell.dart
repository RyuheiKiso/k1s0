/// K1s0 DataTable セル
library;

import 'package:flutter/material.dart';
import '../k1s0_column.dart';

/// DataTable セルウィジェット
class K1s0DataTableCell<T> extends StatelessWidget {
  /// カラム定義
  final K1s0Column<T> column;

  /// 行データ
  final T row;

  /// 行インデックス
  final int index;

  const K1s0DataTableCell({
    super.key,
    required this.column,
    required this.row,
    required this.index,
  });

  @override
  Widget build(BuildContext context) {
    final value = _getValue();

    // カスタムレンダラーがあれば使用
    if (column.renderCell != null) {
      return Container(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        width: column.width,
        child: column.renderCell!(value, row, index),
      );
    }

    // デフォルトのレンダリング
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      width: column.width,
      child: _buildDefaultCell(context, value),
    );
  }

  dynamic _getValue() {
    if (row is Map) {
      return (row as Map)[column.field];
    }
    return null;
  }

  Widget _buildDefaultCell(BuildContext context, dynamic value) {
    final theme = Theme.of(context);

    switch (column.type) {
      case K1s0ColumnType.boolean:
        return _buildBooleanCell(context, value);

      case K1s0ColumnType.singleSelect:
        return _buildSelectCell(context, value);

      default:
        final formattedValue = column.formatValue(value);
        return Text(
          formattedValue,
          textAlign: column.align,
          style: theme.textTheme.bodyMedium,
          overflow: TextOverflow.ellipsis,
        );
    }
  }

  Widget _buildBooleanCell(BuildContext context, dynamic value) {
    final theme = Theme.of(context);

    if (value == null) {
      return Text(
        '-',
        textAlign: column.align,
        style: theme.textTheme.bodyMedium?.copyWith(
          color: theme.hintColor,
        ),
      );
    }

    return Icon(
      value == true ? Icons.check : Icons.close,
      size: 20,
      color: value == true
          ? theme.colorScheme.primary
          : theme.colorScheme.error,
    );
  }

  Widget _buildSelectCell(BuildContext context, dynamic value) {
    final theme = Theme.of(context);

    if (value == null) {
      return Text(
        '-',
        textAlign: column.align,
        style: theme.textTheme.bodyMedium?.copyWith(
          color: theme.hintColor,
        ),
      );
    }

    final option = column.valueOptions?.firstWhere(
      (opt) => opt.value == value,
      orElse: () => K1s0ValueOption(label: value.toString(), value: value),
    );

    return Chip(
      label: Text(
        option?.label ?? value.toString(),
        style: theme.textTheme.labelSmall,
      ),
      materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
      padding: EdgeInsets.zero,
      labelPadding: const EdgeInsets.symmetric(horizontal: 8),
    );
  }
}
