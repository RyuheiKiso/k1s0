/**
 * 標準 API レスポンスラッパー。
 * 単一のリソースを `data` フィールドで包む。
 */
export interface ApiResponse<T> {
  /** レスポンスデータ */
  data: T;
}

/**
 * ページネーション付き API レスポンス。
 * リスト系エンドポイントで使用する。
 */
export interface PaginatedResponse<T> {
  /** レスポンスデータの配列 */
  data: T[];
  /** 現在のページ番号 */
  page: number;
  /** 1ページあたりの件数 */
  perPage: number;
  /** 総件数 */
  total: number;
  /** 総ページ数 */
  totalPages: number;
}
