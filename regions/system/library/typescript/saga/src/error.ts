/** Saga クライアントエラー。 */
export class SagaError extends Error {
  constructor(
    message: string,
    public readonly statusCode?: number,
    cause?: Error,
  ) {
    super(message, { cause });
    this.name = 'SagaError';
  }
}
