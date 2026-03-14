import { ErrorCode } from './error-code.js';

/**
 * Auth サービスの既知エラーコード。
 */
export const auth = {
  /** クレーム情報の欠落 */
  missingClaims: () => new ErrorCode('SYS_AUTH_MISSING_CLAIMS'),
  /** 権限拒否 */
  permissionDenied: () => new ErrorCode('SYS_AUTH_PERMISSION_DENIED'),
  /** 未認証 */
  unauthorized: () => new ErrorCode('SYS_AUTH_UNAUTHORIZED'),
  /** トークン期限切れ */
  tokenExpired: () => new ErrorCode('SYS_AUTH_TOKEN_EXPIRED'),
  /** 不正なトークン */
  invalidToken: () => new ErrorCode('SYS_AUTH_INVALID_TOKEN'),
  /** JWKS 取得失敗 */
  jwksFetchFailed: () => new ErrorCode('SYS_AUTH_JWKS_FETCH_FAILED'),
  /** 監査バリデーション */
  auditValidation: () => new ErrorCode('SYS_AUTH_AUDIT_VALIDATION'),
} as const;

/**
 * Config サービスの既知エラーコード。
 */
export const config = {
  /** キーが見つからない */
  keyNotFound: () => new ErrorCode('SYS_CONFIG_KEY_NOT_FOUND'),
  /** サービスが見つからない */
  serviceNotFound: () => new ErrorCode('SYS_CONFIG_SERVICE_NOT_FOUND'),
  /** スキーマが見つからない */
  schemaNotFound: () => new ErrorCode('SYS_CONFIG_SCHEMA_NOT_FOUND'),
  /** バージョン競合 */
  versionConflict: () => new ErrorCode('SYS_CONFIG_VERSION_CONFLICT'),
  /** バリデーション失敗 */
  validationFailed: () => new ErrorCode('SYS_CONFIG_VALIDATION_FAILED'),
  /** 内部エラー */
  internalError: () => new ErrorCode('SYS_CONFIG_INTERNAL_ERROR'),
} as const;

/**
 * DLQ Manager サービスの既知エラーコード。
 */
export const dlq = {
  /** 見つからない */
  notFound: () => new ErrorCode('SYS_DLQ_NOT_FOUND'),
  /** バリデーションエラー */
  validationError: () => new ErrorCode('SYS_DLQ_VALIDATION_ERROR'),
  /** 競合 */
  conflict: () => new ErrorCode('SYS_DLQ_CONFLICT'),
  /** 処理失敗 */
  processFailed: () => new ErrorCode('SYS_DLQ_PROCESS_FAILED'),
  /** 内部エラー */
  internalError: () => new ErrorCode('SYS_DLQ_INTERNAL_ERROR'),
} as const;

/**
 * Tenant サービスの既知エラーコード。
 */
export const tenant = {
  /** 見つからない */
  notFound: () => new ErrorCode('SYS_TENANT_NOT_FOUND'),
  /** 名前の競合 */
  nameConflict: () => new ErrorCode('SYS_TENANT_NAME_CONFLICT'),
  /** 不正なステータス */
  invalidStatus: () => new ErrorCode('SYS_TENANT_INVALID_STATUS'),
  /** 不正な入力 */
  invalidInput: () => new ErrorCode('SYS_TENANT_INVALID_INPUT'),
  /** バリデーションエラー */
  validationError: () => new ErrorCode('SYS_TENANT_VALIDATION_ERROR'),
  /** メンバー競合 */
  memberConflict: () => new ErrorCode('SYS_TENANT_MEMBER_CONFLICT'),
  /** メンバーが見つからない */
  memberNotFound: () => new ErrorCode('SYS_TENANT_MEMBER_NOT_FOUND'),
  /** 内部エラー */
  internalError: () => new ErrorCode('SYS_TENANT_INTERNAL_ERROR'),
} as const;

/**
 * Session サービスの既知エラーコード。
 */
export const session = {
  /** 見つからない */
  notFound: () => new ErrorCode('SYS_SESSION_NOT_FOUND'),
  /** 期限切れ */
  expired: () => new ErrorCode('SYS_SESSION_EXPIRED'),
  /** 既に無効化済み */
  alreadyRevoked: () => new ErrorCode('SYS_SESSION_ALREADY_REVOKED'),
  /** バリデーションエラー */
  validationError: () => new ErrorCode('SYS_SESSION_VALIDATION_ERROR'),
  /** デバイス数上限超過 */
  maxDevicesExceeded: () => new ErrorCode('SYS_SESSION_MAX_DEVICES_EXCEEDED'),
  /** 権限なし */
  forbidden: () => new ErrorCode('SYS_SESSION_FORBIDDEN'),
  /** 内部エラー */
  internalError: () => new ErrorCode('SYS_SESSION_INTERNAL_ERROR'),
} as const;

/**
 * API Registry サービスの既知エラーコード。
 */
export const apiRegistry = {
  /** 見つからない */
  notFound: () => new ErrorCode('SYS_APIREG_NOT_FOUND'),
  /** バリデーションエラー */
  badRequest: () => new ErrorCode('SYS_APIREG_VALIDATION_ERROR'),
  /** 競合 */
  conflict: () => new ErrorCode('SYS_APIREG_CONFLICT'),
  /** 未認証 */
  unauthorized: () => new ErrorCode('SYS_APIREG_UNAUTHORIZED'),
  /** スキーマ不正 */
  schemaInvalid: () => new ErrorCode('SYS_APIREG_SCHEMA_INVALID'),
  /** 内部エラー */
  internalError: () => new ErrorCode('SYS_APIREG_INTERNAL_ERROR'),
  /** バリデーターエラー */
  validatorError: () => new ErrorCode('SYS_APIREG_VALIDATOR_ERROR'),
  /** スキーマが見つからない */
  schemaNotFound: () => new ErrorCode('SYS_APIREG_SCHEMA_NOT_FOUND'),
  /** バージョンが見つからない */
  versionNotFound: () => new ErrorCode('SYS_APIREG_VERSION_NOT_FOUND'),
  /** 最新バージョンは削除不可 */
  cannotDeleteLatest: () => new ErrorCode('SYS_APIREG_CANNOT_DELETE_LATEST'),
  /** 既に存在する */
  alreadyExists: () => new ErrorCode('SYS_APIREG_ALREADY_EXISTS'),
} as const;

/**
 * Event Store サービスの既知エラーコード。
 */
export const eventStore = {
  /** ストリームが見つからない */
  streamNotFound: () => new ErrorCode('SYS_EVSTORE_STREAM_NOT_FOUND'),
  /** イベントが見つからない */
  eventNotFound: () => new ErrorCode('SYS_EVSTORE_EVENT_NOT_FOUND'),
  /** スナップショットが見つからない */
  snapshotNotFound: () => new ErrorCode('SYS_EVSTORE_SNAPSHOT_NOT_FOUND'),
  /** バージョン競合 */
  versionConflict: () => new ErrorCode('SYS_EVSTORE_VERSION_CONFLICT'),
  /** ストリームが既に存在する */
  streamAlreadyExists: () =>
    new ErrorCode('SYS_EVSTORE_STREAM_ALREADY_EXISTS'),
} as const;

/**
 * File サービスの既知エラーコード。
 */
export const file = {
  /** バリデーション */
  validation: () => new ErrorCode('SYS_FILE_VALIDATION'),
  /** 見つからない */
  notFound: () => new ErrorCode('SYS_FILE_NOT_FOUND'),
  /** 既に完了 */
  alreadyCompleted: () => new ErrorCode('SYS_FILE_ALREADY_COMPLETED'),
  /** 利用不可 */
  notAvailable: () => new ErrorCode('SYS_FILE_NOT_AVAILABLE'),
  /** アクセス拒否 */
  accessDenied: () => new ErrorCode('SYS_FILE_ACCESS_DENIED'),
  /** ストレージエラー */
  storageError: () => new ErrorCode('SYS_FILE_STORAGE_ERROR'),
  /** サイズ超過 */
  sizeExceeded: () => new ErrorCode('SYS_FILE_SIZE_EXCEEDED'),
  /** アップロード失敗 */
  uploadFailed: () => new ErrorCode('SYS_FILE_UPLOAD_FAILED'),
  /** 取得失敗 */
  getFailed: () => new ErrorCode('SYS_FILE_GET_FAILED'),
  /** 一覧取得失敗 */
  listFailed: () => new ErrorCode('SYS_FILE_LIST_FAILED'),
  /** 削除失敗 */
  deleteFailed: () => new ErrorCode('SYS_FILE_DELETE_FAILED'),
  /** 完了処理失敗 */
  completeFailed: () => new ErrorCode('SYS_FILE_COMPLETE_FAILED'),
  /** ダウンロードURL生成失敗 */
  downloadUrlFailed: () => new ErrorCode('SYS_FILE_DOWNLOAD_URL_FAILED'),
  /** タグ更新失敗 */
  tagsUpdateFailed: () => new ErrorCode('SYS_FILE_TAGS_UPDATE_FAILED'),
} as const;

/**
 * Scheduler サービスの既知エラーコード。
 */
export const scheduler = {
  /** 既に存在する */
  alreadyExists: () => new ErrorCode('SYS_SCHED_ALREADY_EXISTS'),
} as const;

/**
 * Notification サービスの既知エラーコード。
 */
export const notification = {
  /** 不正なID */
  invalidId: () => new ErrorCode('SYS_NOTIFY_INVALID_ID'),
  /** バリデーションエラー */
  validationError: () => new ErrorCode('SYS_NOTIFY_VALIDATION_ERROR'),
  /** 見つからない */
  notFound: () => new ErrorCode('SYS_NOTIFY_NOT_FOUND'),
  /** チャネルが見つからない */
  channelNotFound: () => new ErrorCode('SYS_NOTIFY_CHANNEL_NOT_FOUND'),
  /** テンプレートが見つからない */
  templateNotFound: () => new ErrorCode('SYS_NOTIFY_TEMPLATE_NOT_FOUND'),
  /** 既に送信済み */
  alreadySent: () => new ErrorCode('SYS_NOTIFY_ALREADY_SENT'),
  /** チャネル無効 */
  channelDisabled: () => new ErrorCode('SYS_NOTIFY_CHANNEL_DISABLED'),
  /** 内部エラー */
  internalError: () => new ErrorCode('SYS_NOTIFY_INTERNAL_ERROR'),
  /** 送信失敗 */
  sendFailed: () => new ErrorCode('SYS_NOTIFY_SEND_FAILED'),
  /** 一覧取得失敗 */
  listFailed: () => new ErrorCode('SYS_NOTIFY_LIST_FAILED'),
  /** 取得失敗 */
  getFailed: () => new ErrorCode('SYS_NOTIFY_GET_FAILED'),
  /** リトライ失敗 */
  retryFailed: () => new ErrorCode('SYS_NOTIFY_RETRY_FAILED'),
  /** チャネル作成失敗 */
  channelCreateFailed: () => new ErrorCode('SYS_NOTIFY_CHANNEL_CREATE_FAILED'),
  /** チャネル一覧取得失敗 */
  channelListFailed: () => new ErrorCode('SYS_NOTIFY_CHANNEL_LIST_FAILED'),
  /** チャネル取得失敗 */
  channelGetFailed: () => new ErrorCode('SYS_NOTIFY_CHANNEL_GET_FAILED'),
  /** チャネル更新失敗 */
  channelUpdateFailed: () => new ErrorCode('SYS_NOTIFY_CHANNEL_UPDATE_FAILED'),
  /** チャネル削除失敗 */
  channelDeleteFailed: () => new ErrorCode('SYS_NOTIFY_CHANNEL_DELETE_FAILED'),
  /** テンプレート作成失敗 */
  templateCreateFailed: () =>
    new ErrorCode('SYS_NOTIFY_TEMPLATE_CREATE_FAILED'),
  /** テンプレート一覧取得失敗 */
  templateListFailed: () => new ErrorCode('SYS_NOTIFY_TEMPLATE_LIST_FAILED'),
  /** テンプレート取得失敗 */
  templateGetFailed: () => new ErrorCode('SYS_NOTIFY_TEMPLATE_GET_FAILED'),
  /** テンプレート更新失敗 */
  templateUpdateFailed: () =>
    new ErrorCode('SYS_NOTIFY_TEMPLATE_UPDATE_FAILED'),
  /** テンプレート削除失敗 */
  templateDeleteFailed: () =>
    new ErrorCode('SYS_NOTIFY_TEMPLATE_DELETE_FAILED'),
} as const;

/**
 * Order サービス（サービスティア）の既知エラーコード。
 */
export const order = {
  /** 見つからない */
  notFound: () => new ErrorCode('SVC_ORDER_NOT_FOUND'),
  /** バリデーション失敗 */
  validationFailed: () => new ErrorCode('SVC_ORDER_VALIDATION_FAILED'),
  /** 不正なステータス遷移 */
  invalidStatusTransition: () =>
    new ErrorCode('SVC_ORDER_INVALID_STATUS_TRANSITION'),
  /** バージョン競合 */
  versionConflict: () => new ErrorCode('SVC_ORDER_VERSION_CONFLICT'),
  /** 内部エラー */
  internalError: () => new ErrorCode('SVC_ORDER_INTERNAL_ERROR'),
} as const;

/**
 * Feature Flag サービスの既知エラーコード。
 */
export const featureflag = {
  /** 内部エラー */
  internalError: () => new ErrorCode('SYS_FF_INTERNAL_ERROR'),
  /** 見つからない */
  notFound: () => new ErrorCode('SYS_FF_NOT_FOUND'),
  /** 既に存在する */
  alreadyExists: () => new ErrorCode('SYS_FF_ALREADY_EXISTS'),
  /** 一覧取得失敗 */
  listFailed: () => new ErrorCode('SYS_FF_LIST_FAILED'),
  /** 取得失敗 */
  getFailed: () => new ErrorCode('SYS_FF_GET_FAILED'),
  /** 作成失敗 */
  createFailed: () => new ErrorCode('SYS_FF_CREATE_FAILED'),
  /** 更新失敗 */
  updateFailed: () => new ErrorCode('SYS_FF_UPDATE_FAILED'),
  /** 削除失敗 */
  deleteFailed: () => new ErrorCode('SYS_FF_DELETE_FAILED'),
  /** 評価失敗 */
  evaluateFailed: () => new ErrorCode('SYS_FF_EVALUATE_FAILED'),
} as const;
