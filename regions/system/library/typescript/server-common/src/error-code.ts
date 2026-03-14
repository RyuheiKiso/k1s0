/**
 * 構造化されたエラーコードを表すクラス。
 * エラーコードは `SYS_{SERVICE}_{ERROR}` パターンに従う。
 * BIZ_（ビジネス層）・SVC_（サービス層）プレフィックスもサポートする。
 */
export class ErrorCode {
  /** エラーコード文字列（例: "SYS_AUTH_UNAUTHORIZED"） */
  readonly value: string;

  /**
   * ErrorCode を生成する。
   * @param value エラーコード文字列
   */
  constructor(value: string) {
    this.value = value;
  }

  /**
   * エラーコード文字列を返す。
   */
  toString(): string {
    return this.value;
  }

  // --- システムティアのファクトリメソッド ---

  /**
   * 標準の "not found" エラーコードを生成する。
   */
  static notFound(service: string): ErrorCode {
    return new ErrorCode(`SYS_${service.toUpperCase()}_NOT_FOUND`);
  }

  /**
   * 標準の "validation failed" エラーコードを生成する。
   */
  static validation(service: string): ErrorCode {
    return new ErrorCode(`SYS_${service.toUpperCase()}_VALIDATION_FAILED`);
  }

  /**
   * 標準の "internal error" エラーコードを生成する。
   */
  static internal(service: string): ErrorCode {
    return new ErrorCode(`SYS_${service.toUpperCase()}_INTERNAL_ERROR`);
  }

  /**
   * 標準の "unauthorized" エラーコードを生成する。
   */
  static unauthorized(service: string): ErrorCode {
    return new ErrorCode(`SYS_${service.toUpperCase()}_UNAUTHORIZED`);
  }

  /**
   * 標準の "forbidden" エラーコードを生成する。
   */
  static forbidden(service: string): ErrorCode {
    return new ErrorCode(`SYS_${service.toUpperCase()}_PERMISSION_DENIED`);
  }

  /**
   * 標準の "conflict" エラーコードを生成する。
   */
  static conflict(service: string): ErrorCode {
    return new ErrorCode(`SYS_${service.toUpperCase()}_CONFLICT`);
  }

  /**
   * 標準の "unprocessable entity" エラーコードを生成する。
   */
  static unprocessable(service: string): ErrorCode {
    return new ErrorCode(
      `SYS_${service.toUpperCase()}_BUSINESS_RULE_VIOLATION`,
    );
  }

  /**
   * 標準の "rate exceeded" エラーコードを生成する。
   */
  static rateExceeded(service: string): ErrorCode {
    return new ErrorCode(`SYS_${service.toUpperCase()}_RATE_EXCEEDED`);
  }

  /**
   * 標準の "service unavailable" エラーコードを生成する。
   */
  static serviceUnavailable(service: string): ErrorCode {
    return new ErrorCode(`SYS_${service.toUpperCase()}_SERVICE_UNAVAILABLE`);
  }

  // --- ビジネスティアのファクトリメソッド ---

  /**
   * ビジネスティアの "not found" エラーコードを生成する。
   */
  static bizNotFound(service: string): ErrorCode {
    return new ErrorCode(`BIZ_${service.toUpperCase()}_NOT_FOUND`);
  }

  /**
   * ビジネスティアの "validation failed" エラーコードを生成する。
   */
  static bizValidation(service: string): ErrorCode {
    return new ErrorCode(`BIZ_${service.toUpperCase()}_VALIDATION_FAILED`);
  }

  // --- サービスティアのファクトリメソッド ---

  /**
   * サービスティアの "not found" エラーコードを生成する。
   */
  static svcNotFound(service: string): ErrorCode {
    return new ErrorCode(`SVC_${service.toUpperCase()}_NOT_FOUND`);
  }

  /**
   * サービスティアの "validation failed" エラーコードを生成する。
   */
  static svcValidation(service: string): ErrorCode {
    return new ErrorCode(`SVC_${service.toUpperCase()}_VALIDATION_FAILED`);
  }
}
