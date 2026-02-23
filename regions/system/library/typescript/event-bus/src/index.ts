export interface Event {
  id: string;
  eventType: string;
  payload: Record<string, unknown>;
  timestamp: string;
}

export type EventHandler = (event: Event) => Promise<void>;

export class InMemoryEventBus {
  private handlers = new Map<string, EventHandler[]>();

  subscribe(eventType: string, handler: EventHandler): void {
    const existing = this.handlers.get(eventType) ?? [];
    existing.push(handler);
    this.handlers.set(eventType, existing);
  }

  unsubscribe(eventType: string): void {
    this.handlers.delete(eventType);
  }

  async publish(event: Event): Promise<void> {
    const handlers = this.handlers.get(event.eventType) ?? [];
    for (const handler of handlers) {
      await handler(event);
    }
  }
}
