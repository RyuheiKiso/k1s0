/** DLQ クライアントエラー。 */
export class DlqError extends Error {
  constructor(
    message: string,
    public readonly statusCode?: number,
    cause?: Error,
  ) {
    super(message, { cause });
    this.name = 'DlqError';
  }
}
