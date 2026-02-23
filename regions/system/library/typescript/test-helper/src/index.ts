/** テスト用 JWT クレーム */
export interface TestClaims {
  sub: string;
  roles?: string[];
  tenantId?: string;
  iat?: number;
  exp?: number;
}

/** テスト用 JWT トークン生成ヘルパー (HS256 簡易実装) */
export class JwtTestHelper {
  private readonly secret: string;

  constructor(secret: string) {
    this.secret = secret;
  }

  /** 管理者トークンを生成する */
  createAdminToken(): string {
    return this.createToken({ sub: 'admin', roles: ['admin'] });
  }

  /** ユーザートークンを生成する */
  createUserToken(userId: string, roles: string[]): string {
    return this.createToken({ sub: userId, roles });
  }

  /** カスタムクレームでトークンを生成する */
  createToken(claims: TestClaims): string {
    const now = Math.floor(Date.now() / 1000);
    const payload = {
      ...claims,
      iat: claims.iat ?? now,
      exp: claims.exp ?? now + 3600,
    };
    const header = base64UrlEncode(JSON.stringify({ alg: 'HS256', typ: 'JWT' }));
    const body = base64UrlEncode(JSON.stringify(payload));
    const signingInput = `${header}.${body}`;
    const signature = base64UrlEncode(`${signingInput}:${this.secret}`);
    return `${signingInput}.${signature}`;
  }

  /** トークンのペイロードをデコードしてクレームを返す */
  decodeClaims(token: string): TestClaims | null {
    const parts = token.split('.');
    if (parts.length !== 3) return null;
    try {
      return JSON.parse(base64UrlDecode(parts[1]));
    } catch {
      return null;
    }
  }
}

/** モックルート定義 */
interface MockRoute {
  method: string;
  path: string;
  status: number;
  body: string;
}

/** モックサーバー (インメモリ) */
export class MockServer {
  private readonly routes: MockRoute[];
  private readonly requests: Array<{ method: string; path: string }> = [];

  constructor(routes: MockRoute[]) {
    this.routes = routes;
  }

  /** 登録済みルートからレスポンスを取得する */
  handle(method: string, path: string): { status: number; body: string } | null {
    this.requests.push({ method, path });
    const route = this.routes.find((r) => r.method === method && r.path === path);
    return route ? { status: route.status, body: route.body } : null;
  }

  /** 記録されたリクエスト数を返す */
  requestCount(): number {
    return this.requests.length;
  }

  /** 記録されたリクエストを返す */
  recordedRequests(): Array<{ method: string; path: string }> {
    return [...this.requests];
  }
}

/** モックサーバービルダー */
export class MockServerBuilder {
  private serverType: string;
  private routes: MockRoute[] = [];

  private constructor(serverType: string) {
    this.serverType = serverType;
  }

  static notificationServer(): MockServerBuilder {
    return new MockServerBuilder('notification');
  }

  static ratelimitServer(): MockServerBuilder {
    return new MockServerBuilder('ratelimit');
  }

  static tenantServer(): MockServerBuilder {
    return new MockServerBuilder('tenant');
  }

  withHealthOk(): MockServerBuilder {
    this.routes.push({ method: 'GET', path: '/health', status: 200, body: '{"status":"ok"}' });
    return this;
  }

  withSuccessResponse(path: string, body: string): MockServerBuilder {
    this.routes.push({ method: 'POST', path, status: 200, body });
    return this;
  }

  withErrorResponse(path: string, status: number): MockServerBuilder {
    this.routes.push({ method: 'POST', path, status, body: '{"error":"mock error"}' });
    return this;
  }

  build(): MockServer {
    return new MockServer(this.routes);
  }
}

/** テスト用フィクスチャビルダー */
export class FixtureBuilder {
  static uuid(): string {
    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
      const r = (Math.random() * 16) | 0;
      const v = c === 'x' ? r : (r & 0x3) | 0x8;
      return v.toString(16);
    });
  }

  static email(): string {
    return `test-${FixtureBuilder.uuid().substring(0, 8)}@example.com`;
  }

  static name(): string {
    return `user-${FixtureBuilder.uuid().substring(0, 8)}`;
  }

  static int(min = 0, max = 100): number {
    if (min >= max) return min;
    return min + Math.floor(Math.random() * (max - min));
  }

  static tenantId(): string {
    return `tenant-${FixtureBuilder.uuid().substring(0, 8)}`;
  }
}

/** テスト用アサーションヘルパー */
export class AssertionHelper {
  /** JSON 部分一致アサーション */
  static assertJsonContains(actual: unknown, expected: unknown): void {
    if (!jsonContains(actual, expected)) {
      throw new Error(
        `JSON partial match failed.\nActual: ${JSON.stringify(actual)}\nExpected: ${JSON.stringify(expected)}`
      );
    }
  }

  /** イベント一覧に指定タイプのイベントが含まれるか検証する */
  static assertEventEmitted(events: Array<Record<string, unknown>>, eventType: string): void {
    const found = events.some((e) => e.type === eventType);
    if (!found) {
      throw new Error(`Event '${eventType}' not found in events`);
    }
  }
}

function jsonContains(actual: unknown, expected: unknown): boolean {
  if (typeof expected === 'object' && expected !== null && !Array.isArray(expected)) {
    if (typeof actual !== 'object' || actual === null || Array.isArray(actual)) return false;
    const a = actual as Record<string, unknown>;
    const e = expected as Record<string, unknown>;
    return Object.keys(e).every((k) => k in a && jsonContains(a[k], e[k]));
  }
  if (Array.isArray(expected)) {
    if (!Array.isArray(actual)) return false;
    return expected.every((ev) => actual.some((av) => jsonContains(av, ev)));
  }
  return actual === expected;
}

function base64UrlEncode(str: string): string {
  const bytes = new TextEncoder().encode(str);
  let result = '';
  const TABLE = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_';
  let i = 0;
  while (i + 2 < bytes.length) {
    const n = (bytes[i] << 16) | (bytes[i + 1] << 8) | bytes[i + 2];
    result += TABLE[(n >> 18) & 63] + TABLE[(n >> 12) & 63] + TABLE[(n >> 6) & 63] + TABLE[n & 63];
    i += 3;
  }
  const rem = bytes.length - i;
  if (rem === 2) {
    const n = (bytes[i] << 16) | (bytes[i + 1] << 8);
    result += TABLE[(n >> 18) & 63] + TABLE[(n >> 12) & 63] + TABLE[(n >> 6) & 63];
  } else if (rem === 1) {
    const n = bytes[i] << 16;
    result += TABLE[(n >> 18) & 63] + TABLE[(n >> 12) & 63];
  }
  return result;
}

function base64UrlDecode(str: string): string {
  const TABLE = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_';
  const bytes: number[] = [];
  let buf = 0;
  let bits = 0;
  for (const ch of str) {
    const val = TABLE.indexOf(ch);
    if (val === -1) continue;
    buf = (buf << 6) | val;
    bits += 6;
    if (bits >= 8) {
      bits -= 8;
      bytes.push((buf >> bits) & 0xff);
      buf &= (1 << bits) - 1;
    }
  }
  return new TextDecoder().decode(new Uint8Array(bytes));
}
