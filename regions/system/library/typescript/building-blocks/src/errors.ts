export class ComponentError extends Error {
  constructor(
    public readonly component: string,
    public readonly operation: string,
    message: string,
    public readonly cause?: Error,
  ) {
    super(`[${component}] ${operation}: ${message}`);
    this.name = 'ComponentError';
  }
}

export class ETagMismatchError extends Error {
  constructor(
    public readonly key: string,
    public readonly expected: string,
    public readonly actual: string,
  ) {
    super(`ETag mismatch for key "${key}": expected "${expected}", got "${actual}"`);
    this.name = 'ETagMismatchError';
  }
}
