export interface AuditEvent {
  id: string;
  tenantId: string;
  actorId: string;
  action: string;
  resourceType: string;
  resourceId: string;
  timestamp: string;
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
