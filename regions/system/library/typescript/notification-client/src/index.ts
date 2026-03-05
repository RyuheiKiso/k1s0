import { randomBytes } from 'crypto';

export type NotificationChannel = 'email' | 'sms' | 'push' | 'slack' | 'webhook';

export interface NotificationRequest {
  id: string;
  channel: NotificationChannel;
  recipient: string;
  subject?: string;
  body: string;
  metadata?: Record<string, unknown>;
}

export interface NotificationResponse {
  id: string;
  status: string;
  messageId?: string;
}

export type SendNotificationInput = NotificationRequest;
export type SendNotificationOutput = NotificationResponse;

export interface NotificationClient {
  send(request: NotificationRequest): Promise<NotificationResponse>;
  sendBatch(inputs: SendNotificationInput[]): Promise<SendNotificationOutput[]>;
}

export class InMemoryNotificationClient implements NotificationClient {
  private sent: NotificationRequest[] = [];

  async send(request: NotificationRequest): Promise<NotificationResponse> {
    this.sent.push(request);
    return {
      id: request.id,
      status: 'sent',
      messageId: randomBytes(8).toString('hex'),
    };
  }

  async sendBatch(
    inputs: SendNotificationInput[],
  ): Promise<SendNotificationOutput[]> {
    const outputs: SendNotificationOutput[] = [];
    for (const input of inputs) {
      this.sent.push(input);
      outputs.push({
        id: input.id,
        status: 'sent',
        messageId: randomBytes(8).toString('hex'),
      });
    }
    return outputs;
  }

  getSent(): NotificationRequest[] {
    return [...this.sent];
  }
}
