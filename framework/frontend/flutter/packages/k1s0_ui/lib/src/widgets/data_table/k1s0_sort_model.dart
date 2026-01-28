/// K1s0 DataTable ソートモデル
library;

/// ソート方向
enum K1s0SortOrder {
  /// 昇順
  asc,

  /// 降順
  desc,
}

/// ソート項目
class K1s0SortItem {
  /// フィールド名
  final String field;

  /// ソート方向
  final K1s0SortOrder sort;

  const K1s0SortItem({
    required this.field,
    required this.sort,
  });

  /// 逆順を取得
  K1s0SortItem get reversed => K1s0SortItem(
        field: field,
        sort: sort == K1s0SortOrder.asc ? K1s0SortOrder.desc : K1s0SortOrder.asc,
      );

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is K1s0SortItem &&
          runtimeType == other.runtimeType &&
          field == other.field &&
          sort == other.sort;

  @override
  int get hashCode => field.hashCode ^ sort.hashCode;

  @override
  String toString() => 'K1s0SortItem(field: $field, sort: $sort)';
}
