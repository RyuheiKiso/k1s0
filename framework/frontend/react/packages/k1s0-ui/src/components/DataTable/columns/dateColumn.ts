/**
 * 日付カラムヘルパー
 */

import dayjs from 'dayjs';
import type { K1s0Column, DateColumnOptions } from '../types.js';

/**
 * 日付フォーマット済みカラムを作成する
 *
 * @param options - 日付カラムのオプション
 * @returns K1s0Column
 *
 * @example
 * ```tsx
 * const columns = [
 *   dateColumn<User>({ field: 'createdAt', headerName: '作成日' }),
 *   dateColumn<User>({ field: 'updatedAt', headerName: '更新日', format: 'YYYY/MM/DD HH:mm' }),
 * ];
 * ```
 */
export function dateColumn<T extends { id: string | number }>(
  options: DateColumnOptions<T>
): K1s0Column<T> {
  const {
    field,
    headerName = '日付',
    format = 'YYYY/MM/DD',
    width = 120,
    sortable = true,
  } = options;

  return {
    field,
    headerName,
    width,
    sortable,
    type: 'date',
    valueFormatter: (value: unknown) => {
      if (value == null) return '';
      const date = dayjs(value as string | number | Date);
      return date.isValid() ? date.format(format) : '';
    },
    valueGetter: (value: unknown) => {
      if (value == null) return null;
      const date = dayjs(value as string | number | Date);
      return date.isValid() ? date.toDate() : null;
    },
  };
}

/**
 * 日時フォーマット済みカラムを作成する
 *
 * @param options - 日付カラムのオプション
 * @returns K1s0Column
 */
export function dateTimeColumn<T extends { id: string | number }>(
  options: DateColumnOptions<T>
): K1s0Column<T> {
  return dateColumn({
    ...options,
    format: options.format ?? 'YYYY/MM/DD HH:mm',
    width: options.width ?? 150,
  });
}
