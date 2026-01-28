/**
 * Boolean カラムヘルパー
 */

import React from 'react';
import CheckIcon from '@mui/icons-material/Check';
import CloseIcon from '@mui/icons-material/Close';
import type { K1s0Column, BooleanColumnOptions } from '../types.js';

/**
 * Boolean カラムを作成する
 *
 * @param options - Boolean カラムのオプション
 * @returns K1s0Column
 *
 * @example
 * ```tsx
 * const columns = [
 *   booleanColumn<User>({ field: 'isActive', headerName: '有効' }),
 *   booleanColumn<User>({
 *     field: 'isAdmin',
 *     headerName: '管理者',
 *     trueLabel: '管理者',
 *     falseLabel: '一般',
 *   }),
 * ];
 * ```
 */
export function booleanColumn<T extends { id: string | number }>(
  options: BooleanColumnOptions<T>
): K1s0Column<T> {
  const {
    field,
    headerName = '',
    trueLabel = 'はい',
    falseLabel = 'いいえ',
    showIcon = true,
    width = 100,
  } = options;

  return {
    field,
    headerName,
    width,
    type: 'boolean',
    sortable: true,
    align: 'center',
    headerAlign: 'center',
    renderCell: (params) => {
      const value = params.value as boolean | null | undefined;

      if (value == null) {
        return <span style={{ color: '#999' }}>-</span>;
      }

      if (showIcon) {
        return value ? (
          <CheckIcon color="success" fontSize="small" />
        ) : (
          <CloseIcon color="error" fontSize="small" />
        );
      }

      return (
        <span style={{ color: value ? '#2e7d32' : '#d32f2f' }}>
          {value ? trueLabel : falseLabel}
        </span>
      );
    },
    valueFormatter: (value: unknown) => {
      if (value == null) return '';
      return value ? trueLabel : falseLabel;
    },
  };
}
