import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { AuthClient, AuthError } from '../src/auth-client';
import { MemoryTokenStore } from '../src/token-store';
import type { AuthConfig, TokenSet, OIDCDiscovery } from '../src/types';

const TEST_DISCOVERY: OIDCDiscovery = {
  authorization_endpoint: 'https://auth.example.com/authorize',
  token_endpoint: 'https://auth.example.com/token',
  end_session_endpoint: 'https://auth.example.com/logout',
  jwks_uri: 'https://auth.example.com/certs',
  issuer: 'https://auth.example.com/realms/k1s0',
};

const TEST_CONFIG: AuthConfig = {
  discoveryUrl: 'https://auth.example.com/.well-known/openid-configuration',
  clientId: 'test-client',
  redirectUri: 'https://app.example.com/callback',
  scopes: ['openid', 'profile', 'email'],
  postLogoutRedirectUri: 'https://app.example.com/',
};

function createMockFetch(responses: Record<string, unknown> = {}) {
  return vi.fn(async (input: RequestInfo | URL, _init?: RequestInit) => {
    const url = typeof input === 'string' ? input : input.toString();

    if (url === TEST_CONFIG.discoveryUrl) {
      return new Response(JSON.stringify(TEST_DISCOVERY), { status: 200 });
    }

    if (url === TEST_DISCOVERY.token_endpoint) {
      const tokenResponse = responses['token'] ?? {
        access_token: 'mock-access-token',
        refresh_token: 'mock-refresh-token',
        id_token: 'mock-id-token',
        expires_in: 900,
        token_type: 'Bearer',
      };
      return new Response(JSON.stringify(tokenResponse), { status: 200 });
    }

    return new Response('Not Found', { status: 404 });
  });
}

describe('AuthClient', () => {
  let tokenStore: MemoryTokenStore;
  let redirectUrl: string | null;
  let mockFetch: ReturnType<typeof createMockFetch>;

  beforeEach(() => {
    tokenStore = new MemoryTokenStore();
    redirectUrl = null;
    mockFetch = createMockFetch();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  function createClient(overrides: Partial<ConstructorParameters<typeof AuthClient>[0]> = {}) {
    return new AuthClient({
      config: TEST_CONFIG,
      tokenStore,
      fetch: mockFetch,
      redirect: (url: string) => {
        redirectUrl = url;
      },
      generateState: () => 'mock-state-value',
      ...overrides,
    });
  }

  describe('login', () => {
    it('should redirect to the authorization endpoint with PKCE params', async () => {
      const client = createClient();
      await client.login();

      expect(redirectUrl).not.toBeNull();
      const url = new URL(redirectUrl!);
      expect(url.origin + url.pathname).toBe(TEST_DISCOVERY.authorization_endpoint);
      expect(url.searchParams.get('response_type')).toBe('code');
      expect(url.searchParams.get('client_id')).toBe(TEST_CONFIG.clientId);
      expect(url.searchParams.get('redirect_uri')).toBe(TEST_CONFIG.redirectUri);
      expect(url.searchParams.get('scope')).toBe('openid profile email');
      expect(url.searchParams.get('code_challenge_method')).toBe('S256');
      expect(url.searchParams.get('code_challenge')).toBeTruthy();
      expect(url.searchParams.get('state')).toBe('mock-state-value');
    });

    it('should store code_verifier and state', async () => {
      const client = createClient();
      await client.login();

      expect(tokenStore.getCodeVerifier()).toBeTruthy();
      expect(tokenStore.getState()).toBe('mock-state-value');
    });

    it('should fetch the OIDC discovery document', async () => {
      const client = createClient();
      await client.login();

      expect(mockFetch).toHaveBeenCalledWith(TEST_CONFIG.discoveryUrl);
    });
  });

  describe('handleCallback', () => {
    it('should exchange code for tokens', async () => {
      const client = createClient();
      // Setup: call login first to store verifier and state
      await client.login();

      vi.setSystemTime(new Date('2026-01-01T00:00:00Z'));
      const tokenSet = await client.handleCallback('auth-code-123', 'mock-state-value');

      expect(tokenSet.accessToken).toBe('mock-access-token');
      expect(tokenSet.refreshToken).toBe('mock-refresh-token');
      expect(tokenSet.idToken).toBe('mock-id-token');
      expect(tokenSet.expiresAt).toBe(Date.now() + 900_000);
    });

    it('should send correct parameters to token endpoint', async () => {
      const client = createClient();
      await client.login();
      await client.handleCallback('auth-code-123', 'mock-state-value');

      // Find the token endpoint call
      const tokenCall = mockFetch.mock.calls.find(
        ([url]) => url === TEST_DISCOVERY.token_endpoint,
      );
      expect(tokenCall).toBeTruthy();

      const body = tokenCall![1]?.body as URLSearchParams;
      expect(body.get('grant_type')).toBe('authorization_code');
      expect(body.get('client_id')).toBe(TEST_CONFIG.clientId);
      expect(body.get('code')).toBe('auth-code-123');
      expect(body.get('redirect_uri')).toBe(TEST_CONFIG.redirectUri);
      expect(body.get('code_verifier')).toBeTruthy();
    });

    it('should throw on state mismatch', async () => {
      const client = createClient();
      await client.login();

      await expect(
        client.handleCallback('code', 'wrong-state'),
      ).rejects.toThrow('State mismatch');
    });

    it('should throw when PKCE verifier is missing', async () => {
      const client = createClient();
      tokenStore.setState('mock-state-value');
      // Don't set code verifier

      await expect(
        client.handleCallback('code', 'mock-state-value'),
      ).rejects.toThrow('Missing PKCE verifier');
    });

    it('should throw on token request failure', async () => {
      const failFetch = vi.fn(async (input: RequestInfo | URL) => {
        const url = typeof input === 'string' ? input : input.toString();
        if (url === TEST_CONFIG.discoveryUrl) {
          return new Response(JSON.stringify(TEST_DISCOVERY), { status: 200 });
        }
        return new Response('Unauthorized', { status: 401 });
      });

      const client = createClient({ fetch: failFetch });
      await client.login();

      await expect(
        client.handleCallback('code', 'mock-state-value'),
      ).rejects.toThrow('Token request failed: 401');
    });

    it('should notify listeners on successful callback', async () => {
      const client = createClient();
      const listener = vi.fn();
      client.onAuthStateChange(listener);

      await client.login();
      await client.handleCallback('code', 'mock-state-value');

      expect(listener).toHaveBeenCalledWith(true);
    });

    it('should clear code verifier and state after successful callback', async () => {
      const client = createClient();
      await client.login();
      await client.handleCallback('code', 'mock-state-value');

      expect(tokenStore.getCodeVerifier()).toBeNull();
      expect(tokenStore.getState()).toBeNull();
    });

    it('should store the token set', async () => {
      const client = createClient();
      await client.login();
      await client.handleCallback('code', 'mock-state-value');

      const stored = tokenStore.getTokenSet();
      expect(stored).not.toBeNull();
      expect(stored!.accessToken).toBe('mock-access-token');
    });
  });

  describe('getAccessToken', () => {
    it('should return the access token when valid', async () => {
      vi.setSystemTime(new Date('2026-01-01T00:00:00Z'));
      tokenStore.setTokenSet({
        accessToken: 'valid-token',
        refreshToken: 'refresh-token',
        idToken: 'id-token',
        expiresAt: Date.now() + 300_000, // 5 minutes from now
      });

      const client = createClient();
      const token = await client.getAccessToken();
      expect(token).toBe('valid-token');
    });

    it('should throw when not authenticated', async () => {
      const client = createClient();
      await expect(client.getAccessToken()).rejects.toThrow('Not authenticated');
    });

    it('should auto-refresh when token expires within 60 seconds', async () => {
      vi.setSystemTime(new Date('2026-01-01T00:00:00Z'));
      tokenStore.setTokenSet({
        accessToken: 'expiring-token',
        refreshToken: 'refresh-token',
        idToken: 'id-token',
        expiresAt: Date.now() + 30_000, // 30 seconds from now (< 60s threshold)
      });

      const refreshedFetch = createMockFetch({
        token: {
          access_token: 'refreshed-token',
          refresh_token: 'new-refresh-token',
          id_token: 'new-id-token',
          expires_in: 900,
          token_type: 'Bearer',
        },
      });

      const client = createClient({ fetch: refreshedFetch });
      const token = await client.getAccessToken();
      expect(token).toBe('refreshed-token');
    });
  });

  describe('refreshToken', () => {
    it('should exchange refresh token for new tokens', async () => {
      vi.setSystemTime(new Date('2026-01-01T00:00:00Z'));
      tokenStore.setTokenSet({
        accessToken: 'old-access',
        refreshToken: 'old-refresh',
        idToken: 'old-id',
        expiresAt: Date.now() + 100_000,
      });

      const client = createClient();
      await client.refreshToken();

      const newTokenSet = tokenStore.getTokenSet();
      expect(newTokenSet!.accessToken).toBe('mock-access-token');
      expect(newTokenSet!.refreshToken).toBe('mock-refresh-token');
    });

    it('should throw when no refresh token is available', async () => {
      const client = createClient();
      await expect(client.refreshToken()).rejects.toThrow('No refresh token');
    });

    it('should send correct refresh parameters', async () => {
      tokenStore.setTokenSet({
        accessToken: 'access',
        refreshToken: 'my-refresh-token',
        idToken: 'id',
        expiresAt: Date.now() + 100_000,
      });

      const client = createClient();
      await client.refreshToken();

      const tokenCall = mockFetch.mock.calls.find(
        ([url]) => url === TEST_DISCOVERY.token_endpoint,
      );
      const body = tokenCall![1]?.body as URLSearchParams;
      expect(body.get('grant_type')).toBe('refresh_token');
      expect(body.get('client_id')).toBe(TEST_CONFIG.clientId);
      expect(body.get('refresh_token')).toBe('my-refresh-token');
    });

    it('should clear tokens and notify listeners on refresh failure', async () => {
      tokenStore.setTokenSet({
        accessToken: 'access',
        refreshToken: 'expired-refresh',
        idToken: 'id',
        expiresAt: Date.now() + 100_000,
      });

      const failFetch = vi.fn(async (input: RequestInfo | URL) => {
        const url = typeof input === 'string' ? input : input.toString();
        if (url === TEST_CONFIG.discoveryUrl) {
          return new Response(JSON.stringify(TEST_DISCOVERY), { status: 200 });
        }
        return new Response('Forbidden', { status: 403 });
      });

      const client = createClient({ fetch: failFetch });
      const listener = vi.fn();
      client.onAuthStateChange(listener);

      await expect(client.refreshToken()).rejects.toThrow('Token refresh failed: 403');
      expect(tokenStore.getTokenSet()).toBeNull();
      expect(listener).toHaveBeenCalledWith(false);
    });
  });

  describe('isAuthenticated', () => {
    it('should return false when no token set', () => {
      const client = createClient();
      expect(client.isAuthenticated()).toBe(false);
    });

    it('should return true when token is valid', () => {
      tokenStore.setTokenSet({
        accessToken: 'token',
        refreshToken: 'refresh',
        idToken: 'id',
        expiresAt: Date.now() + 300_000,
      });
      const client = createClient();
      expect(client.isAuthenticated()).toBe(true);
    });

    it('should return false when token has expired', () => {
      tokenStore.setTokenSet({
        accessToken: 'token',
        refreshToken: 'refresh',
        idToken: 'id',
        expiresAt: Date.now() - 1000,
      });
      const client = createClient();
      expect(client.isAuthenticated()).toBe(false);
    });
  });

  describe('logout', () => {
    it('should clear tokens', async () => {
      tokenStore.setTokenSet({
        accessToken: 'token',
        refreshToken: 'refresh',
        idToken: 'id',
        expiresAt: Date.now() + 300_000,
      });
      const client = createClient();
      await client.logout();

      expect(tokenStore.getTokenSet()).toBeNull();
    });

    it('should notify listeners', async () => {
      tokenStore.setTokenSet({
        accessToken: 'token',
        refreshToken: 'refresh',
        idToken: 'id',
        expiresAt: Date.now() + 300_000,
      });
      const client = createClient();
      const listener = vi.fn();
      client.onAuthStateChange(listener);
      await client.logout();

      expect(listener).toHaveBeenCalledWith(false);
    });

    it('should redirect to end_session_endpoint with id_token_hint', async () => {
      tokenStore.setTokenSet({
        accessToken: 'token',
        refreshToken: 'refresh',
        idToken: 'my-id-token',
        expiresAt: Date.now() + 300_000,
      });
      const client = createClient();
      await client.logout();

      expect(redirectUrl).not.toBeNull();
      const url = new URL(redirectUrl!);
      expect(url.origin + url.pathname).toBe(TEST_DISCOVERY.end_session_endpoint);
      expect(url.searchParams.get('id_token_hint')).toBe('my-id-token');
      expect(url.searchParams.get('post_logout_redirect_uri')).toBe(TEST_CONFIG.postLogoutRedirectUri);
      expect(url.searchParams.get('client_id')).toBe(TEST_CONFIG.clientId);
    });

    it('should not redirect when no token set exists', async () => {
      const client = createClient();
      await client.logout();

      expect(redirectUrl).toBeNull();
    });
  });

  describe('onAuthStateChange', () => {
    it('should register and notify a listener', async () => {
      const client = createClient();
      const listener = vi.fn();
      client.onAuthStateChange(listener);

      await client.login();
      await client.handleCallback('code', 'mock-state-value');

      expect(listener).toHaveBeenCalledWith(true);
    });

    it('should return an unsubscribe function', async () => {
      const client = createClient();
      const listener = vi.fn();
      const unsubscribe = client.onAuthStateChange(listener);

      unsubscribe();

      await client.login();
      await client.handleCallback('code', 'mock-state-value');

      expect(listener).not.toHaveBeenCalled();
    });

    it('should support multiple listeners', async () => {
      const client = createClient();
      const listener1 = vi.fn();
      const listener2 = vi.fn();
      client.onAuthStateChange(listener1);
      client.onAuthStateChange(listener2);

      await client.login();
      await client.handleCallback('code', 'mock-state-value');

      expect(listener1).toHaveBeenCalledWith(true);
      expect(listener2).toHaveBeenCalledWith(true);
    });
  });

  describe('getTokenSet', () => {
    it('should return null when no tokens', () => {
      const client = createClient();
      expect(client.getTokenSet()).toBeNull();
    });

    it('should return stored token set', () => {
      const ts: TokenSet = {
        accessToken: 'a',
        refreshToken: 'r',
        idToken: 'i',
        expiresAt: Date.now() + 300_000,
      };
      tokenStore.setTokenSet(ts);
      const client = createClient();
      expect(client.getTokenSet()).toEqual(ts);
    });
  });

  describe('discovery caching', () => {
    it('should cache the discovery document', async () => {
      const client = createClient();
      await client.login();
      // Reset the redirect to detect a second login redirect
      redirectUrl = null;
      await client.login();

      // discovery endpoint should only be fetched once (cached on second call)
      const discoveryCalls = mockFetch.mock.calls.filter(
        ([url]) => url === TEST_CONFIG.discoveryUrl,
      );
      expect(discoveryCalls.length).toBe(1);
    });
  });

  describe('AuthError', () => {
    it('should have the correct name and message', () => {
      const error = new AuthError('test error');
      expect(error.name).toBe('AuthError');
      expect(error.message).toBe('test error');
      expect(error).toBeInstanceOf(Error);
    });
  });
});
