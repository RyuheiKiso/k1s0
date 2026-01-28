/// K1s0 DataTable メインウィジェット
library;

import 'package:flutter/material.dart';
import 'k1s0_column.dart';
import 'k1s0_sort_model.dart';
import 'k1s0_selection.dart';
import 'controllers/data_table_controller.dart';
import 'components/data_table_header.dart';
import 'components/data_table_row.dart';
import 'components/data_table_pagination.dart';
import 'components/data_table_loading.dart';
import 'components/data_table_empty.dart';

/// DataTable 密度
enum K1s0DataTableDensity {
  /// コンパクト（36px）
  compact,

  /// 標準（52px）
  standard,

  /// ゆったり（68px）
  comfortable,
}

/// K1s0 DataTable ウィジェット
///
/// Material 3 準拠のデータテーブルウィジェット。
/// ソート、ページネーション、選択機能を提供。
///
/// 例:
/// ```dart
/// K1s0DataTable<User>(
///   rows: users,
///   columns: [
///     K1s0Column(field: 'name', headerName: '氏名', flex: 1, sortable: true),
///     K1s0Column(field: 'email', headerName: 'メール', flex: 1),
///   ],
///   getRowId: (user) => user.id,
///   checkboxSelection: true,
///   onRowTap: (user) => context.go('/users/${user.id}'),
/// )
/// ```
class K1s0DataTable<T> extends StatefulWidget {
  /// データ行
  final List<T> rows;

  /// カラム定義
  final List<K1s0Column<T>> columns;

  /// 行 ID 取得関数
  final String Function(T row) getRowId;

  /// ページネーション有効化
  final bool pagination;

  /// ページサイズ
  final int pageSize;

  /// ページサイズ選択肢
  final List<int> pageSizeOptions;

  /// ソートモデル
  final List<K1s0SortItem>? sortModel;

  /// ソートモデル変更時コールバック
  final void Function(List<K1s0SortItem>)? onSortModelChange;

  /// チェックボックス選択有効化
  final bool checkboxSelection;

  /// 選択モード
  final K1s0SelectionMode selectionMode;

  /// 選択行 ID
  final Set<String>? rowSelectionModel;

  /// 選択変更時コールバック
  final void Function(Set<String>)? onRowSelectionModelChange;

  /// ローディング状態
  final bool loading;

  /// 行タップ時コールバック
  final void Function(T row)? onRowTap;

  /// 行ダブルタップ時コールバック
  final void Function(T row)? onRowDoubleTap;

  /// 密度
  final K1s0DataTableDensity density;

  /// スティッキーヘッダー
  final bool stickyHeader;

  /// 空状態のカスタム表示
  final Widget? emptyWidget;

  /// ローディング状態のカスタム表示
  final Widget? loadingWidget;

  /// ヘッダー背景色
  final Color? headerBackgroundColor;

  const K1s0DataTable({
    super.key,
    required this.rows,
    required this.columns,
    required this.getRowId,
    this.pagination = true,
    this.pageSize = 20,
    this.pageSizeOptions = const [10, 20, 50],
    this.sortModel,
    this.onSortModelChange,
    this.checkboxSelection = false,
    this.selectionMode = K1s0SelectionMode.none,
    this.rowSelectionModel,
    this.onRowSelectionModelChange,
    this.loading = false,
    this.onRowTap,
    this.onRowDoubleTap,
    this.density = K1s0DataTableDensity.standard,
    this.stickyHeader = false,
    this.emptyWidget,
    this.loadingWidget,
    this.headerBackgroundColor,
  });

  @override
  State<K1s0DataTable<T>> createState() => _K1s0DataTableState<T>();
}

class _K1s0DataTableState<T> extends State<K1s0DataTable<T>> {
  late K1s0DataTableController<T> _controller;

  @override
  void initState() {
    super.initState();
    _initController();
  }

  @override
  void didUpdateWidget(K1s0DataTable<T> oldWidget) {
    super.didUpdateWidget(oldWidget);

    // データが変更されたら更新
    if (widget.rows != oldWidget.rows) {
      _controller.rows = widget.rows;
    }

    // ソートモデルが変更されたら更新
    if (widget.sortModel != oldWidget.sortModel && widget.sortModel != null) {
      _controller.sortModel = widget.sortModel!;
    }

    // 選択が変更されたら更新
    if (widget.rowSelectionModel != oldWidget.rowSelectionModel &&
        widget.rowSelectionModel != null) {
      _controller.setSelection(widget.rowSelectionModel!);
    }
  }

  void _initController() {
    _controller = K1s0DataTableController<T>(
      rows: widget.rows,
      getRowId: widget.getRowId,
      initialSortModel: widget.sortModel,
      selectionMode: widget.checkboxSelection
          ? K1s0SelectionMode.multiple
          : widget.selectionMode,
      initialSelection: widget.rowSelectionModel,
      initialPageSize: widget.pageSize,
    );

    _controller.addListener(_onControllerChange);
  }

  void _onControllerChange() {
    // ソート変更を通知
    widget.onSortModelChange?.call(_controller.sortModel);

    // 選択変更を通知
    widget.onRowSelectionModelChange?.call(_controller.selectedIds);

    setState(() {});
  }

  @override
  void dispose() {
    _controller.removeListener(_onControllerChange);
    _controller.dispose();
    super.dispose();
  }

  double get _rowHeight {
    switch (widget.density) {
      case K1s0DataTableDensity.compact:
        return 36;
      case K1s0DataTableDensity.standard:
        return 52;
      case K1s0DataTableDensity.comfortable:
        return 68;
    }
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        // ヘッダー
        K1s0DataTableHeader<T>(
          columns: widget.columns,
          sortModel: _controller.sortModel,
          onSort: (field) => _controller.toggleSort(field),
          showCheckbox: widget.checkboxSelection,
          isAllSelected: _controller.isAllSelected,
          isIndeterminate: _controller.isIndeterminate,
          onSelectAll: () => _controller.toggleSelectAll(),
          backgroundColor: widget.headerBackgroundColor,
        ),

        // ボディ
        Expanded(
          child: _buildBody(),
        ),

        // ページネーション
        if (widget.pagination)
          K1s0DataTablePagination(
            page: _controller.page,
            pageSize: _controller.pageSize,
            totalRows: widget.rows.length,
            pageSizeOptions: widget.pageSizeOptions,
            onPageChange: (page) => _controller.page = page,
            onPageSizeChange: (size) => _controller.pageSize = size,
          ),
      ],
    );
  }

  Widget _buildBody() {
    // ローディング状態
    if (widget.loading) {
      return widget.loadingWidget ??
          K1s0DataTableLoading(
            columnCount: widget.columns.length,
            showCheckbox: widget.checkboxSelection,
            rowHeight: _rowHeight,
          );
    }

    // 空状態
    if (widget.rows.isEmpty) {
      return widget.emptyWidget ?? const K1s0DataTableEmpty();
    }

    // データ行
    final rows = widget.pagination
        ? _controller.currentPageRows
        : _controller.rows;

    return ListView.builder(
      itemCount: rows.length,
      itemBuilder: (context, index) {
        final row = rows[index];
        final isSelected = _controller.isSelected(row);

        return K1s0DataTableRow<T>(
          row: row,
          index: index,
          columns: widget.columns,
          isSelected: isSelected,
          showCheckbox: widget.checkboxSelection,
          onSelect: () => _controller.toggleRowSelection(row),
          onTap: widget.onRowTap != null ? () => widget.onRowTap!(row) : null,
          onDoubleTap: widget.onRowDoubleTap != null
              ? () => widget.onRowDoubleTap!(row)
              : null,
          rowHeight: _rowHeight,
        );
      },
    );
  }
}
