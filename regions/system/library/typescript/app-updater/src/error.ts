/** アプリアップデーター操作中に発生するエラーの基底クラス */
export class AppUpdaterError extends Error {
  readonly code: string;

  constructor(message: string, code: string) {
    super(message);
    this.name = 'AppUpdaterError';
    this.code = code;
  }
}

/** サーバーへの接続エラー */
export class ConnectionError extends AppUpdaterError {
  constructor(message: string) {
    super(message, 'CONNECTION_ERROR');
    this.name = 'ConnectionError';
  }
}

/** 設定値が不正な場合のエラー */
export class InvalidConfigError extends AppUpdaterError {
  constructor(message: string) {
    super(message, 'INVALID_CONFIG');
    this.name = 'InvalidConfigError';
  }
}

/** レスポンスのパースエラー */
export class ParseError extends AppUpdaterError {
  constructor(message: string) {
    super(message, 'PARSE_ERROR');
    this.name = 'ParseError';
  }
}

/** 認証エラー（401） */
export class UnauthorizedError extends AppUpdaterError {
  constructor(message: string) {
    super(message, 'UNAUTHORIZED');
    this.name = 'UnauthorizedError';
  }
}

/** 指定したアプリが見つからない場合のエラー（404） */
export class AppNotFoundError extends AppUpdaterError {
  constructor(message: string) {
    super(message, 'APP_NOT_FOUND');
    this.name = 'AppNotFoundError';
  }
}

/** 指定したバージョンが見つからない場合のエラー */
export class VersionNotFoundError extends AppUpdaterError {
  constructor(message: string) {
    super(message, 'VERSION_NOT_FOUND');
    this.name = 'VersionNotFoundError';
  }
}

/** チェックサム不一致エラー */
export class ChecksumError extends AppUpdaterError {
  constructor(message: string) {
    super(message, 'CHECKSUM_ERROR');
    this.name = 'ChecksumError';
  }
}
