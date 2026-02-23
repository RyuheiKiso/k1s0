import { createHmac, timingSafeEqual } from 'crypto';

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

export interface WebhookClient {
  send(url: string, payload: WebhookPayload): Promise<number>;
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
