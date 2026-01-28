/**
 * カラム定義ヘルパー
 */

import type { K1s0Column } from '../types.js';

/**
 * 型安全なカラム定義を作成する
 *
 * @param columns - カラム定義の配列
 * @returns K1s0Column 配列
 *
 * @example
 * ```tsx
 * interface User {
 *   id: string;
 *   name: string;
 *   email: string;
 * }
 *
 * const columns = createColumns<User>([
 *   { field: 'name', headerName: '氏名', flex: 1 },
 *   { field: 'email', headerName: 'メール', flex: 1 },
 * ]);
 * ```
 */
export function createColumns<T extends { id: string | number }>(
  columns: K1s0Column<T>[]
): K1s0Column<T>[] {
  return columns.map((column) => ({
    ...column,
    // デフォルト値を設定
    sortable: column.sortable ?? true,
    filterable: column.filterable ?? true,
    exportable: column.exportable ?? true,
    disableColumnMenu: column.disableColumnMenu ?? false,
  }));
}
