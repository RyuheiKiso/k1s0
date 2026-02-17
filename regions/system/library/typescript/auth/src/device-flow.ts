/**
 * Device Authorization Grant フロー（RFC 8628）のクライアント実装。
 * CLI やスマートデバイス等、ブラウザリダイレクトが困難な環境向け。
 */

/** DeviceCodeResponse はデバイス認可リクエストのレスポンス。 */
export interface DeviceCodeResponse {
  device_code: string;
  user_code: string;
  verification_uri: string;
  verification_uri_complete?: string;
  expires_in: number;
  interval: number;
}

/** DeviceTokenResponse はトークンエンドポイントのレスポンス。 */
export interface DeviceTokenResponse {
  access_token: string;
  refresh_token?: string;
  token_type: string;
  expires_in: number;
}

/** DeviceAuthClient のオプション。 */
export interface DeviceAuthClientOptions {
  /** デバイス認可エンドポイント URL */
  deviceEndpoint: string;
  /** トークンエンドポイント URL */
  tokenEndpoint: string;
  /** fetch 関数の注入（テスト用） */
  fetch?: typeof globalThis.fetch;
}

/** DeviceFlowError は Device Authorization Grant フローのエラー。 */
export class DeviceFlowError extends Error {
  readonly errorCode: string;

  constructor(errorCode: string, message?: string) {
    super(message ?? `Device flow error: ${errorCode}`);
    this.name = 'DeviceFlowError';
    this.errorCode = errorCode;
  }
}

/** デバイスコードコールバック。 */
export type DeviceCodeCallback = (resp: DeviceCodeResponse) => void;

/** DeviceAuthClient は Device Authorization Grant フロー（RFC 8628）のクライアント。 */
export class DeviceAuthClient {
  private readonly deviceEndpoint: string;
  private readonly tokenEndpoint: string;
  private readonly fetchFn: typeof globalThis.fetch;

  constructor(options: DeviceAuthClientOptions) {
    this.deviceEndpoint = options.deviceEndpoint;
    this.tokenEndpoint = options.tokenEndpoint;
    this.fetchFn = options.fetch ?? globalThis.fetch.bind(globalThis);
  }

  /**
   * デバイス認可リクエストを送信し、デバイスコード情報を返す。
   */
  async requestDeviceCode(clientId: string, scope?: string): Promise<DeviceCodeResponse> {
    const params = new URLSearchParams({ client_id: clientId });
    if (scope) {
      params.set('scope', scope);
    }

    const resp = await this.fetchFn(this.deviceEndpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: params,
    });

    if (!resp.ok) {
      throw new DeviceFlowError(
        'request_failed',
        `Device code request failed: ${resp.status}`,
      );
    }

    return (await resp.json()) as DeviceCodeResponse;
  }

  /**
   * device_code を使ってトークンエンドポイントをポーリングする。
   * interval が 0 の場合はデフォルトの 5 秒を使用する。
   */
  async pollToken(
    clientId: string,
    deviceCode: string,
    interval: number,
    signal?: AbortSignal,
  ): Promise<DeviceTokenResponse> {
    let intervalMs = (interval <= 0 ? 5 : interval) * 1000;

    // eslint-disable-next-line no-constant-condition
    while (true) {
      const params = new URLSearchParams({
        grant_type: 'urn:ietf:params:oauth:grant-type:device_code',
        device_code: deviceCode,
        client_id: clientId,
      });

      const resp = await this.fetchFn(this.tokenEndpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: params,
      });

      if (resp.ok) {
        return (await resp.json()) as DeviceTokenResponse;
      }

      const errBody = (await resp.json()) as { error: string; error_description?: string };

      switch (errBody.error) {
        case 'authorization_pending':
          break;
        case 'slow_down':
          intervalMs += 5000;
          break;
        case 'expired_token':
          throw new DeviceFlowError('expired_token', 'Device code has expired');
        case 'access_denied':
          throw new DeviceFlowError('access_denied', 'User denied the authorization request');
        default:
          throw new DeviceFlowError(errBody.error, errBody.error_description);
      }

      // interval 待機（AbortSignal 対応）
      await new Promise<void>((resolve, reject) => {
        if (signal?.aborted) {
          reject(new DeviceFlowError('aborted', 'Polling was aborted'));
          return;
        }

        const timer = setTimeout(resolve, intervalMs);

        signal?.addEventListener(
          'abort',
          () => {
            clearTimeout(timer);
            reject(new DeviceFlowError('aborted', 'Polling was aborted'));
          },
          { once: true },
        );
      });
    }
  }

  /**
   * Device Authorization Grant フロー全体を実行する統合メソッド。
   * onUserCode コールバックでユーザーにデバイスコード情報を通知する。
   */
  async deviceFlow(
    clientId: string,
    scope: string | undefined,
    onUserCode: DeviceCodeCallback,
    signal?: AbortSignal,
  ): Promise<DeviceTokenResponse> {
    const deviceResp = await this.requestDeviceCode(clientId, scope);
    onUserCode(deviceResp);
    return this.pollToken(clientId, deviceResp.device_code, deviceResp.interval, signal);
  }
}
