/**
 * コード生成時に発生するエラーを表すカスタムエラークラス。
 * エラーの種類を識別するための code フィールドを持つ。
 */
export class CodegenError extends Error {
  /** エラーの種類を識別するコード */
  readonly code: string;

  /**
   * CodegenError を生成する。
   * @param code エラーコード（例: 'INVALID_CONFIG', 'TEMPLATE_NOT_FOUND'）
   * @param message 人間向けのエラーメッセージ
   */
  constructor(code: string, message: string) {
    super(message);
    this.name = 'CodegenError';
    this.code = code;
  }
}
