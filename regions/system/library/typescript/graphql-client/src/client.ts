import type { GraphQlQuery, GraphQlResponse } from './types.js';

export interface GraphQlClient {
  execute<T = unknown>(query: GraphQlQuery): Promise<GraphQlResponse<T>>;
  executeMutation<T = unknown>(mutation: GraphQlQuery): Promise<GraphQlResponse<T>>;
  subscribe<T = unknown>(subscription: GraphQlQuery): AsyncIterable<GraphQlResponse<T>>;
}

export class InMemoryGraphQlClient implements GraphQlClient {
  private responses = new Map<string, unknown>();
  private subscriptionEvents = new Map<string, unknown[]>();

  setResponse(operationName: string, response: unknown): void {
    this.responses.set(operationName, response);
  }

  setSubscriptionEvents(operationName: string, events: unknown[]): void {
    this.subscriptionEvents.set(operationName, events);
  }

  async execute<T = unknown>(query: GraphQlQuery): Promise<GraphQlResponse<T>> {
    const key = query.operationName ?? '';
    const response = this.responses.get(key);
    if (response === undefined) {
      return {
        errors: [{ message: `No response configured for operation: ${key}` }],
      };
    }
    return { data: response as T };
  }

  async executeMutation<T = unknown>(mutation: GraphQlQuery): Promise<GraphQlResponse<T>> {
    return this.execute<T>(mutation);
  }

  async *subscribe<T = unknown>(subscription: GraphQlQuery): AsyncIterable<GraphQlResponse<T>> {
    const key = subscription.operationName ?? subscription.query;
    const events = this.subscriptionEvents.get(key) ?? [];
    for (const event of events) {
      yield { data: event as T };
    }
  }
}
