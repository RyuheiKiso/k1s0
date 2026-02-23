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
