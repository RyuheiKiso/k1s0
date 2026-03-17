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

export function validateURL(url: string): void {
  try {
    new URL(url);
  } catch {
    throw new ValidationError('url', `invalid URL: ${url}`, 'INVALID_URL');
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
