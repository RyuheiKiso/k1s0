export class ValidationError extends Error {
  public readonly code: string;

  constructor(
    public readonly field: string,
    message: string,
    code?: string,
  ) {
    super(message);
    this.name = 'ValidationError';
    this.code = code ?? `INVALID_${field.toUpperCase().replace(/[^A-Z0-9]/g, '_')}`;
  }
}

export class ValidationErrors {
  private readonly errors: ValidationError[] = [];

  hasErrors(): boolean {
    return this.errors.length > 0;
  }

  getErrors(): ReadonlyArray<ValidationError> {
    return this.errors;
  }

  add(error: ValidationError): void {
    this.errors.push(error);
  }
}

// 4言語統一バリデーション正規表現パターン（H-18）
// メールアドレス: TLD 2文字以上を必須とする
const EMAIL_REGEX = /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/;
// UUID: v4 のみ許可する
const UUID_REGEX = /^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-4[0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$/;
// テナントID: 先頭・末尾は英数字、中間はハイフン許可、3-63文字
const TENANT_ID_REGEX = /^[a-z0-9][a-z0-9-]{1,61}[a-z0-9]$/;

export function validateEmail(email: string): void {
  if (!EMAIL_REGEX.test(email)) {
    throw new ValidationError('email', `invalid email: ${email}`, 'INVALID_EMAIL');
  }
}

export function validateUUID(id: string): void {
  if (!UUID_REGEX.test(id)) {
    throw new ValidationError('id', `invalid UUID: ${id}`, 'INVALID_UUID');
  }
}

// URLバリデーション: スキームをhttp/httpsのみに制限する。
// javascript:, data:, file: 等の危険なスキームを許可すると、
// XSSやローカルファイル読み取りなどの攻撃に悪用される可能性がある。
export function validateURL(url: string): void {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    throw new ValidationError('url', `invalid URL: ${url}`, 'INVALID_URL');
  }

  // 許可するスキームをhttp/httpsのみに制限する
  const allowedProtocols = ['http:', 'https:'];
  if (!allowedProtocols.includes(parsed.protocol)) {
    throw new ValidationError(
      'url',
      `unsupported URL scheme: ${parsed.protocol} (only http and https are allowed)`,
      'INVALID_URL_SCHEME',
    );
  }
}

/**
 * DNS Rebinding 攻撃防御のため、プライベートIPアドレスへのURL参照をブロックする
 * M-010 監査対応: RFC 1918 / loopback / link-local / ULA への直接アクセスを拒否する
 * 注意: DNS 解決は行わずホスト名の IP リテラルのみチェックする（解決後 IP はサーバー側で検証）
 */
export function validateURLNotPrivate(url: string): void {
  // まず既存の URL 妥当性チェック（スキームなど）を実行する
  validateURL(url);
  const { hostname } = new URL(url);
  const privatePatterns = [
    // RFC 1918: プライベートIPv4アドレス空間
    /^10\./,
    /^172\.(1[6-9]|2\d|3[01])\./,
    /^192\.168\./,
    // loopback: 自ループバックアドレス
    /^127\./,
    // IPv6 loopback: Node.js WHATWG URL API は URL.hostname に [::1]（ブラケット付き）を返す
    // CRIT-002 対応: ::1 と [::1] の両形式に対応し、SSRF 保護の穴を塞ぐ
    /^::1$|^\[::1\]$/,
    // IPv4-mapped IPv6 ループバックの 16 進数短縮形（例: [::ffff:7f00:1] は 127.0.0.1 と等価）
    // Node.js URL API は ::ffff:127.0.0.1 を ::ffff:7f00:1 に正規化し [::ffff:7f00:1] を返す
    // 0x7f = 127 であるため、[::ffff:7fXX:] パターンをすべて拒否する
    /^\[::ffff:7f[0-9a-f]{2}:/i,
    // link-local: リンクローカルアドレス（169.254.0.0/16）
    /^169\.254\./,
    // IPv6 ULA（Unique Local Address, fc00::/7）
    /^fc00:/i,
    // IPv6 link-local（fe80::/10）
    /^fe80:/i,
  ];
  if (privatePatterns.some(p => p.test(hostname))) {
    throw new ValidationError(
      'url',
      'プライベートIPアドレスへのアクセスは禁止されています',
      'PRIVATE_IP_FORBIDDEN',
    );
  }
}

export function validateTenantId(tenantId: string): void {
  if (!TENANT_ID_REGEX.test(tenantId)) {
    throw new ValidationError('tenantId', `invalid tenant ID: ${tenantId}`, 'INVALID_TENANT_ID');
  }
}

export function validatePagination(page: number, perPage: number): void {
  if (!Number.isInteger(page) || page < 1) {
    throw new ValidationError('page', `page must be >= 1, got ${page}`, 'INVALID_PAGE');
  }
  if (!Number.isInteger(perPage) || perPage < 1 || perPage > 100) {
    throw new ValidationError('perPage', `perPage must be 1-100, got ${perPage}`, 'INVALID_PER_PAGE');
  }
}

export function validateDateRange(startDate: Date, endDate: Date): void {
  if (startDate > endDate) {
    throw new ValidationError(
      'dateRange',
      `start date (${startDate.toISOString()}) must be <= end date (${endDate.toISOString()})`,
      'INVALID_DATE_RANGE',
    );
  }
}
