import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { DeviceAuthClient, DeviceFlowError } from '../src/device-flow';
import type { DeviceCodeResponse } from '../src/device-flow';

const DEVICE_ENDPOINT = 'https://auth.example.com/device';
const TOKEN_ENDPOINT = 'https://auth.example.com/token';

describe('DeviceAuthClient', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('requestDeviceCode', () => {
    it('should return DeviceCodeResponse on success', async () => {
      const mockFetch = vi.fn(async (_input: RequestInfo | URL, init?: RequestInit) => {
        const body = init?.body as URLSearchParams;
        expect(body.get('client_id')).toBe('test-client');
        expect(body.get('scope')).toBe('openid profile');

        return new Response(
          JSON.stringify({
            device_code: 'device-code-123',
            user_code: 'ABCD-EFGH',
            verification_uri: 'https://auth.example.com/device',
            verification_uri_complete: 'https://auth.example.com/device?user_code=ABCD-EFGH',
            expires_in: 600,
            interval: 5,
          }),
          { status: 200 },
        );
      });

      const client = new DeviceAuthClient({
        deviceEndpoint: DEVICE_ENDPOINT,
        tokenEndpoint: TOKEN_ENDPOINT,
        fetch: mockFetch,
      });

      const resp = await client.requestDeviceCode('test-client', 'openid profile');
      expect(resp.device_code).toBe('device-code-123');
      expect(resp.user_code).toBe('ABCD-EFGH');
      expect(resp.verification_uri).toBe('https://auth.example.com/device');
      expect(resp.verification_uri_complete).toBe(
        'https://auth.example.com/device?user_code=ABCD-EFGH',
      );
      expect(resp.expires_in).toBe(600);
      expect(resp.interval).toBe(5);
    });
  });

  describe('pollToken', () => {
    it('should poll with authorization_pending then return token', async () => {
      let callCount = 0;
      const mockFetch = vi.fn(async () => {
        callCount++;
        if (callCount <= 2) {
          return new Response(JSON.stringify({ error: 'authorization_pending' }), {
            status: 400,
          });
        }
        return new Response(
          JSON.stringify({
            access_token: 'access-token-xyz',
            refresh_token: 'refresh-token-xyz',
            token_type: 'Bearer',
            expires_in: 900,
          }),
          { status: 200 },
        );
      });

      const client = new DeviceAuthClient({
        deviceEndpoint: DEVICE_ENDPOINT,
        tokenEndpoint: TOKEN_ENDPOINT,
        fetch: mockFetch,
      });

      const pollPromise = client.pollToken('test-client', 'device-code-123', 1);

      // 1回目のポーリング後、タイマーを進める
      await vi.advanceTimersByTimeAsync(1000);
      // 2回目のポーリング後、タイマーを進める
      await vi.advanceTimersByTimeAsync(1000);

      const result = await pollPromise;
      expect(result.access_token).toBe('access-token-xyz');
      expect(result.refresh_token).toBe('refresh-token-xyz');
      expect(result.token_type).toBe('Bearer');
      expect(result.expires_in).toBe(900);
      expect(callCount).toBeGreaterThanOrEqual(3);
    });

    it('should increase interval on slow_down', async () => {
      let callCount = 0;
      const callTimestamps: number[] = [];

      const mockFetch = vi.fn(async () => {
        callCount++;
        callTimestamps.push(Date.now());
        if (callCount === 1) {
          return new Response(JSON.stringify({ error: 'slow_down' }), { status: 400 });
        }
        return new Response(
          JSON.stringify({
            access_token: 'access-token',
            token_type: 'Bearer',
            expires_in: 900,
          }),
          { status: 200 },
        );
      });

      const client = new DeviceAuthClient({
        deviceEndpoint: DEVICE_ENDPOINT,
        tokenEndpoint: TOKEN_ENDPOINT,
        fetch: mockFetch,
      });

      const pollPromise = client.pollToken('test-client', 'device-code-123', 1);

      // slow_down 後は interval が 1+5=6 秒になるので 6 秒進める
      await vi.advanceTimersByTimeAsync(6000);

      const result = await pollPromise;
      expect(result.access_token).toBe('access-token');
      // slow_down 後の待機時間が増加していることを確認
      expect(callTimestamps.length).toBe(2);
      expect(callTimestamps[1]! - callTimestamps[0]!).toBeGreaterThanOrEqual(6000);
    });

    it('should throw DeviceFlowError on expired_token', async () => {
      const mockFetch = vi.fn(async () => {
        return new Response(JSON.stringify({ error: 'expired_token' }), { status: 400 });
      });

      const client = new DeviceAuthClient({
        deviceEndpoint: DEVICE_ENDPOINT,
        tokenEndpoint: TOKEN_ENDPOINT,
        fetch: mockFetch,
      });

      await expect(client.pollToken('test-client', 'device-code-123', 5)).rejects.toThrow(
        DeviceFlowError,
      );

      try {
        await client.pollToken('test-client', 'device-code-123', 5);
      } catch (e) {
        expect(e).toBeInstanceOf(DeviceFlowError);
        expect((e as DeviceFlowError).errorCode).toBe('expired_token');
      }
    });

    it('should throw DeviceFlowError on access_denied', async () => {
      const mockFetch = vi.fn(async () => {
        return new Response(JSON.stringify({ error: 'access_denied' }), { status: 400 });
      });

      const client = new DeviceAuthClient({
        deviceEndpoint: DEVICE_ENDPOINT,
        tokenEndpoint: TOKEN_ENDPOINT,
        fetch: mockFetch,
      });

      try {
        await client.pollToken('test-client', 'device-code-123', 5);
      } catch (e) {
        expect(e).toBeInstanceOf(DeviceFlowError);
        expect((e as DeviceFlowError).errorCode).toBe('access_denied');
      }
    });

    it('should respect AbortSignal for cancellation', async () => {
      const mockFetch = vi.fn(async () => {
        return new Response(JSON.stringify({ error: 'authorization_pending' }), { status: 400 });
      });

      const client = new DeviceAuthClient({
        deviceEndpoint: DEVICE_ENDPOINT,
        tokenEndpoint: TOKEN_ENDPOINT,
        fetch: mockFetch,
      });

      const controller = new AbortController();

      // 即座に abort した signal を渡す
      controller.abort();

      try {
        await client.pollToken('test-client', 'device-code-123', 1, controller.signal);
        expect.unreachable('should have thrown');
      } catch (e) {
        expect(e).toBeInstanceOf(DeviceFlowError);
        expect((e as DeviceFlowError).errorCode).toBe('aborted');
      }
    });
  });

  describe('deviceFlow', () => {
    it('should execute the full device flow', async () => {
      let tokenCallCount = 0;
      const mockFetch = vi.fn(async (_input: RequestInfo | URL, init?: RequestInit) => {
        const body = init?.body as URLSearchParams;

        // device code request (no grant_type)
        if (!body.has('grant_type')) {
          return new Response(
            JSON.stringify({
              device_code: 'device-code-flow',
              user_code: 'WXYZ-1234',
              verification_uri: 'https://auth.example.com/device',
              verification_uri_complete: 'https://auth.example.com/device?user_code=WXYZ-1234',
              expires_in: 600,
              interval: 1,
            }),
            { status: 200 },
          );
        }

        // token request
        tokenCallCount++;
        if (tokenCallCount <= 1) {
          return new Response(JSON.stringify({ error: 'authorization_pending' }), { status: 400 });
        }

        return new Response(
          JSON.stringify({
            access_token: 'flow-access-token',
            refresh_token: 'flow-refresh-token',
            token_type: 'Bearer',
            expires_in: 900,
          }),
          { status: 200 },
        );
      });

      const client = new DeviceAuthClient({
        deviceEndpoint: DEVICE_ENDPOINT,
        tokenEndpoint: TOKEN_ENDPOINT,
        fetch: mockFetch,
      });

      let receivedUserCode = '';
      let receivedVerificationURI = '';

      const flowPromise = client.deviceFlow('test-client', 'openid', (resp: DeviceCodeResponse) => {
        receivedUserCode = resp.user_code;
        receivedVerificationURI = resp.verification_uri;
      });

      // ポーリングのタイマーを進める
      await vi.advanceTimersByTimeAsync(1000);

      const result = await flowPromise;
      expect(result.access_token).toBe('flow-access-token');
      expect(result.refresh_token).toBe('flow-refresh-token');
      expect(receivedUserCode).toBe('WXYZ-1234');
      expect(receivedVerificationURI).toBe('https://auth.example.com/device');
    });
  });
});
