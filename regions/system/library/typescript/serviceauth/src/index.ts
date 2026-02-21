/** サービストークンのクレーム。 */
export interface ServiceClaims {
  sub: string;
  iss: string;
  scope?: string;
  exp?: number;
}

export interface SpiffeId {
  trustDomain: string;
  namespace: string;
  serviceAccount: string;
  uri: string;
}

export class ServiceAuthError extends Error {
  constructor(message: string, cause?: Error) {
    super(message, { cause });
    this.name = 'ServiceAuthError';
  }
}

export function parseSpiffeId(uri: string): SpiffeId {
  if (!uri.startsWith('spiffe://')) {
    throw new ServiceAuthError('invalid SPIFFE ID: must start with spiffe://');
  }

  const rest = uri.slice('spiffe://'.length);
  const slashIndex = rest.indexOf('/');
  if (slashIndex === -1) {
    throw new ServiceAuthError(`invalid SPIFFE ID format: ${uri}`);
  }

  const trustDomain = rest.slice(0, slashIndex);
  const path = rest.slice(slashIndex + 1);
  const segments = path.split('/');

  // segments: ["ns", "<ns>", "sa", "<sa>"]
  if (segments.length < 4 || segments[0] !== 'ns' || segments[2] !== 'sa') {
    throw new ServiceAuthError(`invalid SPIFFE ID path (expected /ns/<ns>/sa/<sa>): /${path}`);
  }

  return {
    trustDomain,
    namespace: segments[1],
    serviceAccount: segments[3],
    uri,
  };
}

export function validateSpiffeId(uri: string, expectedNamespace: string): SpiffeId {
  const parsed = parseSpiffeId(uri);
  if (parsed.namespace !== expectedNamespace) {
    throw new ServiceAuthError(
      `SPIFFE ID namespace mismatch: expected ${expectedNamespace}, got ${parsed.namespace}`,
    );
  }
  return parsed;
}

export interface ServiceToken {
  accessToken: string;
  tokenType: string;
  expiresAt: Date;
  scope?: string;
}

export function isExpired(token: ServiceToken): boolean {
  return token.expiresAt < new Date();
}

export function shouldRefresh(token: ServiceToken): boolean {
  return token.expiresAt < new Date(Date.now() + 30_000);
}

export function bearerHeader(token: ServiceToken): string {
  return `Bearer ${token.accessToken}`;
}

export interface ServiceAuthConfig {
  tokenEndpoint: string;
  clientId: string;
  clientSecret: string;
}

export interface ServiceAuthClient {
  getToken(): Promise<ServiceToken>;
  getCachedToken(): Promise<string>;
  validateSpiffeId(uri: string, expectedNamespace: string): SpiffeId;
}

export class HttpServiceAuthClient implements ServiceAuthClient {
  private readonly config: ServiceAuthConfig;
  private cached: ServiceToken | null = null;

  constructor(config: ServiceAuthConfig) {
    this.config = config;
  }

  async getToken(): Promise<ServiceToken> {
    const body = new URLSearchParams({
      grant_type: 'client_credentials',
      client_id: this.config.clientId,
      client_secret: this.config.clientSecret,
    });

    const resp = await fetch(this.config.tokenEndpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: body.toString(),
    });

    if (!resp.ok) {
      const text = await resp.text();
      throw new ServiceAuthError(`token request failed (status ${resp.status}): ${text}`);
    }

    const data = (await resp.json()) as {
      access_token: string;
      token_type: string;
      expires_in: number;
      scope?: string;
    };

    return {
      accessToken: data.access_token,
      tokenType: data.token_type,
      expiresAt: new Date(Date.now() + data.expires_in * 1000),
      scope: data.scope,
    };
  }

  async getCachedToken(): Promise<string> {
    if (this.cached && !shouldRefresh(this.cached)) {
      return bearerHeader(this.cached);
    }

    const token = await this.getToken();
    this.cached = token;
    return bearerHeader(token);
  }

  validateSpiffeId(uri: string, expectedNamespace: string): SpiffeId {
    return validateSpiffeId(uri, expectedNamespace);
  }
}
