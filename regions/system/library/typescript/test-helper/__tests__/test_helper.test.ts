import { describe, it, expect } from 'vitest';
import {
  JwtTestHelper,
  MockServerBuilder,
  FixtureBuilder,
  AssertionHelper,
} from '../src/index.js';

describe('JwtTestHelper', () => {
  const helper = new JwtTestHelper('test-secret');

  it('should create admin token', () => {
    const token = helper.createAdminToken();
    const parts = token.split('.');
    expect(parts).toHaveLength(3);
    const claims = helper.decodeClaims(token);
    expect(claims?.sub).toBe('admin');
    expect(claims?.roles).toEqual(['admin']);
  });

  it('should create user token', () => {
    const token = helper.createUserToken('user-123', ['user']);
    const claims = helper.decodeClaims(token);
    expect(claims?.sub).toBe('user-123');
    expect(claims?.roles).toEqual(['user']);
  });

  it('should create token with tenant', () => {
    const token = helper.createToken({
      sub: 'svc',
      roles: ['service'],
      tenantId: 't-1',
    });
    const claims = helper.decodeClaims(token);
    expect(claims?.tenantId).toBe('t-1');
  });

  it('should return null for invalid token', () => {
    expect(helper.decodeClaims('invalid')).toBeNull();
  });
});

describe('MockServerBuilder', () => {
  it('should build notification server mock', () => {
    const server = MockServerBuilder.notificationServer()
      .withHealthOk()
      .withSuccessResponse('/send', '{"id":"1","status":"sent"}')
      .build();

    const health = server.handle('GET', '/health');
    expect(health?.status).toBe(200);
    expect(health?.body).toContain('ok');

    const send = server.handle('POST', '/send');
    expect(send?.status).toBe(200);

    expect(server.requestCount()).toBe(2);
  });

  it('should return null for unknown route', () => {
    const server = MockServerBuilder.ratelimitServer().withHealthOk().build();
    expect(server.handle('GET', '/nonexistent')).toBeNull();
  });

  it('should support error responses', () => {
    const server = MockServerBuilder.tenantServer()
      .withErrorResponse('/create', 500)
      .build();
    const res = server.handle('POST', '/create');
    expect(res?.status).toBe(500);
    expect(res?.body).toContain('error');
  });
});

describe('FixtureBuilder', () => {
  it('should generate valid UUID', () => {
    const id = FixtureBuilder.uuid();
    expect(id).toHaveLength(36);
    expect(id).toContain('-');
  });

  it('should generate email', () => {
    const email = FixtureBuilder.email();
    expect(email).toContain('@example.com');
  });

  it('should generate name with prefix', () => {
    const name = FixtureBuilder.name();
    expect(name).toMatch(/^user-/);
  });

  it('should generate int in range', () => {
    for (let i = 0; i < 100; i++) {
      const val = FixtureBuilder.int(10, 20);
      expect(val).toBeGreaterThanOrEqual(10);
      expect(val).toBeLessThan(20);
    }
  });

  it('should return min when min equals max', () => {
    expect(FixtureBuilder.int(5, 5)).toBe(5);
  });

  it('should generate tenant id', () => {
    expect(FixtureBuilder.tenantId()).toMatch(/^tenant-/);
  });

  it('should generate unique values', () => {
    const a = FixtureBuilder.uuid();
    const b = FixtureBuilder.uuid();
    expect(a).not.toBe(b);
  });
});

describe('AssertionHelper', () => {
  it('should pass on JSON partial match', () => {
    AssertionHelper.assertJsonContains(
      { id: '1', status: 'ok', extra: 'ignored' },
      { id: '1', status: 'ok' }
    );
  });

  it('should pass on nested JSON partial match', () => {
    AssertionHelper.assertJsonContains(
      { user: { id: '1', name: 'test' }, status: 'ok' },
      { user: { id: '1' } }
    );
  });

  it('should fail on JSON mismatch', () => {
    expect(() => {
      AssertionHelper.assertJsonContains({ id: '1' }, { id: '2' });
    }).toThrow('JSON partial match failed');
  });

  it('should find emitted event', () => {
    const events = [
      { type: 'created', id: '1' },
      { type: 'updated', id: '2' },
    ];
    AssertionHelper.assertEventEmitted(events, 'created');
    AssertionHelper.assertEventEmitted(events, 'updated');
  });

  it('should throw for missing event', () => {
    expect(() => {
      AssertionHelper.assertEventEmitted([{ type: 'created' }], 'deleted');
    }).toThrow('not found');
  });
});
