import type { ErrorDetail } from './error-detail.js';

/**
 * エラーレスポンスの本体を表すインターフェース。
 * コード・メッセージ・リクエストID・詳細情報を含む。
 */
export interface ErrorBody {
  /** 機械可読のエラーコード */
  code: string;
  /** 人間向けのエラーメッセージ */
  message: string;
  /** トレーシング用のリクエストID */
  requestId: string;
  /** フィールドレベルの詳細情報 */
  details: ErrorDetail[];
}

/**
 * エラーレスポンスを `{ "error": ... }` エンベロープで包むクラス。
 * Rust の ErrorResponse に対応する TypeScript 実装。
 */
export class ErrorResponse {
  /** エラーレスポンスの本体 */
  readonly error: ErrorBody;

  /**
   * ErrorResponse を生成する。
   * requestId は crypto.randomUUID() で自動生成される。
   */
  constructor(error: ErrorBody) {
    this.error = error;
  }

  /**
   * コードとメッセージからエラーレスポンスを生成する。
   * details は空配列、requestId は自動生成される。
   */
  static create(code: string, message: string): ErrorResponse {
    return new ErrorResponse({
      code,
      message,
      requestId: crypto.randomUUID(),
      details: [],
    });
  }

  /**
   * コード・メッセージ・詳細情報からエラーレスポンスを生成する。
   * requestId は自動生成される。
   */
  static withDetails(
    code: string,
    message: string,
    details: ErrorDetail[],
  ): ErrorResponse {
    return new ErrorResponse({
      code,
      message,
      requestId: crypto.randomUUID(),
      details,
    });
  }

  /**
   * requestId を上書きした新しい ErrorResponse を返す。
   * 相関IDが利用可能な場合に使用する。
   */
  withRequestId(id: string): ErrorResponse {
    return new ErrorResponse({
      ...this.error,
      requestId: id,
    });
  }
}
