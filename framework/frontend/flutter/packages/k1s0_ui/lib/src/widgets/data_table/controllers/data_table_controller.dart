/// K1s0 DataTable コントローラー
library;

import 'package:flutter/foundation.dart';
import '../k1s0_sort_model.dart';
import '../k1s0_selection.dart';

/// DataTable の状態を管理するコントローラー
class K1s0DataTableController<T> extends ChangeNotifier {
  /// 元のデータ
  List<T> _rows;

  /// 行 ID 取得関数
  final String Function(T row) getRowId;

  /// ソートモデル
  List<K1s0SortItem> _sortModel;

  /// 選択モード
  final K1s0SelectionMode selectionMode;

  /// 選択された行 ID
  Set<String> _selectedIds;

  /// 現在のページ
  int _page;

  /// ページサイズ
  int _pageSize;

  /// コンストラクタ
  K1s0DataTableController({
    required List<T> rows,
    required this.getRowId,
    List<K1s0SortItem>? initialSortModel,
    this.selectionMode = K1s0SelectionMode.none,
    Set<String>? initialSelection,
    int initialPage = 0,
    int initialPageSize = 20,
  })  : _rows = rows,
        _sortModel = initialSortModel ?? [],
        _selectedIds = initialSelection ?? {},
        _page = initialPage,
        _pageSize = initialPageSize;

  /// データ行
  List<T> get rows => _rows;

  /// データを更新
  set rows(List<T> value) {
    _rows = value;
    notifyListeners();
  }

  /// ソートモデル
  List<K1s0SortItem> get sortModel => _sortModel;

  /// ソートモデルを更新
  set sortModel(List<K1s0SortItem> value) {
    _sortModel = value;
    notifyListeners();
  }

  /// 選択された行 ID
  Set<String> get selectedIds => Set.unmodifiable(_selectedIds);

  /// 選択された行データ
  List<T> get selectedRows =>
      _rows.where((row) => _selectedIds.contains(getRowId(row))).toList();

  /// 選択されているかどうか
  bool isSelected(T row) => _selectedIds.contains(getRowId(row));

  /// 全選択されているかどうか
  bool get isAllSelected =>
      _rows.isNotEmpty && _selectedIds.length == _rows.length;

  /// 一部選択されているかどうか
  bool get isIndeterminate =>
      _selectedIds.isNotEmpty && _selectedIds.length < _rows.length;

  /// 現在のページ
  int get page => _page;

  /// ページを更新
  set page(int value) {
    _page = value;
    notifyListeners();
  }

  /// ページサイズ
  int get pageSize => _pageSize;

  /// ページサイズを更新
  set pageSize(int value) {
    _pageSize = value;
    _page = 0; // ページをリセット
    notifyListeners();
  }

  /// 総ページ数
  int get totalPages => (_rows.length / _pageSize).ceil();

  /// 現在のページのデータ
  List<T> get currentPageRows {
    final sortedRows = _applySorting(_rows);
    final start = _page * _pageSize;
    final end = (start + _pageSize).clamp(0, sortedRows.length);
    return sortedRows.sublist(start, end);
  }

  /// ソートを適用
  List<T> _applySorting(List<T> data) {
    if (_sortModel.isEmpty) return data;

    final sorted = List<T>.from(data);
    sorted.sort((a, b) {
      for (final sortItem in _sortModel) {
        final aValue = _getFieldValue(a, sortItem.field);
        final bValue = _getFieldValue(b, sortItem.field);

        int comparison;
        if (aValue == null && bValue == null) {
          comparison = 0;
        } else if (aValue == null) {
          comparison = -1;
        } else if (bValue == null) {
          comparison = 1;
        } else if (aValue is Comparable && bValue is Comparable) {
          comparison = aValue.compareTo(bValue);
        } else {
          comparison = aValue.toString().compareTo(bValue.toString());
        }

        if (comparison != 0) {
          return sortItem.sort == K1s0SortOrder.asc ? comparison : -comparison;
        }
      }
      return 0;
    });

    return sorted;
  }

  /// フィールド値を取得
  dynamic _getFieldValue(T row, String field) {
    if (row is Map) {
      return row[field];
    }
    return null;
  }

  /// 行を選択
  void selectRow(T row) {
    if (selectionMode == K1s0SelectionMode.none) return;

    final id = getRowId(row);
    if (selectionMode == K1s0SelectionMode.single) {
      _selectedIds = {id};
    } else {
      _selectedIds.add(id);
    }
    notifyListeners();
  }

  /// 行の選択を解除
  void deselectRow(T row) {
    final id = getRowId(row);
    _selectedIds.remove(id);
    notifyListeners();
  }

  /// 行の選択を切り替え
  void toggleRowSelection(T row) {
    if (isSelected(row)) {
      deselectRow(row);
    } else {
      selectRow(row);
    }
  }

  /// 全選択
  void selectAll() {
    if (selectionMode == K1s0SelectionMode.multiple) {
      _selectedIds = _rows.map(getRowId).toSet();
      notifyListeners();
    }
  }

  /// 全選択解除
  void deselectAll() {
    _selectedIds.clear();
    notifyListeners();
  }

  /// 全選択を切り替え
  void toggleSelectAll() {
    if (isAllSelected) {
      deselectAll();
    } else {
      selectAll();
    }
  }

  /// 選択をセット
  void setSelection(Set<String> ids) {
    _selectedIds = Set.from(ids);
    notifyListeners();
  }

  /// ソートを追加/更新
  void setSort(String field, K1s0SortOrder order) {
    _sortModel = [K1s0SortItem(field: field, sort: order)];
    notifyListeners();
  }

  /// ソートを切り替え
  void toggleSort(String field) {
    final existing = _sortModel.firstWhere(
      (item) => item.field == field,
      orElse: () => K1s0SortItem(field: field, sort: K1s0SortOrder.desc),
    );

    if (_sortModel.any((item) => item.field == field)) {
      if (existing.sort == K1s0SortOrder.asc) {
        // 昇順 → 降順
        _sortModel = [existing.reversed];
      } else {
        // 降順 → ソート解除
        _sortModel = [];
      }
    } else {
      // 未ソート → 昇順
      _sortModel = [K1s0SortItem(field: field, sort: K1s0SortOrder.asc)];
    }
    notifyListeners();
  }

  /// ソートをクリア
  void clearSort() {
    _sortModel = [];
    notifyListeners();
  }

  /// 次のページ
  void nextPage() {
    if (_page < totalPages - 1) {
      _page++;
      notifyListeners();
    }
  }

  /// 前のページ
  void previousPage() {
    if (_page > 0) {
      _page--;
      notifyListeners();
    }
  }

  /// 指定ページに移動
  void goToPage(int pageIndex) {
    if (pageIndex >= 0 && pageIndex < totalPages) {
      _page = pageIndex;
      notifyListeners();
    }
  }
}
