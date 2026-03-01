import { createHmac, timingSafeEqual, randomUUID } from 'crypto';

export interface WebhookPayload {
  eventType: string;
  timestamp: string;
  data: Record<string, unknown>;
}

export function generateSignature(secret: string, body: string): string {
  return createHmac('sha256', secret).update(body).digest('hex');
}

export function verifySignature(secret: string, body: string, signature: string): boolean {
  const expected = generateSignature(secret, body);
  const a = Buffer.from(expected, 'hex');
  const b = Buffer.from(signature, 'hex');
  if (a.length !== b.length) {
    return false;
  }
  return timingSafeEqual(a, b);
}

export interface WebhookConfig {
  maxRetries?: number;
  initialBackoffMs?: number;
  maxBackoffMs?: number;
}

export type WebhookErrorCode = 'SEND_FAILED' | 'MAX_RETRIES_EXCEEDED';

export class WebhookError extends Error {
  readonly code: WebhookErrorCode;

  constructor(message: string, code: WebhookErrorCode) {
    super(message);
    this.name = 'WebhookError';
    this.code = code;
  }
}

export interface WebhookClient {
  send(url: string, payload: WebhookPayload): Promise<number>;
}

export class HttpWebhookClient implements WebhookClient {
  private readonly secret?: string;
  private readonly maxRetries: number;
  private readonly initialBackoffMs: number;
  private readonly maxBackoffMs: number;
  private readonly fetchFn: typeof fetch;

  constructor(
    config: WebhookConfig & { secret?: string } = {},
    fetchFn?: typeof fetch,
  ) {
    this.secret = config.secret;
    this.maxRetries = config.maxRetries ?? 3;
    this.initialBackoffMs = config.initialBackoffMs ?? 1000;
    this.maxBackoffMs = config.maxBackoffMs ?? 30000;
    this.fetchFn = fetchFn ?? globalThis.fetch;
  }

  async send(url: string, payload: WebhookPayload): Promise<number> {
    const body = JSON.stringify(payload);
    const idempotencyKey = randomUUID();

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      'Idempotency-Key': idempotencyKey,
    };

    if (this.secret) {
      headers['X-K1s0-Signature'] = generateSignature(this.secret, body);
    }

    let lastStatus = 0;
    let lastError: Error | undefined;

    for (let attempt = 0; attempt <= this.maxRetries; attempt++) {
      if (attempt > 0) {
        const backoff = Math.min(
          this.initialBackoffMs * Math.pow(2, attempt - 1),
          this.maxBackoffMs,
        );
        const jitter = Math.random() * backoff;
        const delay = backoff + jitter;
        console.log(
          `[webhook-client] Retry attempt ${attempt}/${this.maxRetries} for ${url} after ${Math.round(delay)}ms`,
        );
        await new Promise((resolve) => setTimeout(resolve, delay));
      }

      try {
        console.log(
          `[webhook-client] Sending webhook to ${url} (attempt ${attempt + 1}/${this.maxRetries + 1}, idempotency-key=${idempotencyKey})`,
        );

        const response = await this.fetchFn(url, {
          method: 'POST',
          headers,
          body,
        });

        lastStatus = response.status;

        if (this.isRetryable(lastStatus)) {
          console.log(
            `[webhook-client] Retryable status ${lastStatus} from ${url}`,
          );
          lastError = new WebhookError(
            `Webhook request to ${url} returned status ${lastStatus}`,
            'SEND_FAILED',
          );
          continue;
        }

        return lastStatus;
      } catch (err) {
        lastError =
          err instanceof Error
            ? err
            : new Error(String(err));
        console.log(
          `[webhook-client] Network error on attempt ${attempt + 1}/${this.maxRetries + 1} for ${url}: ${lastError.message}`,
        );
      }
    }

    throw new WebhookError(
      `Webhook delivery to ${url} failed after ${this.maxRetries + 1} attempts: ${lastError?.message ?? `status ${lastStatus}`}`,
      'MAX_RETRIES_EXCEEDED',
    );
  }

  private isRetryable(status: number): boolean {
    return status === 429 || status >= 500;
  }
}

export class InMemoryWebhookClient implements WebhookClient {
  private sent: Array<{ url: string; payload: WebhookPayload }> = [];

  async send(url: string, payload: WebhookPayload): Promise<number> {
    this.sent.push({ url, payload });
    return 200;
  }

  getSent(): Array<{ url: string; payload: WebhookPayload }> {
    return [...this.sent];
  }
}
