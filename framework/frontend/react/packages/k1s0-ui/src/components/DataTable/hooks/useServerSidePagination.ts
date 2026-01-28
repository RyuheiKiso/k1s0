/**
 * サーバーサイドページネーションフック
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import type {
  GridSortModel,
  GridFilterModel,
  GridPaginationModel,
} from '@mui/x-data-grid';
import type {
  UseServerSidePaginationOptions,
  UseServerSidePaginationReturn,
  ServerSidePaginationParams,
} from '../types.js';

/**
 * サーバーサイドページネーション/ソート/フィルタを管理するフック
 *
 * @param options - フックのオプション
 * @returns データと状態管理関数
 *
 * @example
 * ```tsx
 * function UserList() {
 *   const {
 *     rows,
 *     rowCount,
 *     loading,
 *     paginationModel,
 *     setPaginationModel,
 *     sortModel,
 *     setSortModel,
 *     refetch,
 *   } = useServerSidePagination<User>({
 *     fetchFn: async (params) => {
 *       const response = await api.getUsers({
 *         page: params.page,
 *         pageSize: params.pageSize,
 *         sortField: params.sortModel[0]?.field,
 *         sortOrder: params.sortModel[0]?.sort,
 *       });
 *       return {
 *         rows: response.data,
 *         rowCount: response.total,
 *       };
 *     },
 *     initialPageSize: 20,
 *   });
 *
 *   return (
 *     <K1s0DataTable
 *       rows={rows}
 *       columns={columns}
 *       loading={loading}
 *       paginationMode="server"
 *       sortingMode="server"
 *       rowCount={rowCount}
 *       paginationModel={paginationModel}
 *       onPaginationModelChange={setPaginationModel}
 *       sortModel={sortModel}
 *       onSortModelChange={setSortModel}
 *     />
 *   );
 * }
 * ```
 */
export function useServerSidePagination<T extends { id: string | number }>(
  options: UseServerSidePaginationOptions<T>
): UseServerSidePaginationReturn<T> {
  const {
    fetchFn,
    initialPageSize = 20,
    initialPage = 0,
    deps = [],
  } = options;

  // 状態
  const [rows, setRows] = useState<T[]>([]);
  const [rowCount, setRowCount] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  // ページネーション状態
  const [paginationModel, setPaginationModel] = useState<GridPaginationModel>({
    page: initialPage,
    pageSize: initialPageSize,
  });

  // ソート状態
  const [sortModel, setSortModel] = useState<GridSortModel>([]);

  // フィルタ状態
  const [filterModel, setFilterModel] = useState<GridFilterModel>({ items: [] });

  // 最新の fetch パラメータを追跡
  const latestParamsRef = useRef<ServerSidePaginationParams | null>(null);

  // データ取得関数
  const fetchData = useCallback(async () => {
    const params: ServerSidePaginationParams = {
      page: paginationModel.page,
      pageSize: paginationModel.pageSize,
      sortModel,
      filterModel,
    };

    // パラメータを保存
    latestParamsRef.current = params;

    setLoading(true);
    setError(null);

    try {
      const result = await fetchFn(params);

      // 最新のリクエストの結果のみを適用
      if (latestParamsRef.current === params) {
        setRows(result.rows);
        setRowCount(result.rowCount);
      }
    } catch (err) {
      // 最新のリクエストの結果のみを適用
      if (latestParamsRef.current === params) {
        setError(err instanceof Error ? err : new Error(String(err)));
        setRows([]);
        setRowCount(0);
      }
    } finally {
      // 最新のリクエストの結果のみを適用
      if (latestParamsRef.current === params) {
        setLoading(false);
      }
    }
  }, [fetchFn, paginationModel, sortModel, filterModel]);

  // 依存値が変更されたらデータを再取得
  useEffect(() => {
    fetchData();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [paginationModel, sortModel, filterModel, ...deps]);

  // 手動リフェッチ
  const refetch = useCallback(() => {
    fetchData();
  }, [fetchData]);

  // ソートモデル変更時にページを0にリセット
  const handleSortModelChange = useCallback((model: GridSortModel) => {
    setSortModel(model);
    setPaginationModel((prev) => ({ ...prev, page: 0 }));
  }, []);

  // フィルタモデル変更時にページを0にリセット
  const handleFilterModelChange = useCallback((model: GridFilterModel) => {
    setFilterModel(model);
    setPaginationModel((prev) => ({ ...prev, page: 0 }));
  }, []);

  return {
    rows,
    rowCount,
    loading,
    error,
    paginationModel,
    setPaginationModel,
    sortModel,
    setSortModel: handleSortModelChange,
    filterModel,
    setFilterModel: handleFilterModelChange,
    refetch,
  };
}
