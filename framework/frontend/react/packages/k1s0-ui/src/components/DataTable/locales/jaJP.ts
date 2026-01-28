/**
 * K1s0 DataTable 日本語ローカライズ
 */

import type { GridLocaleText } from '@mui/x-data-grid';

export const jaJP: Partial<GridLocaleText> = {
  // Root
  noRowsLabel: 'データがありません',
  noResultsOverlayLabel: '検索結果がありません',

  // Density selector toolbar button text
  toolbarDensity: '行の高さ',
  toolbarDensityLabel: '行の高さ',
  toolbarDensityCompact: 'コンパクト',
  toolbarDensityStandard: '標準',
  toolbarDensityComfortable: 'ゆったり',

  // Columns selector toolbar button text
  toolbarColumns: '列',
  toolbarColumnsLabel: '列の表示設定',

  // Filters toolbar button text
  toolbarFilters: 'フィルタ',
  toolbarFiltersLabel: 'フィルタを表示',
  toolbarFiltersTooltipHide: 'フィルタを非表示',
  toolbarFiltersTooltipShow: 'フィルタを表示',
  toolbarFiltersTooltipActive: (count) =>
    count > 1 ? `${count}件のフィルタが有効` : `${count}件のフィルタが有効`,

  // Quick filter toolbar field
  toolbarQuickFilterPlaceholder: '検索…',
  toolbarQuickFilterLabel: '検索',
  toolbarQuickFilterDeleteIconLabel: 'クリア',

  // Export selector toolbar button text
  toolbarExport: 'エクスポート',
  toolbarExportLabel: 'エクスポート',
  toolbarExportCSV: 'CSVダウンロード',
  toolbarExportPrint: '印刷',
  toolbarExportExcel: 'Excelダウンロード',

  // Columns management text
  columnsManagementSearchTitle: '検索',
  columnsManagementNoColumns: '表示する列がありません',
  columnsManagementShowHideAllText: 'すべて表示/非表示',
  columnsManagementReset: 'リセット',

  // Filter panel text
  filterPanelAddFilter: 'フィルタを追加',
  filterPanelRemoveAll: 'すべて削除',
  filterPanelDeleteIconLabel: '削除',
  filterPanelLogicOperator: '論理演算子',
  filterPanelOperator: '演算子',
  filterPanelOperatorAnd: 'And',
  filterPanelOperatorOr: 'Or',
  filterPanelColumns: '列',
  filterPanelInputLabel: '値',
  filterPanelInputPlaceholder: 'フィルタ値',

  // Filter operators text
  filterOperatorContains: 'を含む',
  filterOperatorDoesNotContain: 'を含まない',
  filterOperatorEquals: 'と等しい',
  filterOperatorDoesNotEqual: 'と等しくない',
  filterOperatorStartsWith: 'で始まる',
  filterOperatorEndsWith: 'で終わる',
  filterOperatorIs: 'は',
  filterOperatorNot: 'ではない',
  filterOperatorAfter: 'より後',
  filterOperatorOnOrAfter: '以降',
  filterOperatorBefore: 'より前',
  filterOperatorOnOrBefore: '以前',
  filterOperatorIsEmpty: '空',
  filterOperatorIsNotEmpty: '空でない',
  filterOperatorIsAnyOf: 'いずれか',

  // Filter values text
  filterValueAny: 'すべて',
  filterValueTrue: 'はい',
  filterValueFalse: 'いいえ',

  // Column menu text
  columnMenuLabel: 'メニュー',
  columnMenuShowColumns: '列の表示設定',
  columnMenuManageColumns: '列の管理',
  columnMenuFilter: 'フィルタ',
  columnMenuHideColumn: 'この列を非表示',
  columnMenuUnsort: 'ソート解除',
  columnMenuSortAsc: '昇順でソート',
  columnMenuSortDesc: '降順でソート',

  // Column header text
  columnHeaderFiltersTooltipActive: (count) =>
    count > 1 ? `${count}件のフィルタが有効` : `${count}件のフィルタが有効`,
  columnHeaderFiltersLabel: 'フィルタを表示',
  columnHeaderSortIconLabel: 'ソート',

  // Rows selected footer text
  footerRowSelected: (count) =>
    count > 1 ? `${count.toLocaleString()}行を選択中` : `${count.toLocaleString()}行を選択中`,

  // Total row amount footer text
  footerTotalRows: '合計行数:',

  // Total visible row amount footer text
  footerTotalVisibleRows: (visibleCount, totalCount) =>
    `${visibleCount.toLocaleString()} / ${totalCount.toLocaleString()}`,

  // Checkbox selection text
  checkboxSelectionHeaderName: '選択',
  checkboxSelectionSelectAllRows: 'すべての行を選択',
  checkboxSelectionUnselectAllRows: 'すべての行の選択を解除',
  checkboxSelectionSelectRow: 'この行を選択',
  checkboxSelectionUnselectRow: 'この行の選択を解除',

  // Boolean cell text
  booleanCellTrueLabel: 'はい',
  booleanCellFalseLabel: 'いいえ',

  // Actions cell more text
  actionsCellMore: 'その他',

  // Column pinning text
  pinToLeft: '左に固定',
  pinToRight: '右に固定',
  unpin: '固定解除',

  // Tree Data
  treeDataGroupingHeaderName: 'グループ',
  treeDataExpand: '子を展開',
  treeDataCollapse: '子を折りたたむ',

  // Grouping columns
  groupingColumnHeaderName: 'グループ',
  groupColumn: (name) => `${name}でグループ化`,
  unGroupColumn: (name) => `${name}のグループ化を解除`,

  // Master/detail
  detailPanelToggle: '詳細パネル切替',
  expandDetailPanel: '展開',
  collapseDetailPanel: '折りたたむ',

  // Pagination
  MuiTablePagination: {
    labelRowsPerPage: '表示件数:',
    labelDisplayedRows: ({ from, to, count }) =>
      `${from}–${to} / ${count !== -1 ? count : `${to}以上`}`,
  },
};
