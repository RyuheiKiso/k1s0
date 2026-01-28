/**
 * K1s0 DataTable 状態管理フック
 */

import { useState, useCallback, useMemo } from 'react';
import type {
  GridRowSelectionModel,
  GridSortModel,
  GridFilterModel,
  GridPaginationModel,
} from '@mui/x-data-grid';
import type { UseK1s0DataTableReturn } from '../types.js';

/**
 * DataTable の状態管理オプション
 */
interface UseK1s0DataTableOptions<T> {
  /** データ行 */
  rows: T[];
  /** 初期ページサイズ */
  initialPageSize?: number;
  /** 初期ソートモデル */
  initialSortModel?: GridSortModel;
  /** 初期フィルタモデル */
  initialFilterModel?: GridFilterModel;
  /** 初期選択行 */
  initialSelection?: GridRowSelectionModel;
  /** 行の ID を取得する関数（デフォルト: row.id） */
  getRowId?: (row: T) => string | number;
}

/**
 * K1s0 DataTable の状態を管理するフック
 *
 * @param options - フックのオプション
 * @returns 状態と状態更新関数
 *
 * @example
 * ```tsx
 * function UserList() {
 *   const users = useUsers();
 *
 *   const {
 *     selectedRowIds,
 *     selectedRows,
 *     clearSelection,
 *     sortModel,
 *     setSortModel,
 *     paginationModel,
 *     setPaginationModel,
 *   } = useK1s0DataTable({
 *     rows: users,
 *     initialPageSize: 20,
 *   });
 *
 *   return (
 *     <K1s0DataTable
 *       rows={users}
 *       columns={columns}
 *       rowSelectionModel={selectedRowIds}
 *       onRowSelectionModelChange={(model) => setSelectedRowIds(model)}
 *       sortModel={sortModel}
 *       onSortModelChange={(model) => setSortModel(model)}
 *       paginationModel={paginationModel}
 *       onPaginationModelChange={(model) => setPaginationModel(model)}
 *     />
 *   );
 * }
 * ```
 */
export function useK1s0DataTable<T extends { id: string | number }>(
  options: UseK1s0DataTableOptions<T>
): UseK1s0DataTableReturn<T> {
  const {
    rows,
    initialPageSize = 20,
    initialSortModel = [],
    initialFilterModel = { items: [] },
    initialSelection = [],
    getRowId = (row: T) => row.id,
  } = options;

  // 選択状態
  const [selectedRowIds, setSelectedRowIds] =
    useState<GridRowSelectionModel>(initialSelection);

  // ソート状態
  const [sortModel, setSortModel] = useState<GridSortModel>(initialSortModel);

  // フィルタ状態
  const [filterModel, setFilterModel] = useState<GridFilterModel>(initialFilterModel);

  // ページネーション状態
  const [paginationModel, setPaginationModel] = useState<GridPaginationModel>({
    page: 0,
    pageSize: initialPageSize,
  });

  // 選択された行データを計算
  const selectedRows = useMemo(() => {
    const idSet = new Set(selectedRowIds);
    return rows.filter((row) => idSet.has(getRowId(row)));
  }, [rows, selectedRowIds, getRowId]);

  // 選択をクリア
  const clearSelection = useCallback(() => {
    setSelectedRowIds([]);
  }, []);

  // 全選択
  const selectAll = useCallback(() => {
    setSelectedRowIds(rows.map(getRowId));
  }, [rows, getRowId]);

  return {
    selectedRowIds,
    selectedRows,
    clearSelection,
    selectAll,
    sortModel,
    setSortModel,
    filterModel,
    setFilterModel,
    paginationModel,
    setPaginationModel,
  };
}
