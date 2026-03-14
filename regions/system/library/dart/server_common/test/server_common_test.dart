import 'package:k1s0_server_common/server_common.dart';
import 'package:test/test.dart';

void main() {
  group('ErrorCode', () {
    // notFound ファクトリが正しいコード文字列を生成することを確認する。
    test('notFoundが正しいコードを生成すること', () {
      final code = ErrorCode.notFound('CONFIG');
      expect(code.value, 'SYS_CONFIG_NOT_FOUND');
    });

    // validation ファクトリが正しいコード文字列を生成することを確認する。
    test('validationが正しいコードを生成すること', () {
      final code = ErrorCode.validation('DLQ');
      expect(code.value, 'SYS_DLQ_VALIDATION_FAILED');
    });

    // internal ファクトリが正しいコード文字列を生成することを確認する。
    test('internalが正しいコードを生成すること', () {
      final code = ErrorCode.internal('AUTH');
      expect(code.value, 'SYS_AUTH_INTERNAL_ERROR');
    });

    // unauthorized ファクトリが正しいコード文字列を生成することを確認する。
    test('unauthorizedが正しいコードを生成すること', () {
      final code = ErrorCode.unauthorized('AUTH');
      expect(code.value, 'SYS_AUTH_UNAUTHORIZED');
    });

    // forbidden ファクトリが正しいコード文字列を生成することを確認する。
    test('forbiddenが正しいコードを生成すること', () {
      final code = ErrorCode.forbidden('AUTH');
      expect(code.value, 'SYS_AUTH_PERMISSION_DENIED');
    });

    // conflict ファクトリが正しいコード文字列を生成することを確認する。
    test('conflictが正しいコードを生成すること', () {
      final code = ErrorCode.conflict('TENANT');
      expect(code.value, 'SYS_TENANT_CONFLICT');
    });

    // unprocessable ファクトリが正しいコード文字列を生成することを確認する。
    test('unprocessableが正しいコードを生成すること', () {
      final code = ErrorCode.unprocessable('ACCT');
      expect(code.value, 'SYS_ACCT_BUSINESS_RULE_VIOLATION');
    });

    // rateExceeded ファクトリが正しいコード文字列を生成することを確認する。
    test('rateExceededが正しいコードを生成すること', () {
      final code = ErrorCode.rateExceeded('RATE');
      expect(code.value, 'SYS_RATE_RATE_EXCEEDED');
    });

    // serviceUnavailable ファクトリが正しいコード文字列を生成することを確認する。
    test('serviceUnavailableが正しいコードを生成すること', () {
      final code = ErrorCode.serviceUnavailable('AUTH');
      expect(code.value, 'SYS_AUTH_SERVICE_UNAVAILABLE');
    });

    // 小文字のサービス名が大文字に変換されることを確認する。
    test('小文字のサービス名が大文字に変換されること', () {
      final code = ErrorCode.notFound('config');
      expect(code.value, 'SYS_CONFIG_NOT_FOUND');
    });
  });

  group('ErrorCode BIZ/SVC', () {
    // BIZ プレフィックスのファクトリが正しいコードを生成することを確認する。
    test('bizNotFoundが正しいコードを生成すること', () {
      final code = ErrorCode.bizNotFound('ORDER');
      expect(code.value, 'BIZ_ORDER_NOT_FOUND');
    });

    // BIZ バリデーションファクトリが正しいコードを生成することを確認する。
    test('bizValidationが正しいコードを生成すること', () {
      final code = ErrorCode.bizValidation('ORDER');
      expect(code.value, 'BIZ_ORDER_VALIDATION_FAILED');
    });

    // SVC プレフィックスのファクトリが正しいコードを生成することを確認する。
    test('svcNotFoundが正しいコードを生成すること', () {
      final code = ErrorCode.svcNotFound('PAYMENT');
      expect(code.value, 'SVC_PAYMENT_NOT_FOUND');
    });

    // SVC バリデーションファクトリが正しいコードを生成することを確認する。
    test('svcValidationが正しいコードを生成すること', () {
      final code = ErrorCode.svcValidation('PAYMENT');
      expect(code.value, 'SVC_PAYMENT_VALIDATION_FAILED');
    });
  });

  group('ErrorResponse', () {
    // ErrorResponse のコンストラクタが正しいフィールドを設定することを確認する。
    test('コードとメッセージが正しく設定されること', () {
      final resp = ErrorResponse(
        const ErrorCode('SYS_CONFIG_KEY_NOT_FOUND'),
        'config key not found',
      );
      expect(resp.error.code.value, 'SYS_CONFIG_KEY_NOT_FOUND');
      expect(resp.error.message, 'config key not found');
      expect(resp.error.requestId, isNotEmpty);
      expect(resp.error.details, isEmpty);
    });

    // JSON シリアライズでエンベロープ構造が正しいことを確認する。
    test('JSONシリアライズが正しいこと', () {
      final resp = ErrorResponse(
        const ErrorCode('SYS_CONFIG_KEY_NOT_FOUND'),
        'not found',
      );
      final json = resp.toJson();
      expect(json['error']['code'], 'SYS_CONFIG_KEY_NOT_FOUND');
      expect(json['error']['message'], 'not found');
      expect(json['error']['request_id'], isA<String>());
      // details が空の場合は含まれないことを確認する
      expect(json['error'].containsKey('details'), isFalse);
    });

    // withRequestId でリクエスト ID が上書きされることを確認する。
    test('withRequestIdでリクエストIDが上書きされること', () {
      final resp = ErrorResponse(
        const ErrorCode('SYS_CONFIG_KEY_NOT_FOUND'),
        'not found',
      ).withRequestId('custom-request-id');
      expect(resp.error.requestId, 'custom-request-id');
    });
  });

  group('ErrorResponse.withDetails', () {
    // 詳細情報付きの ErrorResponse が正しく生成されることを確認する。
    test('詳細情報が正しく設定されること', () {
      final details = [
        const ErrorDetail(
          field: 'namespace',
          reason: 'required',
          message: 'must not be empty',
        ),
        const ErrorDetail(
          field: 'key',
          reason: 'format',
          message: 'invalid format',
        ),
      ];
      final resp = ErrorResponse.withDetails(
        const ErrorCode('SYS_CONFIG_VALIDATION_FAILED'),
        'validation failed',
        details,
      );
      expect(resp.error.details, hasLength(2));
      expect(resp.error.details[0].field, 'namespace');
      expect(resp.error.details[0].reason, 'required');
      expect(resp.error.details[0].message, 'must not be empty');
    });

    // 詳細情報付き JSON シリアライズが正しいことを確認する。
    test('詳細情報付きJSONシリアライズが正しいこと', () {
      final details = [
        const ErrorDetail(
          field: 'field1',
          reason: 'invalid',
          message: 'error1',
        ),
      ];
      final resp = ErrorResponse.withDetails(
        const ErrorCode('SYS_CONFIG_VALIDATION_FAILED'),
        'validation',
        details,
      );
      final json = resp.toJson();
      expect(json['error']['details'][0]['field'], 'field1');
      expect(json['error']['details'][0]['reason'], 'invalid');
      expect(json['error']['details'][0]['message'], 'error1');
    });
  });

  group('ServiceError', () {
    // notFound ファクトリが正しい ServiceError を生成することを確認する。
    test('notFoundが正しいエラーを生成すること', () {
      final err = ServiceError.notFound('CONFIG', 'key not found');
      expect(err.type, ServiceErrorType.notFound);
      expect(err.code.value, 'SYS_CONFIG_NOT_FOUND');
      expect(err.message, 'key not found');
    });

    // badRequest ファクトリが正しい ServiceError を生成することを確認する。
    test('badRequestが正しいエラーを生成すること', () {
      final err = ServiceError.badRequest('CONFIG', 'validation failed');
      expect(err.type, ServiceErrorType.badRequest);
      expect(err.code.value, 'SYS_CONFIG_VALIDATION_FAILED');
    });

    // unauthorized ファクトリが正しい ServiceError を生成することを確認する。
    test('unauthorizedが正しいエラーを生成すること', () {
      final err = ServiceError.unauthorized('AUTH', 'invalid token');
      expect(err.type, ServiceErrorType.unauthorized);
      expect(err.code.value, 'SYS_AUTH_UNAUTHORIZED');
    });

    // forbidden ファクトリが正しい ServiceError を生成することを確認する。
    test('forbiddenが正しいエラーを生成すること', () {
      final err = ServiceError.forbidden('AUTH', 'access denied');
      expect(err.type, ServiceErrorType.forbidden);
      expect(err.code.value, 'SYS_AUTH_PERMISSION_DENIED');
    });

    // conflict ファクトリが正しい ServiceError を生成することを確認する。
    test('conflictが正しいエラーを生成すること', () {
      final err = ServiceError.conflict('TENANT', 'name exists');
      expect(err.type, ServiceErrorType.conflict);
      expect(err.code.value, 'SYS_TENANT_CONFLICT');
    });

    // unprocessableEntity ファクトリが正しい ServiceError を生成することを確認する。
    test('unprocessableEntityが正しいエラーを生成すること', () {
      final err = ServiceError.unprocessableEntity('ACCT', 'ledger is closed');
      expect(err.type, ServiceErrorType.unprocessableEntity);
      expect(err.code.value, 'SYS_ACCT_BUSINESS_RULE_VIOLATION');
    });

    // tooManyRequests ファクトリが正しい ServiceError を生成することを確認する。
    test('tooManyRequestsが正しいエラーを生成すること', () {
      final err = ServiceError.tooManyRequests('RATE', 'rate limit exceeded');
      expect(err.type, ServiceErrorType.tooManyRequests);
      expect(err.code.value, 'SYS_RATE_RATE_EXCEEDED');
    });

    // internal ファクトリが正しい ServiceError を生成することを確認する。
    test('internalが正しいエラーを生成すること', () {
      final err = ServiceError.internal('AUTH', 'unexpected error');
      expect(err.type, ServiceErrorType.internal);
      expect(err.code.value, 'SYS_AUTH_INTERNAL_ERROR');
    });

    // serviceUnavailable ファクトリが正しい ServiceError を生成することを確認する。
    test('serviceUnavailableが正しいエラーを生成すること', () {
      final err = ServiceError.serviceUnavailable('AUTH', 'service down');
      expect(err.type, ServiceErrorType.serviceUnavailable);
      expect(err.code.value, 'SYS_AUTH_SERVICE_UNAVAILABLE');
    });

    // toErrorResponse が正しい ErrorResponse を生成することを確認する。
    test('toErrorResponseが正しいレスポンスを生成すること', () {
      final err = ServiceError.notFound('CONFIG', 'key not found');
      final resp = err.toErrorResponse();
      expect(resp.error.code.value, 'SYS_CONFIG_NOT_FOUND');
      expect(resp.error.message, 'key not found');
      expect(resp.error.requestId, isNotEmpty);
    });

    // ServiceError が Exception を実装していることを確認する。
    test('ServiceErrorがExceptionを実装していること', () {
      final err = ServiceError.notFound('CONFIG', 'key not found');
      expect(err, isA<Exception>());
    });
  });

  group('Well-known error codes', () {
    // Auth サービスの既知エラーコードが正しいことを確認する。
    test('AuthErrorsが正しいコードを返すこと', () {
      expect(AuthErrors.missingClaims().value, 'SYS_AUTH_MISSING_CLAIMS');
      expect(
        AuthErrors.permissionDenied().value,
        'SYS_AUTH_PERMISSION_DENIED',
      );
      expect(AuthErrors.unauthorized().value, 'SYS_AUTH_UNAUTHORIZED');
      expect(AuthErrors.tokenExpired().value, 'SYS_AUTH_TOKEN_EXPIRED');
      expect(AuthErrors.invalidToken().value, 'SYS_AUTH_INVALID_TOKEN');
      expect(AuthErrors.jwksFetchFailed().value, 'SYS_AUTH_JWKS_FETCH_FAILED');
      expect(
        AuthErrors.auditValidation().value,
        'SYS_AUTH_AUDIT_VALIDATION',
      );
    });

    // Config サービスの既知エラーコードが正しいことを確認する。
    test('ConfigErrorsが正しいコードを返すこと', () {
      expect(ConfigErrors.keyNotFound().value, 'SYS_CONFIG_KEY_NOT_FOUND');
      expect(
        ConfigErrors.serviceNotFound().value,
        'SYS_CONFIG_SERVICE_NOT_FOUND',
      );
      expect(
        ConfigErrors.schemaNotFound().value,
        'SYS_CONFIG_SCHEMA_NOT_FOUND',
      );
      expect(
        ConfigErrors.versionConflict().value,
        'SYS_CONFIG_VERSION_CONFLICT',
      );
      expect(
        ConfigErrors.validationFailed().value,
        'SYS_CONFIG_VALIDATION_FAILED',
      );
      expect(
        ConfigErrors.internalError().value,
        'SYS_CONFIG_INTERNAL_ERROR',
      );
    });

    // DLQ サービスの既知エラーコードが正しいことを確認する。
    test('DlqErrorsが正しいコードを返すこと', () {
      expect(DlqErrors.notFound().value, 'SYS_DLQ_NOT_FOUND');
      expect(DlqErrors.validationError().value, 'SYS_DLQ_VALIDATION_ERROR');
      expect(DlqErrors.conflict().value, 'SYS_DLQ_CONFLICT');
      expect(DlqErrors.processFailed().value, 'SYS_DLQ_PROCESS_FAILED');
      expect(DlqErrors.internalError().value, 'SYS_DLQ_INTERNAL_ERROR');
    });

    // Tenant サービスの既知エラーコードが正しいことを確認する。
    test('TenantErrorsが正しいコードを返すこと', () {
      expect(TenantErrors.notFound().value, 'SYS_TENANT_NOT_FOUND');
      expect(TenantErrors.nameConflict().value, 'SYS_TENANT_NAME_CONFLICT');
      expect(TenantErrors.invalidStatus().value, 'SYS_TENANT_INVALID_STATUS');
      expect(TenantErrors.invalidInput().value, 'SYS_TENANT_INVALID_INPUT');
      expect(
        TenantErrors.validationError().value,
        'SYS_TENANT_VALIDATION_ERROR',
      );
      expect(
        TenantErrors.memberConflict().value,
        'SYS_TENANT_MEMBER_CONFLICT',
      );
      expect(
        TenantErrors.memberNotFound().value,
        'SYS_TENANT_MEMBER_NOT_FOUND',
      );
      expect(
        TenantErrors.internalError().value,
        'SYS_TENANT_INTERNAL_ERROR',
      );
    });

    // Session サービスの既知エラーコードが正しいことを確認する。
    test('SessionErrorsが正しいコードを返すこと', () {
      expect(SessionErrors.notFound().value, 'SYS_SESSION_NOT_FOUND');
      expect(SessionErrors.expired().value, 'SYS_SESSION_EXPIRED');
      expect(
        SessionErrors.alreadyRevoked().value,
        'SYS_SESSION_ALREADY_REVOKED',
      );
      expect(
        SessionErrors.validationError().value,
        'SYS_SESSION_VALIDATION_ERROR',
      );
      expect(
        SessionErrors.maxDevicesExceeded().value,
        'SYS_SESSION_MAX_DEVICES_EXCEEDED',
      );
      expect(SessionErrors.forbidden().value, 'SYS_SESSION_FORBIDDEN');
      expect(
        SessionErrors.internalError().value,
        'SYS_SESSION_INTERNAL_ERROR',
      );
    });

    // Order サービスの既知エラーコードが正しいことを確認する。
    test('OrderErrorsが正しいコードを返すこと', () {
      expect(OrderErrors.notFound().value, 'SVC_ORDER_NOT_FOUND');
      expect(
        OrderErrors.validationFailed().value,
        'SVC_ORDER_VALIDATION_FAILED',
      );
      expect(
        OrderErrors.invalidStatusTransition().value,
        'SVC_ORDER_INVALID_STATUS_TRANSITION',
      );
      expect(
        OrderErrors.versionConflict().value,
        'SVC_ORDER_VERSION_CONFLICT',
      );
      expect(OrderErrors.internalError().value, 'SVC_ORDER_INTERNAL_ERROR');
    });
  });

  group('ApiResponse', () {
    // ApiResponse が data フィールドを正しく保持することを確認する。
    test('dataフィールドが正しく設定されること', () {
      const resp = ApiResponse(data: 'hello');
      expect(resp.data, 'hello');
    });

    // toJson が正しい JSON 構造を生成することを確認する。
    test('toJsonが正しい構造を返すこと', () {
      const resp = ApiResponse(data: 42);
      final json = resp.toJson((v) => v);
      expect(json['data'], 42);
    });
  });

  group('PaginatedResponse', () {
    // PaginatedResponse がページネーション情報を正しく保持することを確認する。
    test('ページネーション情報が正しく設定されること', () {
      const resp = PaginatedResponse(
        data: [1, 2, 3],
        page: 1,
        perPage: 10,
        total: 25,
        totalPages: 3,
      );
      expect(resp.data, hasLength(3));
      expect(resp.page, 1);
      expect(resp.perPage, 10);
      expect(resp.total, 25);
      expect(resp.totalPages, 3);
    });

    // toJson が正しい JSON 構造を生成することを確認する。
    test('toJsonが正しい構造を返すこと', () {
      const resp = PaginatedResponse(
        data: ['a', 'b'],
        page: 2,
        perPage: 5,
        total: 12,
        totalPages: 3,
      );
      final json = resp.toJson((v) => v);
      expect(json['data'], ['a', 'b']);
      expect(json['page'], 2);
      expect(json['per_page'], 5);
      expect(json['total'], 12);
      expect(json['total_pages'], 3);
    });
  });
}
