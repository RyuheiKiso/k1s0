export class AppUpdaterError extends Error {
  readonly code: string;

  constructor(message: string, code: string) {
    super(message);
    this.name = 'AppUpdaterError';
    this.code = code;
  }
}

export class ConnectionError extends AppUpdaterError {
  constructor(message: string) {
    super(message, 'CONNECTION_ERROR');
    this.name = 'ConnectionError';
  }
}

export class InvalidConfigError extends AppUpdaterError {
  constructor(message: string) {
    super(message, 'INVALID_CONFIG');
    this.name = 'InvalidConfigError';
  }
}

export class ParseError extends AppUpdaterError {
  constructor(message: string) {
    super(message, 'PARSE_ERROR');
    this.name = 'ParseError';
  }
}

export class UnauthorizedError extends AppUpdaterError {
  constructor(message: string) {
    super(message, 'UNAUTHORIZED');
    this.name = 'UnauthorizedError';
  }
}

export class AppNotFoundError extends AppUpdaterError {
  constructor(message: string) {
    super(message, 'APP_NOT_FOUND');
    this.name = 'AppNotFoundError';
  }
}

export class VersionNotFoundError extends AppUpdaterError {
  constructor(message: string) {
    super(message, 'VERSION_NOT_FOUND');
    this.name = 'VersionNotFoundError';
  }
}

export class ChecksumError extends AppUpdaterError {
  constructor(message: string) {
    super(message, 'CHECKSUM_ERROR');
    this.name = 'ChecksumError';
  }
}
