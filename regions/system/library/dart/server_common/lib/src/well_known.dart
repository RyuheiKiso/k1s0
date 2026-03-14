import 'error_code.dart';

/// Auth サービスの既知エラーコード。
class AuthErrors {
  AuthErrors._();

  /// クレーム欠落エラー
  static ErrorCode missingClaims() =>
      const ErrorCode('SYS_AUTH_MISSING_CLAIMS');

  /// 権限拒否エラー
  static ErrorCode permissionDenied() =>
      const ErrorCode('SYS_AUTH_PERMISSION_DENIED');

  /// 未認証エラー
  static ErrorCode unauthorized() =>
      const ErrorCode('SYS_AUTH_UNAUTHORIZED');

  /// トークン期限切れエラー
  static ErrorCode tokenExpired() =>
      const ErrorCode('SYS_AUTH_TOKEN_EXPIRED');

  /// 無効なトークンエラー
  static ErrorCode invalidToken() =>
      const ErrorCode('SYS_AUTH_INVALID_TOKEN');

  /// JWKS 取得失敗エラー
  static ErrorCode jwksFetchFailed() =>
      const ErrorCode('SYS_AUTH_JWKS_FETCH_FAILED');

  /// 監査バリデーションエラー
  static ErrorCode auditValidation() =>
      const ErrorCode('SYS_AUTH_AUDIT_VALIDATION');
}

/// Config サービスの既知エラーコード。
class ConfigErrors {
  ConfigErrors._();

  /// キーが見つからないエラー
  static ErrorCode keyNotFound() =>
      const ErrorCode('SYS_CONFIG_KEY_NOT_FOUND');

  /// サービスが見つからないエラー
  static ErrorCode serviceNotFound() =>
      const ErrorCode('SYS_CONFIG_SERVICE_NOT_FOUND');

  /// スキーマが見つからないエラー
  static ErrorCode schemaNotFound() =>
      const ErrorCode('SYS_CONFIG_SCHEMA_NOT_FOUND');

  /// バージョン競合エラー
  static ErrorCode versionConflict() =>
      const ErrorCode('SYS_CONFIG_VERSION_CONFLICT');

  /// バリデーション失敗エラー
  static ErrorCode validationFailed() =>
      const ErrorCode('SYS_CONFIG_VALIDATION_FAILED');

  /// 内部エラー
  static ErrorCode internalError() =>
      const ErrorCode('SYS_CONFIG_INTERNAL_ERROR');
}

/// DLQ Manager サービスの既知エラーコード。
class DlqErrors {
  DlqErrors._();

  /// メッセージが見つからないエラー
  static ErrorCode notFound() =>
      const ErrorCode('SYS_DLQ_NOT_FOUND');

  /// バリデーションエラー
  static ErrorCode validationError() =>
      const ErrorCode('SYS_DLQ_VALIDATION_ERROR');

  /// 競合エラー
  static ErrorCode conflict() =>
      const ErrorCode('SYS_DLQ_CONFLICT');

  /// 処理失敗エラー
  static ErrorCode processFailed() =>
      const ErrorCode('SYS_DLQ_PROCESS_FAILED');

  /// 内部エラー
  static ErrorCode internalError() =>
      const ErrorCode('SYS_DLQ_INTERNAL_ERROR');
}

/// Tenant サービスの既知エラーコード。
class TenantErrors {
  TenantErrors._();

  /// テナントが見つからないエラー
  static ErrorCode notFound() =>
      const ErrorCode('SYS_TENANT_NOT_FOUND');

  /// テナント名競合エラー
  static ErrorCode nameConflict() =>
      const ErrorCode('SYS_TENANT_NAME_CONFLICT');

  /// 無効なステータスエラー
  static ErrorCode invalidStatus() =>
      const ErrorCode('SYS_TENANT_INVALID_STATUS');

  /// 無効な入力エラー
  static ErrorCode invalidInput() =>
      const ErrorCode('SYS_TENANT_INVALID_INPUT');

  /// バリデーションエラー
  static ErrorCode validationError() =>
      const ErrorCode('SYS_TENANT_VALIDATION_ERROR');

  /// メンバー競合エラー
  static ErrorCode memberConflict() =>
      const ErrorCode('SYS_TENANT_MEMBER_CONFLICT');

  /// メンバーが見つからないエラー
  static ErrorCode memberNotFound() =>
      const ErrorCode('SYS_TENANT_MEMBER_NOT_FOUND');

  /// 内部エラー
  static ErrorCode internalError() =>
      const ErrorCode('SYS_TENANT_INTERNAL_ERROR');
}

/// Session サービスの既知エラーコード。
class SessionErrors {
  SessionErrors._();

  /// セッションが見つからないエラー
  static ErrorCode notFound() =>
      const ErrorCode('SYS_SESSION_NOT_FOUND');

  /// セッション期限切れエラー
  static ErrorCode expired() =>
      const ErrorCode('SYS_SESSION_EXPIRED');

  /// 既に失効済みエラー
  static ErrorCode alreadyRevoked() =>
      const ErrorCode('SYS_SESSION_ALREADY_REVOKED');

  /// バリデーションエラー
  static ErrorCode validationError() =>
      const ErrorCode('SYS_SESSION_VALIDATION_ERROR');

  /// 最大デバイス数超過エラー
  static ErrorCode maxDevicesExceeded() =>
      const ErrorCode('SYS_SESSION_MAX_DEVICES_EXCEEDED');

  /// アクセス禁止エラー
  static ErrorCode forbidden() =>
      const ErrorCode('SYS_SESSION_FORBIDDEN');

  /// 内部エラー
  static ErrorCode internalError() =>
      const ErrorCode('SYS_SESSION_INTERNAL_ERROR');
}

/// Order サービスの既知エラーコード（サービス層）。
class OrderErrors {
  OrderErrors._();

  /// 注文が見つからないエラー
  static ErrorCode notFound() =>
      const ErrorCode('SVC_ORDER_NOT_FOUND');

  /// バリデーション失敗エラー
  static ErrorCode validationFailed() =>
      const ErrorCode('SVC_ORDER_VALIDATION_FAILED');

  /// 無効なステータス遷移エラー
  static ErrorCode invalidStatusTransition() =>
      const ErrorCode('SVC_ORDER_INVALID_STATUS_TRANSITION');

  /// バージョン競合エラー
  static ErrorCode versionConflict() =>
      const ErrorCode('SVC_ORDER_VERSION_CONFLICT');

  /// 内部エラー
  static ErrorCode internalError() =>
      const ErrorCode('SVC_ORDER_INTERNAL_ERROR');
}
