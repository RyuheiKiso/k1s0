/**
 * 数値カラムヘルパー
 */

import type { K1s0Column, NumberColumnOptions } from '../types.js';

/**
 * 数値フォーマットのオプション
 */
interface FormatNumberOptions {
  decimalPlaces?: number;
  prefix?: string;
  suffix?: string;
  thousandSeparator?: boolean;
}

/**
 * 数値をフォーマットする
 */
function formatNumber(value: number, options: FormatNumberOptions): string {
  const { decimalPlaces = 0, prefix = '', suffix = '', thousandSeparator = true } = options;

  let formatted = value.toFixed(decimalPlaces);

  if (thousandSeparator) {
    const parts = formatted.split('.');
    parts[0] = parts[0].replace(/\B(?=(\d{3})+(?!\d))/g, ',');
    formatted = parts.join('.');
  }

  return `${prefix}${formatted}${suffix}`;
}

/**
 * 数値フォーマット済みカラムを作成する
 *
 * @param options - 数値カラムのオプション
 * @returns K1s0Column
 *
 * @example
 * ```tsx
 * const columns = [
 *   numberColumn<Product>({ field: 'price', headerName: '価格', prefix: '¥' }),
 *   numberColumn<Product>({ field: 'stock', headerName: '在庫', suffix: '個' }),
 *   numberColumn<Product>({ field: 'rate', headerName: '評価', decimalPlaces: 2 }),
 * ];
 * ```
 */
export function numberColumn<T extends { id: string | number }>(
  options: NumberColumnOptions<T>
): K1s0Column<T> {
  const {
    field,
    headerName = '数値',
    decimalPlaces = 0,
    prefix = '',
    suffix = '',
    thousandSeparator = true,
    width = 100,
    sortable = true,
    align = 'right',
  } = options;

  return {
    field,
    headerName,
    width,
    sortable,
    type: 'number',
    align,
    headerAlign: align,
    valueFormatter: (value: unknown) => {
      if (value == null) return '';
      const num = typeof value === 'number' ? value : parseFloat(String(value));
      if (isNaN(num)) return '';
      return formatNumber(num, { decimalPlaces, prefix, suffix, thousandSeparator });
    },
  };
}

/**
 * 通貨フォーマット済みカラムを作成する
 *
 * @param options - 数値カラムのオプション（prefix はデフォルトで '¥'）
 * @returns K1s0Column
 */
export function currencyColumn<T extends { id: string | number }>(
  options: Omit<NumberColumnOptions<T>, 'prefix'> & { prefix?: string }
): K1s0Column<T> {
  return numberColumn({
    ...options,
    prefix: options.prefix ?? '¥',
    decimalPlaces: options.decimalPlaces ?? 0,
  });
}

/**
 * パーセンテージフォーマット済みカラムを作成する
 *
 * @param options - 数値カラムのオプション（suffix はデフォルトで '%'）
 * @returns K1s0Column
 */
export function percentColumn<T extends { id: string | number }>(
  options: Omit<NumberColumnOptions<T>, 'suffix'> & { suffix?: string }
): K1s0Column<T> {
  return numberColumn({
    ...options,
    suffix: options.suffix ?? '%',
    decimalPlaces: options.decimalPlaces ?? 1,
    thousandSeparator: options.thousandSeparator ?? false,
  });
}
