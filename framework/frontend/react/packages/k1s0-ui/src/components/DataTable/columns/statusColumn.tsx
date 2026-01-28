/**
 * ステータスカラムヘルパー
 */

import React from 'react';
import Chip from '@mui/material/Chip';
import type { GridRenderCellParams } from '@mui/x-data-grid';
import type { K1s0Column, StatusColumnOptions } from '../types.js';

/**
 * ステータスバッジ（Chip）カラムを作成する
 *
 * @param options - ステータスカラムのオプション
 * @returns K1s0Column
 *
 * @example
 * ```tsx
 * const columns = [
 *   statusColumn<Order>({
 *     field: 'status',
 *     headerName: 'ステータス',
 *     colorMap: {
 *       pending: 'warning',
 *       processing: 'info',
 *       completed: 'success',
 *       cancelled: 'error',
 *     },
 *     labelMap: {
 *       pending: '保留中',
 *       processing: '処理中',
 *       completed: '完了',
 *       cancelled: 'キャンセル',
 *     },
 *   }),
 * ];
 * ```
 */
export function statusColumn<
  T extends { id: string | number },
  V extends string = string
>(options: StatusColumnOptions<T, V>): K1s0Column<T> {
  const {
    field,
    headerName = 'ステータス',
    colorMap = {},
    labelMap = {},
    width = 120,
  } = options;

  return {
    field,
    headerName,
    width,
    sortable: true,
    align: 'center',
    headerAlign: 'center',
    renderCell: (params: GridRenderCellParams) => {
      const value = params.value as V | null | undefined;

      if (value == null) {
        return <span style={{ color: '#999' }}>-</span>;
      }

      const color = colorMap[value] ?? 'default';
      const label = labelMap[value] ?? value;

      return (
        <Chip
          label={label}
          color={color}
          size="small"
          variant="filled"
          sx={{ minWidth: 60 }}
        />
      );
    },
    valueFormatter: (value: unknown) => {
      if (value == null) return '';
      return labelMap[value as V] ?? String(value);
    },
  };
}
