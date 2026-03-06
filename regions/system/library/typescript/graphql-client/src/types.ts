export interface GraphQlQuery {
  query: string;
  variables?: Record<string, unknown>;
  operationName?: string;
}

export interface GraphQlError {
  message: string;
  locations?: { line: number; column: number }[];
  path?: (string | number)[];
}

export interface GraphQlResponse<T = unknown> {
  data?: T;
  errors?: GraphQlError[];
}

export const ClientErrorKind = {
  Request: 'request',
  Deserialization: 'deserialization',
  GraphQl: 'graphql',
  NotFound: 'not_found',
} as const;

export type ClientErrorKind = (typeof ClientErrorKind)[keyof typeof ClientErrorKind];

export class ClientError extends Error {
  readonly kind: ClientErrorKind;

  constructor(kind: ClientErrorKind, message: string) {
    super(message);
    this.name = 'ClientError';
    this.kind = kind;
  }

  static request(message: string): ClientError {
    return new ClientError(ClientErrorKind.Request, message);
  }

  static deserialization(message: string): ClientError {
    return new ClientError(ClientErrorKind.Deserialization, message);
  }

  static graphQl(message: string): ClientError {
    return new ClientError(ClientErrorKind.GraphQl, message);
  }

  static notFound(message: string): ClientError {
    return new ClientError(ClientErrorKind.NotFound, message);
  }
}
