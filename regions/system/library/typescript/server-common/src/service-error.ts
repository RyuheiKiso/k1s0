import { ErrorCode } from './error-code.js';
import type { ErrorDetail } from './error-detail.js';
import { ErrorResponse } from './error-response.js';

/** サービスエラーの種類を表す型 */
export type ServiceErrorType =
  | 'not_found'
  | 'bad_request'
  | 'unauthorized'
  | 'forbidden'
  | 'conflict'
  | 'unprocessable_entity'
  | 'too_many_requests'
  | 'internal'
  | 'service_unavailable';

/**
 * HTTP ステータスコードにマッピングされるサービスエラークラス。
 * Rust の ServiceError enum に対応する TypeScript 実装。
 */
export class ServiceError extends Error {
  /** エラーの種類 */
  readonly type: ServiceErrorType;
  /** 構造化されたエラーコード */
  readonly code: ErrorCode;
  /** フィールドレベルの詳細情報 */
  readonly details: ErrorDetail[];

  /**
   * ServiceError を生成する。
   */
  constructor(
    type: ServiceErrorType,
    code: ErrorCode,
    message: string,
    details: ErrorDetail[] = [],
  ) {
    super(message);
    this.name = 'ServiceError';
    this.type = type;
    this.code = code;
    this.details = details;
  }

  // --- ファクトリメソッド ---

  /**
   * 404 Not Found エラーを生成する。
   */
  static notFound(service: string, message: string): ServiceError {
    return new ServiceError('not_found', ErrorCode.notFound(service), message);
  }

  /**
   * 400 Bad Request エラーを生成する。
   */
  static badRequest(service: string, message: string): ServiceError {
    return new ServiceError(
      'bad_request',
      ErrorCode.validation(service),
      message,
    );
  }

  /**
   * 400 Bad Request エラーを詳細情報付きで生成する。
   */
  static badRequestWithDetails(
    service: string,
    message: string,
    details: ErrorDetail[],
  ): ServiceError {
    return new ServiceError(
      'bad_request',
      ErrorCode.validation(service),
      message,
      details,
    );
  }

  /**
   * 401 Unauthorized エラーを生成する。
   */
  static unauthorized(service: string, message: string): ServiceError {
    return new ServiceError(
      'unauthorized',
      ErrorCode.unauthorized(service),
      message,
    );
  }

  /**
   * 403 Forbidden エラーを生成する。
   */
  static forbidden(service: string, message: string): ServiceError {
    return new ServiceError(
      'forbidden',
      ErrorCode.forbidden(service),
      message,
    );
  }

  /**
   * 409 Conflict エラーを生成する。
   */
  static conflict(service: string, message: string): ServiceError {
    return new ServiceError(
      'conflict',
      ErrorCode.conflict(service),
      message,
    );
  }

  /**
   * 422 Unprocessable Entity エラーを生成する（ビジネスルール違反）。
   */
  static unprocessableEntity(
    service: string,
    message: string,
  ): ServiceError {
    return new ServiceError(
      'unprocessable_entity',
      ErrorCode.unprocessable(service),
      message,
    );
  }

  /**
   * 429 Too Many Requests エラーを生成する（レートリミット超過）。
   */
  static tooManyRequests(service: string, message: string): ServiceError {
    return new ServiceError(
      'too_many_requests',
      ErrorCode.rateExceeded(service),
      message,
    );
  }

  /**
   * 500 Internal Server Error を生成する。
   */
  static internal(service: string, message: string): ServiceError {
    return new ServiceError(
      'internal',
      ErrorCode.internal(service),
      message,
    );
  }

  /**
   * 503 Service Unavailable エラーを生成する。
   */
  static serviceUnavailable(
    service: string,
    message: string,
  ): ServiceError {
    return new ServiceError(
      'service_unavailable',
      ErrorCode.serviceUnavailable(service),
      message,
    );
  }

  /**
   * ErrorResponse に変換する。
   * 詳細情報がある場合は withDetails、ない場合は create を使用する。
   */
  toErrorResponse(): ErrorResponse {
    if (this.details.length > 0) {
      return ErrorResponse.withDetails(
        this.code.value,
        this.message,
        this.details,
      );
    }
    return ErrorResponse.create(this.code.value, this.message);
  }

  /**
   * エラー種類に対応する HTTP ステータスコードを返す。
   */
  statusCode(): number {
    switch (this.type) {
      case 'not_found':
        return 404;
      case 'bad_request':
        return 400;
      case 'unauthorized':
        return 401;
      case 'forbidden':
        return 403;
      case 'conflict':
        return 409;
      case 'unprocessable_entity':
        return 422;
      case 'too_many_requests':
        return 429;
      case 'internal':
        return 500;
      case 'service_unavailable':
        return 503;
    }
  }
}
