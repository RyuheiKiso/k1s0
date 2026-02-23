export class ValidationError extends Error {
  constructor(
    public readonly field: string,
    message: string,
  ) {
    super(message);
    this.name = 'ValidationError';
  }
}

const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
const UUID_REGEX = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
const TENANT_ID_REGEX = /^[a-z0-9][a-z0-9-]{1,61}[a-z0-9]$/;

export function validateEmail(email: string): void {
  if (!EMAIL_REGEX.test(email)) {
    throw new ValidationError('email', `invalid email: ${email}`);
  }
}

export function validateUUID(id: string): void {
  if (!UUID_REGEX.test(id)) {
    throw new ValidationError('id', `invalid UUID: ${id}`);
  }
}

export function validateURL(url: string): void {
  try {
    new URL(url);
  } catch {
    throw new ValidationError('url', `invalid URL: ${url}`);
  }
}

export function validateTenantId(tenantId: string): void {
  if (!TENANT_ID_REGEX.test(tenantId)) {
    throw new ValidationError('tenantId', `invalid tenant ID: ${tenantId}`);
  }
}
