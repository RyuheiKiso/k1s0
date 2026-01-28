/**
 * K1s0 DataTable コンポーネント
 *
 * MUI DataGrid をラップした k1s0 標準テーブルコンポーネント
 */

import React, { useMemo, useCallback } from 'react';
import {
  DataGrid,
  GridColDef,
  GridRowParams,
  GridCallbackDetails,
  GridRowSelectionModel,
  GridSortModel,
  GridFilterModel,
  GridPaginationModel,
} from '@mui/x-data-grid';
import Box from '@mui/material/Box';
import type { K1s0DataTableProps, K1s0Column } from './types.js';
import { jaJP } from './locales/jaJP.js';
import { K1s0DataTableToolbar } from './K1s0DataTableToolbar.js';

/**
 * K1s0Column を MUI GridColDef に変換する
 */
function toGridColDef<T>(column: K1s0Column<T>): GridColDef {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const { filterOptions, exportable, ...gridColDef } = column;
  return gridColDef as GridColDef;
}

/**
 * K1s0 DataTable
 *
 * MUI DataGrid をベースにした高機能テーブルコンポーネント。
 * ソート、フィルタ、ページネーション、選択、エクスポート機能を提供。
 *
 * @example
 * ```tsx
 * <K1s0DataTable
 *   rows={users}
 *   columns={columns}
 *   checkboxSelection
 *   pagination
 *   pageSize={20}
 *   toolbar
 *   exportOptions={{ csv: true }}
 *   onRowClick={(user) => navigate(`/users/${user.id}`)}
 * />
 * ```
 */
export function K1s0DataTable<T extends { id: string | number }>(
  props: K1s0DataTableProps<T>
): React.ReactElement {
  const {
    rows,
    columns,
    pagination = true,
    pageSize = 20,
    pageSizeOptions = [10, 20, 50, 100],
    paginationMode = 'client',
    paginationModel,
    onPaginationModelChange,
    rowCount,
    sortModel,
    onSortModelChange,
    sortingMode = 'client',
    filterModel,
    onFilterModelChange,
    filterMode = 'client',
    checkboxSelection = false,
    rowSelectionModel,
    onRowSelectionModelChange,
    isRowSelectable,
    loading = false,
    onRowClick,
    onRowDoubleClick,
    density = 'standard',
    autoHeight = false,
    height = 400,
    toolbar = false,
    exportOptions,
    treeData = false,
    getTreeDataPath,
    noRowsOverlay,
    loadingOverlay,
    sx,
    className,
    getRowClassName,
    getCellClassName,
    localeText,
    disableJapaneseLocale = false,
  } = props;

  // カラムを MUI 形式に変換
  const gridColumns = useMemo(() => {
    return columns.map(toGridColDef);
  }, [columns]);

  // ローカライズテキストをマージ
  const mergedLocaleText = useMemo(() => {
    if (disableJapaneseLocale) {
      return localeText;
    }
    return {
      ...jaJP,
      ...localeText,
    };
  }, [localeText, disableJapaneseLocale]);

  // 行クリックハンドラ
  const handleRowClick = useCallback(
    (params: GridRowParams<T>) => {
      onRowClick?.(params.row);
    },
    [onRowClick]
  );

  // 行ダブルクリックハンドラ
  const handleRowDoubleClick = useCallback(
    (params: GridRowParams<T>) => {
      onRowDoubleClick?.(params.row);
    },
    [onRowDoubleClick]
  );

  // ページネーションモデル変更ハンドラ
  const handlePaginationModelChange = useCallback(
    (model: GridPaginationModel, details: GridCallbackDetails) => {
      onPaginationModelChange?.(model, details);
    },
    [onPaginationModelChange]
  );

  // ソートモデル変更ハンドラ
  const handleSortModelChange = useCallback(
    (model: GridSortModel, details: GridCallbackDetails) => {
      onSortModelChange?.(model, details);
    },
    [onSortModelChange]
  );

  // フィルタモデル変更ハンドラ
  const handleFilterModelChange = useCallback(
    (model: GridFilterModel, details: GridCallbackDetails) => {
      onFilterModelChange?.(model, details);
    },
    [onFilterModelChange]
  );

  // 選択モデル変更ハンドラ
  const handleRowSelectionModelChange = useCallback(
    (model: GridRowSelectionModel, details: GridCallbackDetails) => {
      onRowSelectionModelChange?.(model, details);
    },
    [onRowSelectionModelChange]
  );

  // ツールバーをレンダリング
  const renderToolbar = useCallback(() => {
    if (!toolbar) return null;

    if (typeof toolbar !== 'boolean') {
      return toolbar;
    }

    return (
      <K1s0DataTableToolbar
        exportOptions={exportOptions}
        columns={columns}
        rows={rows}
      />
    );
  }, [toolbar, exportOptions, columns, rows]);

  // カスタムスロット
  const slots = useMemo(() => {
    const result: Record<string, unknown> = {};

    if (toolbar) {
      result.toolbar = renderToolbar;
    }

    if (noRowsOverlay) {
      result.noRowsOverlay = () => noRowsOverlay;
    }

    if (loadingOverlay) {
      result.loadingOverlay = () => loadingOverlay;
    }

    return Object.keys(result).length > 0 ? result : undefined;
  }, [toolbar, renderToolbar, noRowsOverlay, loadingOverlay]);

  // 内部ページネーションモデル（制御/非制御モード対応）
  const internalPaginationModel = useMemo(() => {
    return (
      paginationModel ?? {
        page: 0,
        pageSize,
      }
    );
  }, [paginationModel, pageSize]);

  return (
    <Box
      sx={{
        width: '100%',
        height: autoHeight ? 'auto' : height,
        ...sx,
      }}
      className={className}
    >
      <DataGrid
        rows={rows}
        columns={gridColumns}
        // ページネーション
        pagination={pagination}
        paginationMode={paginationMode}
        paginationModel={internalPaginationModel}
        onPaginationModelChange={handlePaginationModelChange}
        pageSizeOptions={pageSizeOptions}
        rowCount={paginationMode === 'server' ? rowCount : undefined}
        // ソート
        sortModel={sortModel}
        onSortModelChange={handleSortModelChange}
        sortingMode={sortingMode}
        // フィルタ
        filterModel={filterModel}
        onFilterModelChange={handleFilterModelChange}
        filterMode={filterMode}
        // 選択
        checkboxSelection={checkboxSelection}
        rowSelectionModel={rowSelectionModel}
        onRowSelectionModelChange={handleRowSelectionModelChange}
        isRowSelectable={isRowSelectable}
        disableRowSelectionOnClick={!!onRowClick}
        // イベント
        onRowClick={onRowClick ? handleRowClick : undefined}
        onRowDoubleClick={onRowDoubleClick ? handleRowDoubleClick : undefined}
        // 表示
        loading={loading}
        density={density}
        autoHeight={autoHeight}
        // ツリーデータ
        treeData={treeData}
        getTreeDataPath={getTreeDataPath}
        // スタイル
        getRowClassName={getRowClassName}
        getCellClassName={
          getCellClassName
            ? (params) =>
                getCellClassName({ row: params.row as T, field: params.field })
            : undefined
        }
        // ローカライズ
        localeText={mergedLocaleText}
        // スロット
        slots={slots}
        // その他
        disableColumnResize={false}
        disableColumnSelector={false}
        sx={{
          border: 0,
          '& .MuiDataGrid-cell:focus': {
            outline: 'none',
          },
          '& .MuiDataGrid-cell:focus-within': {
            outline: 'none',
          },
          '& .MuiDataGrid-columnHeader:focus': {
            outline: 'none',
          },
          '& .MuiDataGrid-columnHeader:focus-within': {
            outline: 'none',
          },
        }}
      />
    </Box>
  );
}
