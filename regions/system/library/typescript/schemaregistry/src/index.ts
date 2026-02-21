/** スキーマ形式。 */
export type SchemaType = 'AVRO' | 'JSON' | 'PROTOBUF';

export interface RegisteredSchema {
  id: number;
  subject: string;
  version: number;
  schema: string;
  schemaType: SchemaType | string;
}

export interface SchemaRegistryConfig {
  url: string;
  username?: string;
  password?: string;
}

export function subjectName(topic: string, keyOrValue: 'key' | 'value'): string {
  return `${topic}-${keyOrValue}`;
}

export function validateSchemaRegistryConfig(config: SchemaRegistryConfig): void {
  if (!config.url) {
    throw new Error('schema registry URL must not be empty');
  }
}

export interface SchemaRegistryClient {
  registerSchema(subject: string, schema: string, schemaType: string): Promise<number>;
  getSchemaById(id: number): Promise<RegisteredSchema>;
  getLatestSchema(subject: string): Promise<RegisteredSchema>;
  getSchemaVersion(subject: string, version: number): Promise<RegisteredSchema>;
  listSubjects(): Promise<string[]>;
  checkCompatibility(subject: string, schema: string): Promise<boolean>;
  healthCheck(): Promise<void>;
}

export class NotFoundError extends Error {
  constructor(public readonly resource: string) {
    super(`not found: ${resource}`);
    this.name = 'NotFoundError';
  }
}

export function isNotFound(err: unknown): boolean {
  return err instanceof NotFoundError;
}

export class SchemaRegistryError extends Error {
  constructor(
    public readonly statusCode: number,
    message: string,
  ) {
    super(`schema registry error (status ${statusCode}): ${message}`);
    this.name = 'SchemaRegistryError';
  }
}

export class HttpSchemaRegistryClient implements SchemaRegistryClient {
  private readonly config: SchemaRegistryConfig;

  constructor(config: SchemaRegistryConfig) {
    validateSchemaRegistryConfig(config);
    this.config = config;
  }

  private async doRequest(method: string, path: string, body?: unknown): Promise<Response> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/vnd.schemaregistry.v1+json',
    };

    if (this.config.username) {
      const credentials = btoa(`${this.config.username}:${this.config.password ?? ''}`);
      headers['Authorization'] = `Basic ${credentials}`;
    }

    const options: RequestInit = { method, headers };
    if (body !== undefined) {
      options.body = JSON.stringify(body);
    }

    return fetch(`${this.config.url}${path}`, options);
  }

  private async handleError(resp: Response, resource: string): Promise<never> {
    if (resp.status === 404) {
      throw new NotFoundError(resource);
    }
    const text = await resp.text();
    throw new SchemaRegistryError(resp.status, text);
  }

  async registerSchema(subject: string, schema: string, schemaType: string): Promise<number> {
    const resp = await this.doRequest('POST', `/subjects/${subject}/versions`, {
      schema,
      schemaType,
    });

    if (!resp.ok) {
      return this.handleError(resp, subject);
    }

    const result = (await resp.json()) as { id: number };
    return result.id;
  }

  async getSchemaById(id: number): Promise<RegisteredSchema> {
    const resp = await this.doRequest('GET', `/schemas/ids/${id}`);

    if (!resp.ok) {
      return this.handleError(resp, `schema id=${id}`);
    }

    const result = (await resp.json()) as RegisteredSchema;
    result.id = id;
    return result;
  }

  async getLatestSchema(subject: string): Promise<RegisteredSchema> {
    return this.getSchemaVersionInternal(subject, 'latest');
  }

  async getSchemaVersion(subject: string, version: number): Promise<RegisteredSchema> {
    return this.getSchemaVersionInternal(subject, String(version));
  }

  private async getSchemaVersionInternal(subject: string, version: string): Promise<RegisteredSchema> {
    const resp = await this.doRequest('GET', `/subjects/${subject}/versions/${version}`);

    if (!resp.ok) {
      return this.handleError(resp, `${subject}/versions/${version}`);
    }

    return (await resp.json()) as RegisteredSchema;
  }

  async listSubjects(): Promise<string[]> {
    const resp = await this.doRequest('GET', '/subjects');

    if (!resp.ok) {
      const text = await resp.text();
      throw new SchemaRegistryError(resp.status, text);
    }

    return (await resp.json()) as string[];
  }

  async checkCompatibility(subject: string, schema: string): Promise<boolean> {
    const resp = await this.doRequest(
      'POST',
      `/compatibility/subjects/${subject}/versions/latest`,
      { schema },
    );

    if (!resp.ok) {
      return this.handleError(resp, subject);
    }

    const result = (await resp.json()) as { is_compatible: boolean };
    return result.is_compatible;
  }

  async healthCheck(): Promise<void> {
    const resp = await this.doRequest('GET', '/');

    if (!resp.ok) {
      throw new SchemaRegistryError(resp.status, 'health check failed');
    }
  }
}
