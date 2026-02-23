import type { GraphQlQuery, GraphQlResponse } from './types.js';

export interface GraphQlClient {
  execute<T = unknown>(query: GraphQlQuery): Promise<GraphQlResponse<T>>;
  executeMutation<T = unknown>(mutation: GraphQlQuery): Promise<GraphQlResponse<T>>;
}

export class InMemoryGraphQlClient implements GraphQlClient {
  private responses = new Map<string, unknown>();

  setResponse(operationName: string, response: unknown): void {
    this.responses.set(operationName, response);
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
}
