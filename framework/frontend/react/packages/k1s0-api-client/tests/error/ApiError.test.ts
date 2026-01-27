/**
 * ApiError のテスト
 */

import { describe, it, expect, vi } from 'vitest';
import { ApiError } from '../../src/error/ApiError';
import type { ApiErrorKind } from '../../src/error/types';

// Response モック
const createMockResponse = (
  body: unknown,
  options: { status: number; headers?: Record<string, string> }
) => {
  const headers = new Map(Object.entries(options.headers ?? {}));
  return {
    ok: options.status >= 200 && options.status < 300,
    status: options.status,
    headers: {
      get: (name: string) => headers.get(name.toLowerCase()) ?? null,
    },
    json: vi.fn().mockResolvedValue(body),
    text: vi.fn().mockResolvedValue(JSON.stringify(body)),
  } as unknown as Response;
};

describe('ApiError', () => {
  describe('コンストラクタ', () => {
    it('基本的なプロパティが正しく設定されること', () => {
      const error = new ApiError({
        kind: 'validation',
        status: 400,
        message: 'Validation error',
        errorCode: 'VALIDATION_ERROR',
        traceId: 'trace-123',
      });

      expect(error.kind).toBe('validation');
      expect(error.status).toBe(400);
      expect(error.message).toBe('Validation error');
      expect(error.errorCode).toBe('VALIDATION_ERROR');
      expect(error.traceId).toBe('trace-123');
      expect(error.name).toBe('ApiError');
    });

    it('problemDetails からフィールドエラーが抽出されること', () => {
      const error = new ApiError({
        kind: 'validation',
        status: 400,
        message: 'Validation error',
        errorCode: 'VALIDATION_ERROR',
        problemDetails: {
          error_code: 'VALIDATION_ERROR',
          title: 'Validation Error',
          status: 400,
          errors: [
            { field: 'email', message: 'Invalid email format', code: 'INVALID_EMAIL' },
            { field: 'password', message: 'Too short', code: 'TOO_SHORT' },
          ],
        },
      });

      expect(error.fieldErrors).toHaveLength(2);
      expect(error.hasFieldErrors).toBe(true);
      expect(error.getFieldError('email')).toBe('Invalid email format');
      expect(error.getFieldError('password')).toBe('Too short');
      expect(error.getFieldError('nonexistent')).toBeUndefined();
    });
  });

  describe('isRetryable', () => {
    it('リトライ可能なエラー種別で true を返すこと', () => {
      const retryableKinds: ApiErrorKind[] = ['network', 'timeout', 'server'];

      for (const kind of retryableKinds) {
        const error = new ApiError({
          kind,
          status: 500,
          message: 'Error',
          errorCode: 'ERROR',
        });
        expect(error.isRetryable).toBe(true);
      }
    });

    it('リトライ不可能なエラー種別で false を返すこと', () => {
      const nonRetryableKinds: ApiErrorKind[] = [
        'authentication',
        'authorization',
        'validation',
        'not_found',
        'conflict',
        'rate_limit',
        'unknown',
      ];

      for (const kind of nonRetryableKinds) {
        const error = new ApiError({
          kind,
          status: 400,
          message: 'Error',
          errorCode: 'ERROR',
        });
        expect(error.isRetryable).toBe(false);
      }
    });
  });

  describe('requiresAuthentication', () => {
    it('authentication エラーで true を返すこと', () => {
      const error = new ApiError({
        kind: 'authentication',
        status: 401,
        message: 'Unauthorized',
        errorCode: 'UNAUTHORIZED',
      });

      expect(error.requiresAuthentication).toBe(true);
    });

    it('他のエラー種別で false を返すこと', () => {
      const error = new ApiError({
        kind: 'authorization',
        status: 403,
        message: 'Forbidden',
        errorCode: 'FORBIDDEN',
      });

      expect(error.requiresAuthentication).toBe(false);
    });
  });

  describe('userMessage', () => {
    it('problemDetails.detail がある場合はそれを返すこと', () => {
      const error = new ApiError({
        kind: 'validation',
        status: 400,
        message: 'Validation error',
        errorCode: 'VALIDATION_ERROR',
        problemDetails: {
          error_code: 'VALIDATION_ERROR',
          title: 'Validation Error',
          status: 400,
          detail: 'Custom user message',
        },
      });

      expect(error.userMessage).toBe('Custom user message');
    });

    it('problemDetails.detail がない場合はデフォルトメッセージを返すこと', () => {
      const error = new ApiError({
        kind: 'server',
        status: 500,
        message: 'Internal error',
        errorCode: 'INTERNAL_ERROR',
      });

      expect(error.userMessage).toBeDefined();
      expect(typeof error.userMessage).toBe('string');
    });
  });

  describe('fromResponse', () => {
    it('ProblemDetails 形式のレスポンスを正しくパースすること', async () => {
      const problemDetails = {
        error_code: 'VALIDATION_ERROR',
        title: 'Validation Error',
        status: 400,
        detail: 'Input validation failed',
        trace_id: 'response-trace-123',
        errors: [{ field: 'email', message: 'Invalid' }],
      };

      const response = createMockResponse(problemDetails, {
        status: 400,
        headers: { 'content-type': 'application/problem+json' },
      });

      const error = await ApiError.fromResponse(response);

      expect(error.status).toBe(400);
      expect(error.errorCode).toBe('VALIDATION_ERROR');
      expect(error.traceId).toBe('response-trace-123');
      expect(error.fieldErrors).toHaveLength(1);
    });

    it('リクエストの traceId が優先されること', async () => {
      const response = createMockResponse(
        { error_code: 'ERROR', title: 'Error', status: 400 },
        { status: 400, headers: { 'content-type': 'application/json' } }
      );

      const error = await ApiError.fromResponse(response, 'request-trace-123');

      // レスポンスに trace_id がない場合はリクエストの traceId を使用
      expect(error.traceId).toBe('request-trace-123');
    });

    it('JSON パースに失敗してもエラーを返すこと', async () => {
      const response = {
        ok: false,
        status: 500,
        headers: {
          get: () => 'text/plain',
        },
        json: vi.fn().mockRejectedValue(new Error('Parse error')),
      } as unknown as Response;

      const error = await ApiError.fromResponse(response);

      expect(error.status).toBe(500);
      expect(error.errorCode).toBe('HTTP_500');
    });

    it('ステータスコードから正しい kind を判定すること', async () => {
      const testCases: Array<{ status: number; expectedKind: ApiErrorKind }> = [
        { status: 400, expectedKind: 'validation' },
        { status: 401, expectedKind: 'authentication' },
        { status: 403, expectedKind: 'authorization' },
        { status: 404, expectedKind: 'not_found' },
        { status: 409, expectedKind: 'conflict' },
        { status: 429, expectedKind: 'rate_limit' },
        { status: 500, expectedKind: 'server' },
        { status: 502, expectedKind: 'server' },
        { status: 503, expectedKind: 'server' },
      ];

      for (const { status, expectedKind } of testCases) {
        const response = createMockResponse(
          { error_code: 'ERROR', title: 'Error', status },
          { status, headers: { 'content-type': 'application/json' } }
        );

        const error = await ApiError.fromResponse(response);
        expect(error.kind).toBe(expectedKind);
      }
    });
  });

  describe('fromNetworkError', () => {
    it('タイムアウトエラーを正しく処理すること', () => {
      const abortError = new DOMException('Timeout', 'AbortError');

      const error = ApiError.fromNetworkError(abortError, 'trace-123');

      expect(error.kind).toBe('timeout');
      expect(error.errorCode).toBe('TIMEOUT');
      expect(error.traceId).toBe('trace-123');
    });

    it('一般的なネットワークエラーを正しく処理すること', () => {
      const networkError = new Error('Network failure');

      const error = ApiError.fromNetworkError(networkError, 'trace-123');

      expect(error.kind).toBe('network');
      expect(error.errorCode).toBe('NETWORK_ERROR');
    });

    it('Error 以外の値も処理できること', () => {
      const error = ApiError.fromNetworkError('string error');

      expect(error.kind).toBe('network');
      expect(error.errorCode).toBe('NETWORK_ERROR');
    });
  });

  describe('from', () => {
    it('既存の ApiError をそのまま返すこと', () => {
      const original = new ApiError({
        kind: 'validation',
        status: 400,
        message: 'Error',
        errorCode: 'ERROR',
      });

      const result = ApiError.from(original);

      expect(result).toBe(original);
    });

    it('Response オブジェクトを同期的に処理すること', () => {
      const response = {
        ok: false,
        status: 500,
        headers: { get: () => null },
      } as unknown as Response;

      const result = ApiError.from(response);

      expect(result.status).toBe(500);
    });

    it('その他のエラーをネットワークエラーとして処理すること', () => {
      const error = new Error('Unknown error');

      const result = ApiError.from(error);

      expect(result.kind).toBe('network');
    });
  });
});
