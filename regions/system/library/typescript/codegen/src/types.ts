/**
 * コード生成の結果を表すインターフェース。
 * 作成されたファイルとスキップされたファイルのパスを保持する。
 */
export interface GenerateResult {
  /** 生成されたファイルのパス一覧 */
  created: string[];
  /** スキップされたファイルのパス一覧（既存ファイルなど） */
  skipped: string[];
}

/**
 * バリデーション結果を表すインターフェース。
 * 有効かどうかとエラーメッセージの一覧を保持する。
 */
export interface ValidationResult {
  /** バリデーションが成功したかどうか */
  valid: boolean;
  /** バリデーションエラーのメッセージ一覧 */
  errors: string[];
}
