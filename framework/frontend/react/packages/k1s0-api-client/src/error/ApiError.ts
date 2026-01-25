import {
  type ProblemDetails,
  ProblemDetailsSchema,
  type ApiErrorKind,
  mapStatusToErrorKind,
  getDefaultErrorMessage,
  isRetryableError,
} from './types.js';

/**
 * APIエラーを表す統一クラス
 * - ProblemDetails形式のエラーを保持
 * - エラー分類（kind）による判定を提供
 * - リトライ可否、ユーザー向けメッセージを統一
 */
export class ApiError extends Error {
  readonly kind: ApiErrorKind;
  readonly status: number;
  readonly errorCode: string;
  readonly traceId: string | undefined;
  readonly problemDetails: ProblemDetails | undefined;
  readonly fieldErrors: ReadonlyArray<{
    field: string;
    message: string;
    code?: string;
  }>;

  constructor(options: {
    kind: ApiErrorKind;
    status: number;
    message: string;
    errorCode: string;
    traceId?: string;
    problemDetails?: ProblemDetails;
    cause?: unknown;
  }) {
    super(options.message, { cause: options.cause });
    this.name = 'ApiError';
    this.kind = options.kind;
    this.status = options.status;
    this.errorCode = options.errorCode;
    this.traceId = options.traceId;
    this.problemDetails = options.problemDetails;
    this.fieldErrors = options.problemDetails?.errors ?? [];
  }

  /**
   * リトライ可能なエラーかどうか
   */
  get isRetryable(): boolean {
    return isRetryableError(this.kind);
  }

  /**
   * 認証が必要なエラーかどうか
   */
  get requiresAuthentication(): boolean {
    return this.kind === 'authentication';
  }

  /**
   * ユーザーに表示するメッセージ
   * problemDetails.detailがあればそれを優先
   */
  get userMessage(): string {
    return this.problemDetails?.detail ?? getDefaultErrorMessage(this.kind);
  }

  /**
   * フィールドエラーがあるかどうか
   */
  get hasFieldErrors(): boolean {
    return this.fieldErrors.length > 0;
  }

  /**
   * 指定フィールドのエラーメッセージを取得
   */
  getFieldError(field: string): string | undefined {
    return this.fieldErrors.find((e) => e.field === field)?.message;
  }

  /**
   * HTTPレスポンスからApiErrorを生成
   */
  static async fromResponse(
    response: Response,
    requestTraceId?: string
  ): Promise<ApiError> {
    const status = response.status;
    const kind = mapStatusToErrorKind(status);

    let problemDetails: ProblemDetails | undefined;
    let errorCode = `HTTP_${status}`;
    let traceId = requestTraceId;
    let message = getDefaultErrorMessage(kind);

    try {
      const contentType = response.headers.get('content-type');
      if (contentType?.includes('application/json') || contentType?.includes('application/problem+json')) {
        const body = await response.json();
        const parsed = ProblemDetailsSchema.safeParse(body);
        if (parsed.success) {
          problemDetails = parsed.data;
          errorCode = problemDetails.error_code;
          traceId = problemDetails.trace_id ?? traceId;
          message = problemDetails.detail ?? problemDetails.title;
        }
      }
    } catch {
      // JSONパース失敗時はデフォルト値を使用
    }

    return new ApiError({
      kind,
      status,
      message,
      errorCode,
      traceId,
      problemDetails,
    });
  }

  /**
   * ネットワークエラー（fetch失敗など）からApiErrorを生成
   */
  static fromNetworkError(error: unknown, requestTraceId?: string): ApiError {
    const cause = error instanceof Error ? error : new Error(String(error));

    // タイムアウトエラーの判定
    if (cause.name === 'AbortError') {
      return new ApiError({
        kind: 'timeout',
        status: 0,
        message: 'リクエストがタイムアウトしました',
        errorCode: 'TIMEOUT',
        traceId: requestTraceId,
        cause,
      });
    }

    return new ApiError({
      kind: 'network',
      status: 0,
      message: 'ネットワークエラーが発生しました',
      errorCode: 'NETWORK_ERROR',
      traceId: requestTraceId,
      cause,
    });
  }

  /**
   * 任意のエラーをApiErrorに変換（既にApiErrorならそのまま返す）
   */
  static from(error: unknown, requestTraceId?: string): ApiError {
    if (error instanceof ApiError) {
      return error;
    }

    if (error instanceof Response) {
      // Responseオブジェクトが直接渡された場合は同期的に処理
      return new ApiError({
        kind: mapStatusToErrorKind(error.status),
        status: error.status,
        message: getDefaultErrorMessage(mapStatusToErrorKind(error.status)),
        errorCode: `HTTP_${error.status}`,
        traceId: requestTraceId,
      });
    }

    return ApiError.fromNetworkError(error, requestTraceId);
  }
}
