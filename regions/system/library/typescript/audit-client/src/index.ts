export interface AuditEvent {
  id: string;
  tenantId: string;
  actorId: string;
  action: string;
  resourceType: string;
  resourceId: string;
  metadata?: Record<string, unknown>;
  timestamp: string;
}

export type AuditErrorKind = 'SerializationError' | 'SendError' | 'Internal';

export class AuditError extends Error {
  readonly kind: AuditErrorKind;
  constructor(kind: AuditErrorKind, message: string) {
    super(message);
    this.name = 'AuditError';
    this.kind = kind;
  }
}

export interface AuditClient {
  record(event: AuditEvent): Promise<void>;
  flush(): Promise<AuditEvent[]>;
}

export class BufferedAuditClient implements AuditClient {
  private buffer: AuditEvent[] = [];

  async record(event: AuditEvent): Promise<void> {
    this.buffer.push(event);
  }

  async flush(): Promise<AuditEvent[]> {
    const events = [...this.buffer];
    this.buffer = [];
    return events;
  }
}
