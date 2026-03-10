import { useEffect, useState } from 'react';
import { useAuth } from '../lib/auth';
import {
  pollDeviceAuthorization,
  startDeviceAuthorization,
  type DeviceAuthorizationChallenge,
} from '../lib/tauri-commands';

function formatTimestamp(epochSeconds: number | null | undefined) {
  if (!epochSeconds) {
    return 'unknown time';
  }

  return new Date(epochSeconds * 1000).toLocaleString();
}

export default function AuthPage() {
  const auth = useAuth();
  const [challenge, setChallenge] = useState<DeviceAuthorizationChallenge | null>(null);
  const [status, setStatus] = useState<'idle' | 'starting' | 'waiting' | 'success' | 'error'>(
    'idle',
  );
  const [message, setMessage] = useState('');
  const [errorMessage, setErrorMessage] = useState('');

  async function beginDeviceFlow() {
    setStatus('starting');
    setErrorMessage('');
    setMessage('');

    try {
      const nextChallenge = await startDeviceAuthorization();
      setChallenge(nextChallenge);
      setStatus('waiting');
      setMessage('Open the verification URL, complete sign-in, and keep this page open.');
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    }
  }

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
            setMessage('Authentication completed.');
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

  return (
    <div className="space-y-6" data-testid="auth-page">
      <section className="glass max-w-4xl p-6">
        <p className="text-xs uppercase tracking-[0.24em] text-sky-100/55">Identity</p>
        <h1 className="mt-2 text-3xl font-semibold text-white">Authenticate the GUI session</h1>
        <p className="mt-3 max-w-2xl text-sm leading-7 text-slate-200/76">
          The desktop app uses the Device Authorization Grant flow. Start the flow here, complete
          sign-in in your browser, then return to this page for token polling.
        </p>
      </section>

      <section className="grid gap-4 lg:grid-cols-[1.15fr_0.85fr]">
        <div className="glass p-6">
          <div className="flex items-center justify-between gap-4">
            <div>
              <h2 className="text-xl font-semibold text-white">Session status</h2>
              <p className="mt-2 text-sm text-slate-200/72">
                {auth.loading
                  ? 'Checking the secure operator session.'
                  : auth.isAuthenticated
                    ? `Authenticated at ${formatTimestamp(auth.session?.authenticated_at_epoch_secs)}.`
                    : 'No active operator session is stored in secure storage.'}
              </p>
            </div>
            <span
              className={`rounded-full px-3 py-1 text-xs font-medium ${
                auth.isAuthenticated
                  ? 'bg-emerald-400/15 text-emerald-200'
                  : 'bg-amber-400/15 text-amber-200'
              }`}
            >
              {auth.loading ? 'checking' : auth.isAuthenticated ? 'authenticated' : 'signed out'}
            </span>
          </div>

          {auth.session && (
            <div className="mt-4 rounded-2xl border border-white/10 bg-white/5 p-4 text-sm text-slate-200/80">
              <p>
                <span className="text-slate-200/55">Issuer:</span> {auth.session.issuer}
              </p>
              <p className="mt-2">
                <span className="text-slate-200/55">Token type:</span> {auth.session.token_type}
              </p>
              <p className="mt-2">
                <span className="text-slate-200/55">Scope:</span> {auth.session.scope ?? 'not provided'}
              </p>
              <p className="mt-2">
                <span className="text-slate-200/55">Expires:</span>{' '}
                {formatTimestamp(auth.session.expires_at_epoch_secs)}
              </p>
            </div>
          )}

          <div className="mt-5 flex gap-3">
            <button
              type="button"
              onClick={() => {
                void beginDeviceFlow();
              }}
              disabled={status === 'starting' || status === 'waiting' || auth.loading}
              className="rounded-xl bg-sky-500/85 px-4 py-2 text-sm font-medium text-white transition hover:bg-sky-500 disabled:opacity-50"
              data-testid="btn-start-auth"
            >
              {status === 'starting' || status === 'waiting' ? 'Starting...' : 'Start device flow'}
            </button>
            {auth.isAuthenticated && (
              <button
                type="button"
                onClick={() => {
                  void auth.clearSession();
                }}
                className="rounded-xl border border-white/15 bg-white/6 px-4 py-2 text-sm font-medium text-white/85 transition hover:bg-white/10"
                data-testid="btn-sign-out"
              >
                Clear session
              </button>
            )}
          </div>

          {message && <p className="mt-4 text-sm text-emerald-200">{message}</p>}
          {errorMessage && (
            <p className="mt-4 text-sm text-rose-300" data-testid="error-message">
              {errorMessage}
            </p>
          )}
        </div>

        <div className="glass p-6">
          <h2 className="text-xl font-semibold text-white">Device challenge</h2>
          {!challenge ? (
            <p className="mt-3 text-sm text-slate-200/72">
              Start the device flow to receive a verification URL and user code.
            </p>
          ) : (
            <div className="mt-4 space-y-4 text-sm text-slate-200/82">
              <div>
                <p className="text-slate-200/55">Verification URL</p>
                <a
                  href={challenge.verification_uri_complete}
                  target="_blank"
                  rel="noreferrer"
                  className="mt-1 block break-all text-sky-200 underline"
                >
                  {challenge.verification_uri_complete}
                </a>
              </div>
              <div>
                <p className="text-slate-200/55">User code</p>
                <code className="mt-1 block rounded-xl bg-slate-950/45 px-3 py-2 text-base text-white">
                  {challenge.user_code}
                </code>
              </div>
              <div className="grid gap-3 sm:grid-cols-2">
                <div className="rounded-2xl border border-white/10 bg-white/5 p-3">
                  <p className="text-slate-200/55">Poll interval</p>
                  <p className="mt-1 text-white">{challenge.interval}s</p>
                </div>
                <div className="rounded-2xl border border-white/10 bg-white/5 p-3">
                  <p className="text-slate-200/55">Expires in</p>
                  <p className="mt-1 text-white">{challenge.expires_in}s</p>
                </div>
              </div>
            </div>
          )}
        </div>
      </section>
    </div>
  );
}
