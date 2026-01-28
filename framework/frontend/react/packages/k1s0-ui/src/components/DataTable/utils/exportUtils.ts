/**
 * エクスポートユーティリティ
 */

import type { K1s0Column } from '../types.js';

/**
 * CSV エクスポートのオプション
 */
interface CsvExportOptions<T> {
  /** カラム定義 */
  columns: K1s0Column<T>[];
  /** データ行 */
  rows: T[];
  /** ファイル名（拡張子なし） */
  fileName?: string;
  /** 区切り文字 */
  delimiter?: string;
  /** UTF-8 BOM を追加（Excel 対応） */
  includeBom?: boolean;
}

/**
 * 値を CSV セーフな文字列に変換する
 */
function escapeCSVValue(value: unknown): string {
  if (value == null) return '';

  const str = String(value);

  // ダブルクォート、カンマ、改行を含む場合はダブルクォートで囲む
  if (str.includes('"') || str.includes(',') || str.includes('\n') || str.includes('\r')) {
    return `"${str.replace(/"/g, '""')}"`;
  }

  return str;
}

/**
 * データを CSV 形式でエクスポートする
 *
 * @param options - エクスポートオプション
 *
 * @example
 * ```tsx
 * exportToCsv({
 *   columns,
 *   rows: users,
 *   fileName: 'users',
 * });
 * ```
 */
export function exportToCsv<T extends { id: string | number }>(
  options: CsvExportOptions<T>
): void {
  const {
    columns,
    rows,
    fileName = 'export',
    delimiter = ',',
    includeBom = true,
  } = options;

  // エクスポート対象のカラムをフィルタ
  const exportableColumns = columns.filter((col) => col.exportable !== false);

  // ヘッダー行
  const headerRow = exportableColumns
    .map((col) => escapeCSVValue(col.headerName ?? col.field))
    .join(delimiter);

  // データ行
  const dataRows = rows.map((row) => {
    return exportableColumns
      .map((col) => {
        const value = row[col.field as keyof T];
        // valueFormatter が定義されていれば使用
        if (col.valueFormatter) {
          return escapeCSVValue(col.valueFormatter(value, row, col, {} as never));
        }
        return escapeCSVValue(value);
      })
      .join(delimiter);
  });

  // CSV 文字列を作成
  const csvContent = [headerRow, ...dataRows].join('\n');

  // BOM を追加（Excel で UTF-8 を正しく認識させるため）
  const bom = includeBom ? '\uFEFF' : '';
  const blob = new Blob([bom + csvContent], { type: 'text/csv;charset=utf-8;' });

  // ダウンロードリンクを作成
  const link = document.createElement('a');
  link.href = URL.createObjectURL(blob);
  link.download = `${fileName}.csv`;
  link.style.display = 'none';

  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);

  // URL を解放
  URL.revokeObjectURL(link.href);
}

/**
 * データを印刷用にフォーマットして印刷する
 *
 * @param options - エクスポートオプション
 */
export function printData<T extends { id: string | number }>(
  options: Omit<CsvExportOptions<T>, 'delimiter' | 'includeBom'>
): void {
  const { columns, rows, fileName = 'データ' } = options;

  // エクスポート対象のカラムをフィルタ
  const exportableColumns = columns.filter((col) => col.exportable !== false);

  // HTML テーブルを作成
  const tableHtml = `
    <!DOCTYPE html>
    <html>
    <head>
      <title>${fileName}</title>
      <style>
        body {
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
          padding: 20px;
        }
        h1 {
          font-size: 18px;
          margin-bottom: 10px;
        }
        table {
          border-collapse: collapse;
          width: 100%;
        }
        th, td {
          border: 1px solid #ddd;
          padding: 8px;
          text-align: left;
        }
        th {
          background-color: #f5f5f5;
          font-weight: 600;
        }
        tr:nth-child(even) {
          background-color: #fafafa;
        }
        @media print {
          body { padding: 0; }
          h1 { margin-bottom: 5px; }
        }
      </style>
    </head>
    <body>
      <h1>${fileName}</h1>
      <table>
        <thead>
          <tr>
            ${exportableColumns
              .map((col) => `<th>${col.headerName ?? col.field}</th>`)
              .join('')}
          </tr>
        </thead>
        <tbody>
          ${rows
            .map(
              (row) => `
            <tr>
              ${exportableColumns
                .map((col) => {
                  const value = row[col.field as keyof T];
                  const formatted = col.valueFormatter
                    ? col.valueFormatter(value, row, col, {} as never)
                    : value;
                  return `<td>${formatted ?? ''}</td>`;
                })
                .join('')}
            </tr>
          `
            )
            .join('')}
        </tbody>
      </table>
    </body>
    </html>
  `;

  // 新しいウィンドウを開いて印刷
  const printWindow = window.open('', '_blank');
  if (printWindow) {
    printWindow.document.write(tableHtml);
    printWindow.document.close();
    printWindow.print();
  }
}
