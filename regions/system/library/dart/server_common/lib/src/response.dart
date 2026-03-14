/// 標準 API レスポンスラッパー。
/// data フィールドにレスポンスデータを格納する。
class ApiResponse<T> {
  /// レスポンスデータ
  final T data;

  const ApiResponse({required this.data});

  /// JSON マップに変換する。
  /// T が toJson メソッドを持つ場合はそれを使用する。
  Map<String, dynamic> toJson(Object Function(T) toJsonT) => {
        'data': toJsonT(data),
      };
}

/// ページネーション付き API レスポンス。
/// data リストとページネーション情報を格納する。
class PaginatedResponse<T> {
  /// レスポンスデータリスト
  final List<T> data;

  /// 現在のページ番号
  final int page;

  /// 1ページあたりの件数
  final int perPage;

  /// 総件数
  final int total;

  /// 総ページ数
  final int totalPages;

  const PaginatedResponse({
    required this.data,
    required this.page,
    required this.perPage,
    required this.total,
    required this.totalPages,
  });

  /// JSON マップに変換する。
  Map<String, dynamic> toJson(Object Function(T) toJsonT) => {
        'data': data.map(toJsonT).toList(),
        'page': page,
        'per_page': perPage,
        'total': total,
        'total_pages': totalPages,
      };
}
