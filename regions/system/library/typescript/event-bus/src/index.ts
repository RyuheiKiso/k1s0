// ---------- DomainEvent ----------
export interface DomainEvent {
  readonly eventType: string;
  readonly aggregateId: string;
  readonly occurredAt: Date;
}

// ---------- Legacy Event (backward compat) ----------
export interface Event extends DomainEvent {
  id: string;
  payload: Record<string, unknown>;
  timestamp: string;
}

// ---------- EventHandler ----------
export interface EventHandler<T extends DomainEvent = DomainEvent> {
  handle(event: T): Promise<void>;
}

// ---------- EventBusConfig ----------
export interface EventBusConfig {
  bufferSize?: number;
  handlerTimeoutMs?: number;
}

const DEFAULT_CONFIG: Required<EventBusConfig> = {
  bufferSize: 1024,
  handlerTimeoutMs: 5000,
};

// ---------- EventBusError ----------
export type EventBusErrorCode =
  | 'PUBLISH_FAILED'
  | 'HANDLER_FAILED'
  | 'CHANNEL_CLOSED';

export class EventBusError extends Error {
  public readonly code: EventBusErrorCode;

  constructor(message: string, code: EventBusErrorCode) {
    super(message);
    this.name = 'EventBusError';
    this.code = code;
  }
}

// ---------- EventSubscription ----------
export interface EventSubscription {
  readonly eventType: string;
  unsubscribe(): void;
}

// ---------- EventBus ----------
export class EventBus {
  private handlers = new Map<string, EventHandler<DomainEvent>[]>();
  private readonly config: Required<EventBusConfig>;
  private closed = false;

  constructor(config?: EventBusConfig) {
    this.config = { ...DEFAULT_CONFIG, ...config };
  }

  async publish<T extends DomainEvent>(event: T): Promise<void> {
    if (this.closed) {
      throw new EventBusError(
        'Cannot publish to a closed EventBus',
        'CHANNEL_CLOSED',
      );
    }

    const handlers = this.handlers.get(event.eventType) ?? [];

    for (const handler of handlers) {
      try {
        await this.executeWithTimeout(handler, event);
      } catch (err) {
        if (err instanceof EventBusError) {
          throw err;
        }
        throw new EventBusError(
          `Handler failed for event "${event.eventType}": ${err instanceof Error ? err.message : String(err)}`,
          'HANDLER_FAILED',
        );
      }
    }
  }

  subscribe<T extends DomainEvent>(
    eventType: string,
    handler: EventHandler<T>,
  ): EventSubscription {
    if (this.closed) {
      throw new EventBusError(
        'Cannot subscribe to a closed EventBus',
        'CHANNEL_CLOSED',
      );
    }

    const existing = this.handlers.get(eventType) ?? [];
    const wrappedHandler = handler as unknown as EventHandler<DomainEvent>;
    existing.push(wrappedHandler);
    this.handlers.set(eventType, existing);

    return {
      eventType,
      unsubscribe: () => {
        const current = this.handlers.get(eventType);
        if (current) {
          const idx = current.indexOf(wrappedHandler);
          if (idx !== -1) {
            current.splice(idx, 1);
          }
          if (current.length === 0) {
            this.handlers.delete(eventType);
          }
        }
      },
    };
  }

  close(): void {
    this.closed = true;
    this.handlers.clear();
  }

  private async executeWithTimeout<T extends DomainEvent>(
    handler: EventHandler<T>,
    event: T,
  ): Promise<void> {
    const timeoutMs = this.config.handlerTimeoutMs;

    return new Promise<void>((resolve, reject) => {
      const timer = setTimeout(() => {
        reject(
          new EventBusError(
            `Handler timed out after ${timeoutMs}ms for event "${event.eventType}"`,
            'HANDLER_FAILED',
          ),
        );
      }, timeoutMs);

      Promise.resolve(handler.handle(event))
        .then(() => {
          clearTimeout(timer);
          resolve();
        })
        .catch((err: unknown) => {
          clearTimeout(timer);
          reject(err);
        });
    });
  }
}

// ---------- Legacy InMemoryEventBus (backward compat) ----------
export class InMemoryEventBus {
  private bus: EventBus;
  private subscriptions = new Map<string, EventSubscription[]>();

  constructor(config?: EventBusConfig) {
    this.bus = new EventBus(config);
  }

  subscribe(eventType: string, handler: (event: Event) => Promise<void>): void {
    const sub = this.bus.subscribe<Event>(eventType, { handle: handler });
    const existing = this.subscriptions.get(eventType) ?? [];
    existing.push(sub);
    this.subscriptions.set(eventType, existing);
  }

  unsubscribe(eventType: string): void {
    this.subscriptions.get(eventType)?.forEach((sub) => sub.unsubscribe());
    this.subscriptions.delete(eventType);
  }

  async publish(event: Event): Promise<void> {
    await this.bus.publish(event);
  }
}
