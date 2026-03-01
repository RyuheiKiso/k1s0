import { randomBytes } from 'crypto';

export type NotificationChannel = 'email' | 'sms' | 'push' | 'slack' | 'webhook';

export interface NotificationRequest {
  id: string;
  channel: NotificationChannel;
  recipient: string;
  subject?: string;
  body: string;
}

export interface NotificationResponse {
  id: string;
  status: string;
  messageId?: string;
}

export interface NotificationClient {
  send(request: NotificationRequest): Promise<NotificationResponse>;
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

  getSent(): NotificationRequest[] {
    return [...this.sent];
  }
}
