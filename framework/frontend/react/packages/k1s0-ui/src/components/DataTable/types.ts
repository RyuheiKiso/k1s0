/**
 * K1s0 DataTable 型定義
 *
 * MUI DataGrid をベースにした型定義
 */

import type {
  GridColDef,
  GridRowSelectionModel,
  GridFilterModel,
  GridSortModel,
  GridPaginationModel,
  GridRowParams,
  GridCallbackDetails,
  GridDensity,
} from '@mui/x-data-grid';

/**
 * フィルタオプション
 */
export interface FilterOption {
  label: string;
  value: unknown;
}

/**
 * K1s0 カラム定義（MUI GridColDef を拡張）
 */
export interface K1s0Column<T> extends Omit<GridColDef, 'field'> {
  /** フィールド名（T のキーに制限） */
  field: keyof T & string;
  /** フィルタオプション */
  filterOptions?: FilterOption[];
  /** エクスポート対象かどうか */
  exportable?: boolean;
}

/**
 * エクスポートオプション
 */
export interface ExportOptions {
  /** CSV エクスポートを有効化 */
  csv?: boolean;
  /** 印刷を有効化 */
  print?: boolean;
  /** ファイル名（拡張子なし） */
  fileName?: string;
}

/**
 * K1s0 DataTable Props
 */
export interface K1s0DataTableProps<T extends { id: string | number }> {
  /** データ行 */
  rows: T[];
  /** カラム定義 */
  columns: K1s0Column<T>[];

  /** ページネーションを有効化 */
  pagination?: boolean;
  /** 1ページあたりの行数 */
  pageSize?: number;
  /** ページサイズ選択肢 */
  pageSizeOptions?: number[];
  /** ページネーションモード */
  paginationMode?: 'client' | 'server';
  /** ページネーションモデル */
  paginationModel?: GridPaginationModel;
  /** ページネーションモデル変更時コールバック */
  onPaginationModelChange?: (
    model: GridPaginationModel,
    details: GridCallbackDetails
  ) => void;
  /** サーバーサイドページネーション時の総行数 */
  rowCount?: number;

  /** ソートモデル */
  sortModel?: GridSortModel;
  /** ソートモデル変更時コールバック */
  onSortModelChange?: (
    model: GridSortModel,
    details: GridCallbackDetails
  ) => void;
  /** ソートモード */
  sortingMode?: 'client' | 'server';

  /** フィルタモデル */
  filterModel?: GridFilterModel;
  /** フィルタモデル変更時コールバック */
  onFilterModelChange?: (
    model: GridFilterModel,
    details: GridCallbackDetails
  ) => void;
  /** フィルタモード */
  filterMode?: 'client' | 'server';

  /** チェックボックス選択を有効化 */
  checkboxSelection?: boolean;
  /** 行選択モデル */
  rowSelectionModel?: GridRowSelectionModel;
  /** 行選択モデル変更時コールバック */
  onRowSelectionModelChange?: (
    model: GridRowSelectionModel,
    details: GridCallbackDetails
  ) => void;
  /** 選択不可の行判定 */
  isRowSelectable?: (params: GridRowParams<T>) => boolean;

  /** ローディング状態 */
  loading?: boolean;

  /** 行クリック時コールバック */
  onRowClick?: (row: T) => void;
  /** 行ダブルクリック時コールバック */
  onRowDoubleClick?: (row: T) => void;

  /** 密度（行の高さ） */
  density?: GridDensity;
  /** 自動高さ調整 */
  autoHeight?: boolean;
  /** スティッキーヘッダー */
  stickyHeader?: boolean;
  /** テーブルの高さ（autoHeight が false の場合） */
  height?: number | string;

  /** ツールバーを表示 */
  toolbar?: boolean | React.ReactNode;
  /** エクスポートオプション */
  exportOptions?: ExportOptions;

  /** ツリーデータモード */
  treeData?: boolean;
  /** ツリーデータのパス取得関数 */
  getTreeDataPath?: (row: T) => string[];

  /** 空状態のカスタム表示 */
  noRowsOverlay?: React.ReactNode;
  /** ローディング状態のカスタム表示 */
  loadingOverlay?: React.ReactNode;

  /** カスタムスタイル */
  sx?: Record<string, unknown>;
  /** カスタムクラス名 */
  className?: string;

  /** 行に適用するクラス名を返す関数 */
  getRowClassName?: (params: GridRowParams<T>) => string;
  /** セルに適用するクラス名を返す関数 */
  getCellClassName?: (params: { row: T; field: string }) => string;

  /** ローカライズ設定（デフォルト: 日本語） */
  localeText?: Record<string, string>;
  /** 日本語ローカライズを無効化 */
  disableJapaneseLocale?: boolean;
}

/**
 * useK1s0DataTable フックの返り値
 */
export interface UseK1s0DataTableReturn<T> {
  /** 選択された行 ID */
  selectedRowIds: GridRowSelectionModel;
  /** 選択された行データ */
  selectedRows: T[];
  /** 選択をクリア */
  clearSelection: () => void;
  /** 全選択 */
  selectAll: () => void;
  /** ソートモデル */
  sortModel: GridSortModel;
  /** ソートモデル設定 */
  setSortModel: (model: GridSortModel) => void;
  /** フィルタモデル */
  filterModel: GridFilterModel;
  /** フィルタモデル設定 */
  setFilterModel: (model: GridFilterModel) => void;
  /** ページネーションモデル */
  paginationModel: GridPaginationModel;
  /** ページネーションモデル設定 */
  setPaginationModel: (model: GridPaginationModel) => void;
}

/**
 * useServerSidePagination フックのパラメータ
 */
export interface ServerSidePaginationParams {
  page: number;
  pageSize: number;
  sortModel: GridSortModel;
  filterModel: GridFilterModel;
}

/**
 * useServerSidePagination フックの fetch 関数の返り値
 */
export interface ServerSidePaginationResult<T> {
  rows: T[];
  rowCount: number;
}

/**
 * useServerSidePagination フックのオプション
 */
export interface UseServerSidePaginationOptions<T> {
  /** データ取得関数 */
  fetchFn: (
    params: ServerSidePaginationParams
  ) => Promise<ServerSidePaginationResult<T>>;
  /** 初期ページサイズ */
  initialPageSize?: number;
  /** 初期ページ */
  initialPage?: number;
  /** 依存値（変更時に再取得） */
  deps?: unknown[];
}

/**
 * useServerSidePagination フックの返り値
 */
export interface UseServerSidePaginationReturn<T> {
  /** データ行 */
  rows: T[];
  /** 総行数 */
  rowCount: number;
  /** ローディング状態 */
  loading: boolean;
  /** エラー */
  error: Error | null;
  /** ページネーションモデル */
  paginationModel: GridPaginationModel;
  /** ページネーションモデル設定 */
  setPaginationModel: (model: GridPaginationModel) => void;
  /** ソートモデル */
  sortModel: GridSortModel;
  /** ソートモデル設定 */
  setSortModel: (model: GridSortModel) => void;
  /** フィルタモデル */
  filterModel: GridFilterModel;
  /** フィルタモデル設定 */
  setFilterModel: (model: GridFilterModel) => void;
  /** データ再取得 */
  refetch: () => void;
}

/**
 * アクションカラムのアクション定義
 */
export interface ActionColumnAction<T> {
  /** アクション名 */
  name: string;
  /** ラベル（ツールチップ） */
  label?: string;
  /** アイコンコンポーネント */
  icon: React.ReactNode;
  /** クリック時コールバック */
  onClick: (row: T) => void;
  /** 非表示条件 */
  hidden?: (row: T) => boolean;
  /** 無効化条件 */
  disabled?: (row: T) => boolean;
  /** 色 */
  color?: 'inherit' | 'primary' | 'secondary' | 'success' | 'error' | 'info' | 'warning';
}

/**
 * アクションカラムのオプション
 */
export interface ActionsColumnOptions<T> {
  /** 編集アクション */
  onEdit?: (row: T) => void;
  /** 削除アクション */
  onDelete?: (row: T) => void;
  /** カスタムアクション */
  actions?: ActionColumnAction<T>[];
  /** ヘッダー名 */
  headerName?: string;
  /** カラム幅 */
  width?: number;
}

/**
 * ステータスカラムのオプション
 */
export interface StatusColumnOptions<T, V extends string = string> {
  /** フィールド名 */
  field: keyof T & string;
  /** ヘッダー名 */
  headerName?: string;
  /** ステータスごとの色設定 */
  colorMap?: Record<V, 'default' | 'primary' | 'secondary' | 'success' | 'error' | 'info' | 'warning'>;
  /** ステータスごとのラベル設定 */
  labelMap?: Record<V, string>;
  /** カラム幅 */
  width?: number;
}

/**
 * 日付カラムのオプション
 */
export interface DateColumnOptions<T> {
  /** フィールド名 */
  field: keyof T & string;
  /** ヘッダー名 */
  headerName?: string;
  /** 日付フォーマット（dayjs フォーマット） */
  format?: string;
  /** カラム幅 */
  width?: number;
  /** ソート可能 */
  sortable?: boolean;
}

/**
 * 数値カラムのオプション
 */
export interface NumberColumnOptions<T> {
  /** フィールド名 */
  field: keyof T & string;
  /** ヘッダー名 */
  headerName?: string;
  /** 小数点以下桁数 */
  decimalPlaces?: number;
  /** 単位（プレフィックス） */
  prefix?: string;
  /** 単位（サフィックス） */
  suffix?: string;
  /** 3桁区切りを有効化 */
  thousandSeparator?: boolean;
  /** カラム幅 */
  width?: number;
  /** ソート可能 */
  sortable?: boolean;
  /** テキスト配置 */
  align?: 'left' | 'center' | 'right';
}

/**
 * Boolean カラムのオプション
 */
export interface BooleanColumnOptions<T> {
  /** フィールド名 */
  field: keyof T & string;
  /** ヘッダー名 */
  headerName?: string;
  /** true のラベル */
  trueLabel?: string;
  /** false のラベル */
  falseLabel?: string;
  /** アイコン表示 */
  showIcon?: boolean;
  /** カラム幅 */
  width?: number;
}
