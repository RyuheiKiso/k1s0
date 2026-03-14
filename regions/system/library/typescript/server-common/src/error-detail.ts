/**
 * エラーの詳細情報を表すインターフェース。
 * REST-API設計 D-007 仕様に準拠したフィールドレベルのエラー情報を提供する。
 */
export interface ErrorDetail {
  /** エラーが発生したフィールド名 */
  field: string;
  /** エラーの理由（機械可読） */
  reason: string;
  /** 人間向けのエラーメッセージ */
  message: string;
}
