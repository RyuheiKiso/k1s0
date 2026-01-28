/**
 * K1s0 DataTable カスタムツールバー
 */

import React from 'react';
import {
  GridToolbarContainer,
  GridToolbarColumnsButton,
  GridToolbarFilterButton,
  GridToolbarDensitySelector,
} from '@mui/x-data-grid';
import Button from '@mui/material/Button';
import FileDownloadIcon from '@mui/icons-material/FileDownload';
import PrintIcon from '@mui/icons-material/Print';
import type { ExportOptions, K1s0Column } from './types.js';
import { exportToCsv, printData } from './utils/exportUtils.js';

/**
 * ツールバー Props
 */
interface K1s0DataTableToolbarProps<T extends { id: string | number }> {
  /** エクスポートオプション */
  exportOptions?: ExportOptions;
  /** カラム定義 */
  columns: K1s0Column<T>[];
  /** データ行 */
  rows: T[];
  /** 追加のツールバー要素 */
  children?: React.ReactNode;
}

/**
 * K1s0 DataTable カスタムツールバー
 *
 * MUI DataGrid のツールバーをカスタマイズしたコンポーネント。
 * カラム表示切替、フィルタ、行の高さ、エクスポート機能を提供。
 */
export function K1s0DataTableToolbar<T extends { id: string | number }>(
  props: K1s0DataTableToolbarProps<T>
): React.ReactElement {
  const { exportOptions, columns, rows, children } = props;

  const handleCsvExport = () => {
    exportToCsv({
      columns,
      rows,
      fileName: exportOptions?.fileName ?? 'export',
    });
  };

  const handlePrint = () => {
    printData({
      columns,
      rows,
      fileName: exportOptions?.fileName ?? 'データ',
    });
  };

  return (
    <GridToolbarContainer sx={{ gap: 1, p: 1 }}>
      <GridToolbarColumnsButton />
      <GridToolbarFilterButton />
      <GridToolbarDensitySelector />

      {exportOptions?.csv && (
        <Button
          size="small"
          startIcon={<FileDownloadIcon />}
          onClick={handleCsvExport}
        >
          CSV
        </Button>
      )}

      {exportOptions?.print && (
        <Button
          size="small"
          startIcon={<PrintIcon />}
          onClick={handlePrint}
        >
          印刷
        </Button>
      )}

      {children}
    </GridToolbarContainer>
  );
}
