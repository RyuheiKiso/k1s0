import { useEffect, useState } from 'react';
import { connectionFailureMessage } from '../constants/messages';
import { useAuth } from '../lib/auth';
import {
  clearStoredDeviceAuthSettings,
  loadStoredDeviceAuthSettings,
  storeDeviceAuthSettings,
} from '../lib/device-auth-settings';
import {
  getDeviceAuthorizationDefaults,
  pollDeviceAuthorization,
  startDeviceAuthorization,
  validateDeviceAuthorizationSettings,
  type DeviceAuthorizationChallenge,
  type DeviceAuthorizationSettings,
} from '../lib/tauri-commands';

function formatTimestamp(epochSeconds: number | null | undefined) {
  if (!epochSeconds) {
    return '不明';
  }

  return new Date(epochSeconds * 1000).toLocaleString();
}

function mergeSettings(
  defaults: DeviceAuthorizationSettings,
  stored: Partial<DeviceAuthorizationSettings> | null,
): DeviceAuthorizationSettings {
  return {
    discovery_url: stored?.discovery_url?.trim() || defaults.discovery_url,
    client_id: stored?.client_id?.trim() || defaults.client_id,
    scope: stored?.scope?.trim() || defaults.scope,
  };
}

export default function AuthPage() {
  const auth = useAuth();
  const [challenge, setChallenge] = useState<DeviceAuthorizationChallenge | null>(null);
  const [defaults, setDefaults] = useState<DeviceAuthorizationSettings | null>(null);
  const [settings, setSettings] = useState<DeviceAuthorizationSettings>({
    discovery_url: '',
    client_id: '',
    scope: '',
  });
  const [settingsReady, setSettingsReady] = useState(false);
  const [status, setStatus] = useState<'idle' | 'starting' | 'waiting' | 'success' | 'error'>(
    'idle',
  );
  const [message, setMessage] = useState('');
  const [errorMessage, setErrorMessage] = useState('');
  const [connectionStatus, setConnectionStatus] = useState<
    'idle' | 'loading' | 'success' | 'error'
  >('idle');
  const [connectionMessage, setConnectionMessage] = useState('');

  useEffect(() => {
    let active = true;

    void getDeviceAuthorizationDefaults()
      .then((resolvedDefaults) => {
        if (!active) {
          return;
        }

        setDefaults(resolvedDefaults);
        setSettings(mergeSettings(resolvedDefaults, loadStoredDeviceAuthSettings()));
        setSettingsReady(true);
      })
      .catch((error) => {
        if (!active) {
          return;
        }

        setErrorMessage(String(error));
        setSettingsReady(true);
      });

    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    if (!settingsReady) {
      return;
    }

    storeDeviceAuthSettings(settings);
  }, [settings, settingsReady]);

  useEffect(() => {
    if (!challenge || status !== 'waiting') {
      return;
    }

    let active = true;
    const timeoutId = window.setTimeout(async () => {
      try {
        const result = await pollDeviceAuthorization(challenge);
        if (!active) {
          return;
        }

        switch (result.status) {
          case 'Pending':
            setMessage(result.message);
            setChallenge((current) =>
              current ? { ...current, interval: result.interval } : current,
            );
            break;
          case 'Success':
            auth.setSession(result.session);
            setStatus('success');
            setMessage('認証が完了しました。');
            setChallenge(null);
            break;
          case 'Error':
            setStatus('error');
            setErrorMessage(result.message);
            break;
        }
      } catch (error) {
        if (active) {
          setStatus('error');
          setErrorMessage(String(error));
        }
      }
    }, Math.max(challenge.interval, 1) * 1000);

    return () => {
      active = false;
      window.clearTimeout(timeoutId);
    };
  }, [auth, challenge, status]);

  function updateSetting<K extends keyof DeviceAuthorizationSettings>(
    key: K,
    value: DeviceAuthorizationSettings[K],
  ) {
    setSettings((current) => ({
      ...current,
      [key]: value,
    }));
    setConnectionStatus('idle');
    setConnectionMessage('');
    setErrorMessage('');
  }

  async function handleConnectionCheck() {
    setConnectionStatus('loading');
    setConnectionMessage('');
    setErrorMessage('');

    try {
      const discovery = await validateDeviceAuthorizationSettings(settings);
      setConnectionStatus('success');
      setConnectionMessage(
        `Discovery OK。発行者: ${discovery.issuer} | デバイスエンドポイント: ${discovery.device_authorization_endpoint}`,
      );
    } catch (error) {
      setConnectionStatus('error');
      setConnectionMessage(
        connectionFailureMessage(settings.discovery_url, String(error)),
      );
    }
  }

  async function beginDeviceFlow() {
    setStatus('starting');
    setErrorMessage('');
    setMessage('');

    try {
      storeDeviceAuthSettings(settings);
      const nextChallenge = await startDeviceAuthorization(settings);
      setChallenge(nextChallenge);
      setStatus('waiting');
      setMessage('認証URLを開いてサインインを完了し、このページを開いたままにしてください。');
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    }
  }

  function resetSettings() {
    if (!defaults) {
      return;
    }

    clearStoredDeviceAuthSettings();
    setSettings(defaults);
    setConnectionStatus('idle');
    setConnectionMessage('');
    setErrorMessage('');
  }

  const authFlowBusy = status === 'starting' || status === 'waiting';

  return (
    <div className="p3-animate-in space-y-6" data-testid="auth-page">
      <section className="glass max-w-5xl p-6">
        <p className="text-xs uppercase tracking-[0.24em] text-cyan-100/55">認証</p>
        <h1 className="mt-2 text-3xl font-semibold text-white">GUIセッションの認証</h1>
        <p className="mt-3 max-w-3xl text-sm leading-7 text-slate-200/76">
          OIDC DiscoveryのURL、クライアントID、スコープをGUIで設定し、接続を確認してからDevice Authorization
          Grantフローを開始してください。これらの設定はこのマシンにローカルで保存され、隠れたランタイム環境変数に依存しなくなります。
        </p>
      </section>

      <section className="grid gap-4 lg:grid-cols-[1.05fr_0.95fr]">
        <div className="glass p-6">
          <div className="flex items-center justify-between gap-4">
            <div>
              <h2 className="text-xl font-semibold text-white">OIDC設定</h2>
              <p className="mt-2 text-sm text-slate-200/72">
                サインイン前に接続チェックを実行して、エンドポイントの障害をGUIで確認してください。
              </p>
            </div>
            <span
              className={`px-3 py-1 text-xs font-medium ${
                connectionStatus === 'success'
                  ? 'bg-cyan-400/15 text-cyan-200'
                  : connectionStatus === 'error'
                    ? 'bg-rose-400/15 text-rose-200'
                    : 'bg-slate-400/15 text-slate-200'
              }`}
            >
              {connectionStatus === 'loading'
                ? '確認中'
                : connectionStatus === 'success'
                  ? '到達可能'
                  : connectionStatus === 'error'
                    ? '失敗'
                    : '未確認'}
            </span>
          </div>

          <div className="mt-5 space-y-4">
            <div>
              <label className="block text-sm font-medium text-slate-200/82">
                OIDC Discovery URL
              </label>
              <input
                value={settings.discovery_url}
                onChange={(event) => updateSetting('discovery_url', event.target.value)}
                disabled={!settingsReady || authFlowBusy}
                className="mt-2 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-3 py-2 text-white disabled:opacity-60"
                data-testid="input-discovery-url"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-200/82">クライアントID</label>
              <input
                value={settings.client_id}
                onChange={(event) => updateSetting('client_id', event.target.value)}
                disabled={!settingsReady || authFlowBusy}
                className="mt-2 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-3 py-2 text-white disabled:opacity-60"
                data-testid="input-client-id"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-200/82">スコープ</label>
              <input
                value={settings.scope}
                onChange={(event) => updateSetting('scope', event.target.value)}
                disabled={!settingsReady || authFlowBusy}
                className="mt-2 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-3 py-2 text-white disabled:opacity-60"
                data-testid="input-scope"
              />
            </div>
          </div>

          <div className="mt-5 flex flex-wrap gap-3">
            <button
              type="button"
              onClick={() => {
                void handleConnectionCheck();
              }}
              disabled={!settingsReady || authFlowBusy || connectionStatus === 'loading'}
              className="border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-4 py-2 text-sm font-medium text-white/85 transition hover:bg-[rgba(0,200,255,0.08)] disabled:opacity-50"
              data-testid="btn-check-connection"
            >
              {connectionStatus === 'loading' ? '確認中...' : '接続確認'}
            </button>
            <button
              type="button"
              onClick={() => {
                void beginDeviceFlow();
              }}
              disabled={!settingsReady || auth.loading || authFlowBusy}
              className="bg-cyan-500/85 px-4 py-2 text-sm font-medium text-white transition hover:bg-cyan-500 disabled:opacity-50"
              data-testid="btn-start-auth"
            >
              {authFlowBusy ? '開始中...' : 'デバイスフロー開始'}
            </button>
            <button
              type="button"
              onClick={resetSettings}
              disabled={!defaults || authFlowBusy}
              className="border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-4 py-2 text-sm font-medium text-white/85 transition hover:bg-[rgba(0,200,255,0.08)] disabled:opacity-50"
              data-testid="btn-reset-defaults"
            >
              デフォルトに戻す
            </button>
            {auth.isAuthenticated && (
              <button
                type="button"
                onClick={() => {
                  void auth.clearSession();
                }}
                className="border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-4 py-2 text-sm font-medium text-white/85 transition hover:bg-[rgba(0,200,255,0.08)]"
                data-testid="btn-sign-out"
              >
                セッションを削除
              </button>
            )}
          </div>

          {connectionMessage && (
            <p
              className={`mt-4 text-sm ${
                connectionStatus === 'success' ? 'text-cyan-200' : 'text-rose-300'
              }`}
              data-testid="connection-message"
            >
              {connectionMessage}
            </p>
          )}
          {message && <p className="mt-4 text-sm text-cyan-200">{message}</p>}
          {errorMessage && (
            <p className="mt-4 text-sm text-rose-300" data-testid="error-message">
              {errorMessage}
            </p>
          )}
        </div>

        <div className="space-y-4">
          <div className="glass p-6">
            <h2 className="text-xl font-semibold text-white">セッション状態</h2>
            <p className="mt-2 text-sm text-slate-200/72">
              {auth.loading
                ? 'セキュアなオペレーターセッションを確認しています。'
                : auth.isAuthenticated
                  ? `認証済み: ${formatTimestamp(auth.session?.authenticated_at_epoch_secs)}`
                  : 'セキュアストレージにアクティブなオペレーターセッションがありません。'}
            </p>

            {auth.session && (
              <div className="mt-4 border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-4 text-sm text-slate-200/80">
                <p>
                  <span className="text-slate-200/55">発行者:</span> {auth.session.issuer}
                </p>
                <p className="mt-2">
                  <span className="text-slate-200/55">トークン種別:</span> {auth.session.token_type}
                </p>
                <p className="mt-2">
                  <span className="text-slate-200/55">スコープ:</span>{' '}
                  {auth.session.scope ?? '未設定'}
                </p>
                <p className="mt-2">
                  <span className="text-slate-200/55">有効期限:</span>{' '}
                  {formatTimestamp(auth.session.expires_at_epoch_secs)}
                </p>
              </div>
            )}
          </div>

          <div className="glass p-6">
            <h2 className="text-xl font-semibold text-white">デバイスチャレンジ</h2>
            {!challenge ? (
              <p className="mt-3 text-sm text-slate-200/72">
                デバイスフローを開始して認証URLとユーザーコードを取得してください。
              </p>
            ) : (
              <div className="mt-4 space-y-4 text-sm text-slate-200/82">
                <div>
                  <p className="text-slate-200/55">認証URL</p>
                  <a
                    href={challenge.verification_uri_complete}
                    target="_blank"
                    rel="noreferrer"
                    className="mt-1 block break-all text-cyan-200 underline"
                  >
                    {challenge.verification_uri_complete}
                  </a>
                </div>
                <div>
                  <p className="text-slate-200/55">ユーザーコード</p>
                  <code className="mt-1 block bg-[rgba(5,8,15,0.45)] px-3 py-2 text-base text-white">
                    {challenge.user_code}
                  </code>
                </div>
                <div className="grid gap-3 sm:grid-cols-2">
                  <div className="border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-3">
                    <p className="text-slate-200/55">ポーリング間隔</p>
                    <p className="mt-1 text-white">{challenge.interval}s</p>
                  </div>
                  <div className="border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-3">
                    <p className="text-slate-200/55">有効期限まで</p>
                    <p className="mt-1 text-white">{challenge.expires_in}s</p>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </section>
    </div>
  );
}
