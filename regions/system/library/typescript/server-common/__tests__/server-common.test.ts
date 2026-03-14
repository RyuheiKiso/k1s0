import { describe, it, expect } from 'vitest';
import {
  ErrorCode,
  ErrorResponse,
  ServiceError,
  auth,
  config,
  dlq,
  tenant,
  session,
  apiRegistry,
  eventStore,
  file,
  scheduler,
  notification,
  order,
  featureflag,
  type ApiResponse,
  type PaginatedResponse,
  type ErrorDetail,
} from '../src/index.js';

describe('ErrorCode', () => {
  // not_found エラーコードが正しい文字列を生成することを確認する
  it('notFound で SYS_{SERVICE}_NOT_FOUND を生成する', () => {
    const code = ErrorCode.notFound('CONFIG');
    expect(code.value).toBe('SYS_CONFIG_NOT_FOUND');
  });

  // validation エラーコードが正しい文字列を生成することを確認する
  it('validation で SYS_{SERVICE}_VALIDATION_FAILED を生成する', () => {
    const code = ErrorCode.validation('DLQ');
    expect(code.value).toBe('SYS_DLQ_VALIDATION_FAILED');
  });

  // internal エラーコードが正しい文字列を生成することを確認する
  it('internal で SYS_{SERVICE}_INTERNAL_ERROR を生成する', () => {
    const code = ErrorCode.internal('AUTH');
    expect(code.value).toBe('SYS_AUTH_INTERNAL_ERROR');
  });

  // unauthorized エラーコードが正しい文字列を生成することを確認する
  it('unauthorized で SYS_{SERVICE}_UNAUTHORIZED を生成する', () => {
    const code = ErrorCode.unauthorized('AUTH');
    expect(code.value).toBe('SYS_AUTH_UNAUTHORIZED');
  });

  // forbidden エラーコードが正しい文字列を生成することを確認する
  it('forbidden で SYS_{SERVICE}_PERMISSION_DENIED を生成する', () => {
    const code = ErrorCode.forbidden('AUTH');
    expect(code.value).toBe('SYS_AUTH_PERMISSION_DENIED');
  });

  // conflict エラーコードが正しい文字列を生成することを確認する
  it('conflict で SYS_{SERVICE}_CONFLICT を生成する', () => {
    const code = ErrorCode.conflict('TENANT');
    expect(code.value).toBe('SYS_TENANT_CONFLICT');
  });

  // unprocessable エラーコードが正しい文字列を生成することを確認する
  it('unprocessable で SYS_{SERVICE}_BUSINESS_RULE_VIOLATION を生成する', () => {
    const code = ErrorCode.unprocessable('ACCT');
    expect(code.value).toBe('SYS_ACCT_BUSINESS_RULE_VIOLATION');
  });

  // rateExceeded エラーコードが正しい文字列を生成することを確認する
  it('rateExceeded で SYS_{SERVICE}_RATE_EXCEEDED を生成する', () => {
    const code = ErrorCode.rateExceeded('RATE');
    expect(code.value).toBe('SYS_RATE_RATE_EXCEEDED');
  });

  // serviceUnavailable エラーコードが正しい文字列を生成することを確認する
  it('serviceUnavailable で SYS_{SERVICE}_SERVICE_UNAVAILABLE を生成する', () => {
    const code = ErrorCode.serviceUnavailable('AUTH');
    expect(code.value).toBe('SYS_AUTH_SERVICE_UNAVAILABLE');
  });

  // BIZ_ プレフィックスのエラーコードが正しい文字列を生成することを確認する
  it('bizNotFound で BIZ_{SERVICE}_NOT_FOUND を生成する', () => {
    expect(ErrorCode.bizNotFound('ORDER').value).toBe('BIZ_ORDER_NOT_FOUND');
  });

  it('bizValidation で BIZ_{SERVICE}_VALIDATION_FAILED を生成する', () => {
    expect(ErrorCode.bizValidation('ORDER').value).toBe(
      'BIZ_ORDER_VALIDATION_FAILED',
    );
  });

  // SVC_ プレフィックスのエラーコードが正しい文字列を生成することを確認する
  it('svcNotFound で SVC_{SERVICE}_NOT_FOUND を生成する', () => {
    expect(ErrorCode.svcNotFound('PAYMENT').value).toBe(
      'SVC_PAYMENT_NOT_FOUND',
    );
  });

  it('svcValidation で SVC_{SERVICE}_VALIDATION_FAILED を生成する', () => {
    expect(ErrorCode.svcValidation('PAYMENT').value).toBe(
      'SVC_PAYMENT_VALIDATION_FAILED',
    );
  });

  // toString でエラーコード文字列を返すことを確認する
  it('toString でエラーコード文字列を返す', () => {
    const code = new ErrorCode('SYS_AUTH_MISSING_CLAIMS');
    expect(code.toString()).toBe('SYS_AUTH_MISSING_CLAIMS');
  });

  // 小文字サービス名が大文字に変換されることを確認する
  it('小文字サービス名が大文字に変換される', () => {
    const code = ErrorCode.notFound('config');
    expect(code.value).toBe('SYS_CONFIG_NOT_FOUND');
  });
});

describe('ErrorResponse', () => {
  // create でコード・メッセージ・requestId が正しく設定されることを確認する
  it('create でエラーレスポンスを生成する', () => {
    const resp = ErrorResponse.create(
      'SYS_CONFIG_KEY_NOT_FOUND',
      'config key not found',
    );
    expect(resp.error.code).toBe('SYS_CONFIG_KEY_NOT_FOUND');
    expect(resp.error.message).toBe('config key not found');
    expect(resp.error.requestId).toBeTruthy();
    expect(resp.error.details).toHaveLength(0);
  });

  // withDetails で詳細情報が正しく設定されることを確認する
  it('withDetails で詳細情報付きレスポンスを生成する', () => {
    const details: ErrorDetail[] = [
      { field: 'namespace', reason: 'required', message: 'must not be empty' },
      { field: 'key', reason: 'format', message: 'invalid format' },
    ];
    const resp = ErrorResponse.withDetails(
      'SYS_CONFIG_VALIDATION_FAILED',
      'validation failed',
      details,
    );
    expect(resp.error.details).toHaveLength(2);
    expect(resp.error.details[0].field).toBe('namespace');
    expect(resp.error.details[0].reason).toBe('required');
    expect(resp.error.details[0].message).toBe('must not be empty');
  });

  // withRequestId で requestId が上書きされることを確認する
  it('withRequestId で requestId を上書きする', () => {
    const resp = ErrorResponse.create('SYS_AUTH_UNAUTHORIZED', 'unauthorized');
    const updated = resp.withRequestId('custom-request-id');
    expect(updated.error.requestId).toBe('custom-request-id');
    expect(updated.error.code).toBe('SYS_AUTH_UNAUTHORIZED');
  });

  // requestId が UUID 形式であることを確認する
  it('requestId が UUID 形式である', () => {
    const resp = ErrorResponse.create('TEST', 'test');
    const uuidRegex =
      /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
    expect(resp.error.requestId).toMatch(uuidRegex);
  });
});

describe('ServiceError', () => {
  // notFound ファクトリメソッドが正しく動作することを確認する
  it('notFound で 404 エラーを生成する', () => {
    const err = ServiceError.notFound('CONFIG', 'key not found');
    expect(err.type).toBe('not_found');
    expect(err.code.value).toBe('SYS_CONFIG_NOT_FOUND');
    expect(err.message).toBe('key not found');
    expect(err.statusCode()).toBe(404);
  });

  // badRequest ファクトリメソッドが正しく動作することを確認する
  it('badRequest で 400 エラーを生成する', () => {
    const err = ServiceError.badRequest('CONFIG', 'invalid input');
    expect(err.type).toBe('bad_request');
    expect(err.code.value).toBe('SYS_CONFIG_VALIDATION_FAILED');
    expect(err.statusCode()).toBe(400);
  });

  // badRequestWithDetails で詳細情報付きエラーを生成することを確認する
  it('badRequestWithDetails で詳細情報付き 400 エラーを生成する', () => {
    const details: ErrorDetail[] = [
      { field: 'page', reason: 'range', message: 'must be >= 1' },
    ];
    const err = ServiceError.badRequestWithDetails(
      'CONFIG',
      'validation failed',
      details,
    );
    expect(err.details).toHaveLength(1);
    expect(err.details[0].reason).toBe('range');
  });

  // unauthorized ファクトリメソッドが正しく動作することを確認する
  it('unauthorized で 401 エラーを生成する', () => {
    const err = ServiceError.unauthorized('AUTH', 'not authenticated');
    expect(err.type).toBe('unauthorized');
    expect(err.statusCode()).toBe(401);
  });

  // forbidden ファクトリメソッドが正しく動作することを確認する
  it('forbidden で 403 エラーを生成する', () => {
    const err = ServiceError.forbidden('AUTH', 'access denied');
    expect(err.type).toBe('forbidden');
    expect(err.statusCode()).toBe(403);
  });

  // conflict ファクトリメソッドが正しく動作することを確認する
  it('conflict で 409 エラーを生成する', () => {
    const err = ServiceError.conflict('TENANT', 'name already exists');
    expect(err.type).toBe('conflict');
    expect(err.statusCode()).toBe(409);
  });

  // unprocessableEntity ファクトリメソッドが正しく動作することを確認する
  it('unprocessableEntity で 422 エラーを生成する', () => {
    const err = ServiceError.unprocessableEntity('ACCT', 'ledger is closed');
    expect(err.type).toBe('unprocessable_entity');
    expect(err.code.value).toBe('SYS_ACCT_BUSINESS_RULE_VIOLATION');
    expect(err.statusCode()).toBe(422);
  });

  // tooManyRequests ファクトリメソッドが正しく動作することを確認する
  it('tooManyRequests で 429 エラーを生成する', () => {
    const err = ServiceError.tooManyRequests('RATE', 'rate limit exceeded');
    expect(err.type).toBe('too_many_requests');
    expect(err.code.value).toBe('SYS_RATE_RATE_EXCEEDED');
    expect(err.statusCode()).toBe(429);
  });

  // internal ファクトリメソッドが正しく動作することを確認する
  it('internal で 500 エラーを生成する', () => {
    const err = ServiceError.internal('AUTH', 'unexpected error');
    expect(err.type).toBe('internal');
    expect(err.statusCode()).toBe(500);
  });

  // serviceUnavailable ファクトリメソッドが正しく動作することを確認する
  it('serviceUnavailable で 503 エラーを生成する', () => {
    const err = ServiceError.serviceUnavailable('AUTH', 'service down');
    expect(err.type).toBe('service_unavailable');
    expect(err.code.value).toBe('SYS_AUTH_SERVICE_UNAVAILABLE');
    expect(err.statusCode()).toBe(503);
  });

  // toErrorResponse が ErrorResponse を正しく生成することを確認する
  it('toErrorResponse で ErrorResponse を生成する', () => {
    const err = ServiceError.notFound('CONFIG', 'key not found');
    const resp = err.toErrorResponse();
    expect(resp.error.code).toBe('SYS_CONFIG_NOT_FOUND');
    expect(resp.error.message).toBe('key not found');
    expect(resp.error.requestId).toBeTruthy();
    expect(resp.error.details).toHaveLength(0);
  });

  // 詳細情報付きエラーの toErrorResponse が details を含むことを確認する
  it('toErrorResponse で詳細情報が含まれる', () => {
    const details: ErrorDetail[] = [
      { field: 'page', reason: 'range', message: 'must be >= 1' },
    ];
    const err = ServiceError.badRequestWithDetails(
      'CONFIG',
      'validation failed',
      details,
    );
    const resp = err.toErrorResponse();
    expect(resp.error.details).toHaveLength(1);
    expect(resp.error.details[0].field).toBe('page');
  });

  // Error を継承していることを確認する
  it('Error を継承している', () => {
    const err = ServiceError.notFound('TEST', 'test');
    expect(err).toBeInstanceOf(Error);
    expect(err).toBeInstanceOf(ServiceError);
  });
});

describe('Well-known error codes', () => {
  // Auth サービスの既知エラーコードを確認する
  it('auth の既知エラーコードが正しい', () => {
    expect(auth.missingClaims().value).toBe('SYS_AUTH_MISSING_CLAIMS');
    expect(auth.permissionDenied().value).toBe('SYS_AUTH_PERMISSION_DENIED');
    expect(auth.unauthorized().value).toBe('SYS_AUTH_UNAUTHORIZED');
    expect(auth.tokenExpired().value).toBe('SYS_AUTH_TOKEN_EXPIRED');
    expect(auth.invalidToken().value).toBe('SYS_AUTH_INVALID_TOKEN');
    expect(auth.jwksFetchFailed().value).toBe('SYS_AUTH_JWKS_FETCH_FAILED');
    expect(auth.auditValidation().value).toBe('SYS_AUTH_AUDIT_VALIDATION');
  });

  // Config サービスの既知エラーコードを確認する
  it('config の既知エラーコードが正しい', () => {
    expect(config.keyNotFound().value).toBe('SYS_CONFIG_KEY_NOT_FOUND');
    expect(config.versionConflict().value).toBe('SYS_CONFIG_VERSION_CONFLICT');
    expect(config.validationFailed().value).toBe(
      'SYS_CONFIG_VALIDATION_FAILED',
    );
    expect(config.internalError().value).toBe('SYS_CONFIG_INTERNAL_ERROR');
  });

  // DLQ サービスの既知エラーコードを確認する
  it('dlq の既知エラーコードが正しい', () => {
    expect(dlq.notFound().value).toBe('SYS_DLQ_NOT_FOUND');
    expect(dlq.processFailed().value).toBe('SYS_DLQ_PROCESS_FAILED');
    expect(dlq.conflict().value).toBe('SYS_DLQ_CONFLICT');
  });

  // Tenant サービスの既知エラーコードを確認する
  it('tenant の既知エラーコードが正しい', () => {
    expect(tenant.notFound().value).toBe('SYS_TENANT_NOT_FOUND');
    expect(tenant.nameConflict().value).toBe('SYS_TENANT_NAME_CONFLICT');
    expect(tenant.memberNotFound().value).toBe('SYS_TENANT_MEMBER_NOT_FOUND');
  });

  // Session サービスの既知エラーコードを確認する
  it('session の既知エラーコードが正しい', () => {
    expect(session.notFound().value).toBe('SYS_SESSION_NOT_FOUND');
    expect(session.expired().value).toBe('SYS_SESSION_EXPIRED');
    expect(session.maxDevicesExceeded().value).toBe(
      'SYS_SESSION_MAX_DEVICES_EXCEEDED',
    );
  });

  // API Registry サービスの既知エラーコードを確認する
  it('apiRegistry の既知エラーコードが正しい', () => {
    expect(apiRegistry.notFound().value).toBe('SYS_APIREG_NOT_FOUND');
    expect(apiRegistry.conflict().value).toBe('SYS_APIREG_CONFLICT');
    expect(apiRegistry.validatorError().value).toBe(
      'SYS_APIREG_VALIDATOR_ERROR',
    );
  });

  // Event Store サービスの既知エラーコードを確認する
  it('eventStore の既知エラーコードが正しい', () => {
    expect(eventStore.streamNotFound().value).toBe(
      'SYS_EVSTORE_STREAM_NOT_FOUND',
    );
    expect(eventStore.versionConflict().value).toBe(
      'SYS_EVSTORE_VERSION_CONFLICT',
    );
  });

  // File サービスの既知エラーコードを確認する
  it('file の既知エラーコードが正しい', () => {
    expect(file.notFound().value).toBe('SYS_FILE_NOT_FOUND');
    expect(file.sizeExceeded().value).toBe('SYS_FILE_SIZE_EXCEEDED');
    expect(file.accessDenied().value).toBe('SYS_FILE_ACCESS_DENIED');
  });

  // Scheduler サービスの既知エラーコードを確認する
  it('scheduler の既知エラーコードが正しい', () => {
    expect(scheduler.alreadyExists().value).toBe('SYS_SCHED_ALREADY_EXISTS');
  });

  // Notification サービスの既知エラーコードを確認する
  it('notification の既知エラーコードが正しい', () => {
    expect(notification.notFound().value).toBe('SYS_NOTIFY_NOT_FOUND');
    expect(notification.alreadySent().value).toBe('SYS_NOTIFY_ALREADY_SENT');
    expect(notification.channelNotFound().value).toBe(
      'SYS_NOTIFY_CHANNEL_NOT_FOUND',
    );
  });

  // Order サービスの既知エラーコードを確認する
  it('order の既知エラーコードが正しい', () => {
    expect(order.notFound().value).toBe('SVC_ORDER_NOT_FOUND');
    expect(order.validationFailed().value).toBe('SVC_ORDER_VALIDATION_FAILED');
    expect(order.versionConflict().value).toBe('SVC_ORDER_VERSION_CONFLICT');
  });

  // Feature Flag サービスの既知エラーコードを確認する
  it('featureflag の既知エラーコードが正しい', () => {
    expect(featureflag.notFound().value).toBe('SYS_FF_NOT_FOUND');
    expect(featureflag.alreadyExists().value).toBe('SYS_FF_ALREADY_EXISTS');
    expect(featureflag.evaluateFailed().value).toBe('SYS_FF_EVALUATE_FAILED');
  });
});

describe('ApiResponse', () => {
  // ApiResponse 型が正しく使えることを確認する
  it('data フィールドを持つオブジェクトを作成できる', () => {
    const response: ApiResponse<{ id: string; name: string }> = {
      data: { id: '1', name: 'test' },
    };
    expect(response.data.id).toBe('1');
    expect(response.data.name).toBe('test');
  });
});

describe('PaginatedResponse', () => {
  // PaginatedResponse 型が正しく使えることを確認する
  it('ページネーション情報を持つオブジェクトを作成できる', () => {
    const response: PaginatedResponse<{ id: string }> = {
      data: [{ id: '1' }, { id: '2' }],
      page: 1,
      perPage: 10,
      total: 2,
      totalPages: 1,
    };
    expect(response.data).toHaveLength(2);
    expect(response.page).toBe(1);
    expect(response.perPage).toBe(10);
    expect(response.total).toBe(2);
    expect(response.totalPages).toBe(1);
  });
});

describe('ServiceError.statusCode', () => {
  // 全てのエラータイプのステータスコードマッピングを確認する
  it('全エラータイプのステータスコードが正しい', () => {
    expect(ServiceError.notFound('T', 'm').statusCode()).toBe(404);
    expect(ServiceError.badRequest('T', 'm').statusCode()).toBe(400);
    expect(ServiceError.unauthorized('T', 'm').statusCode()).toBe(401);
    expect(ServiceError.forbidden('T', 'm').statusCode()).toBe(403);
    expect(ServiceError.conflict('T', 'm').statusCode()).toBe(409);
    expect(ServiceError.unprocessableEntity('T', 'm').statusCode()).toBe(422);
    expect(ServiceError.tooManyRequests('T', 'm').statusCode()).toBe(429);
    expect(ServiceError.internal('T', 'm').statusCode()).toBe(500);
    expect(ServiceError.serviceUnavailable('T', 'm').statusCode()).toBe(503);
  });
});
