import { Link, Outlet } from '@tanstack/react-router';
import Sidebar from './Sidebar';
import { useAuth } from '../lib/auth';
import { useWorkspace } from '../lib/workspace';

export default function Layout() {
  const auth = useAuth();
  const workspace = useWorkspace();

  return (
    <div className="min-h-screen">
      <div className="fixed inset-0 -z-10 overflow-hidden">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_top_left,_rgba(21,128,61,0.18),_transparent_35%),radial-gradient(circle_at_bottom_right,_rgba(14,165,233,0.14),_transparent_32%),linear-gradient(135deg,_#08111d_0%,_#0f172a_40%,_#10243b_100%)]" />
        <div className="absolute -left-16 top-8 h-56 w-56 rounded-full bg-emerald-300/10 blur-3xl" />
        <div className="absolute bottom-0 right-0 h-72 w-72 rounded-full bg-sky-300/10 blur-3xl" />
      </div>

      <div className="flex min-h-screen">
        <Sidebar />
        <main className="flex-1 overflow-auto p-6 lg:p-8">
          <div className="mb-6 grid gap-4 xl:grid-cols-[1.6fr_0.9fr]">
            <section className="glass p-5" data-testid="workspace-panel">
              <div className="mb-3 flex items-center justify-between gap-4">
                <div>
                  <p className="text-xs uppercase tracking-[0.24em] text-emerald-100/55">
                    Workspace
                  </p>
                  <h1 className="mt-2 text-2xl font-semibold text-white">
                    Use one verified workspace root across all flows.
                  </h1>
                </div>
                <span
                  className={`rounded-full px-3 py-1 text-xs font-medium ${
                    workspace.workspaceRoot
                      ? 'bg-emerald-400/15 text-emerald-200'
                      : 'bg-amber-400/15 text-amber-200'
                  }`}
                >
                  {workspace.workspaceRoot ? 'ready' : 'not set'}
                </span>
              </div>

              <div className="flex flex-col gap-3 sm:flex-row">
                <input
                  value={workspace.draftPath}
                  onChange={(event) => workspace.setDraftPath(event.target.value)}
                  placeholder="C:/work/github/k1s0"
                  className="w-full rounded-xl border border-white/15 bg-white/6 px-3 py-2 text-sm text-white/90 outline-none"
                  data-testid="workspace-input"
                />
                <button
                  type="button"
                  onClick={() => {
                    void workspace.applyWorkspace();
                  }}
                  disabled={workspace.resolving}
                  className="rounded-xl bg-emerald-500/80 px-4 py-2 text-sm font-medium text-white transition hover:bg-emerald-500 disabled:opacity-50"
                  data-testid="workspace-apply"
                >
                  Apply
                </button>
                <button
                  type="button"
                  onClick={() => {
                    void workspace.detectWorkspace();
                  }}
                  disabled={workspace.resolving}
                  className="rounded-xl border border-white/15 bg-white/6 px-4 py-2 text-sm font-medium text-white/85 transition hover:bg-white/10 disabled:opacity-50"
                  data-testid="workspace-detect"
                >
                  Detect
                </button>
              </div>

              <p className="mt-3 text-sm text-slate-200/70">
                {workspace.resolving
                  ? 'Resolving workspace root...'
                  : workspace.workspaceRoot
                    ? workspace.workspaceRoot
                    : 'Choose a valid workspace before running scans, generation, or delivery actions.'}
              </p>
              {workspace.errorMessage && (
                <p className="mt-2 text-sm text-rose-300">{workspace.errorMessage}</p>
              )}
            </section>

            <section className="glass p-5" data-testid="auth-panel">
              <p className="text-xs uppercase tracking-[0.24em] text-sky-100/55">
                Authentication
              </p>
              <h2 className="mt-2 text-xl font-semibold text-white">Operator session</h2>
              <p className="mt-3 text-sm text-slate-200/75">
                {auth.loading
                  ? 'Checking the secure operator session before enabling protected actions.'
                  : auth.isAuthenticated
                  ? `Authenticated against ${auth.session?.issuer ?? 'the identity provider'}.`
                  : 'Sign in with the Device Authorization Grant flow before running protected actions.'}
              </p>

              <div className="mt-4 flex items-center gap-3">
                <Link
                  to="/auth"
                  className="rounded-xl bg-sky-500/80 px-4 py-2 text-sm font-medium text-white no-underline transition hover:bg-sky-500"
                  data-testid="auth-link"
                >
                  {auth.isAuthenticated ? 'View session' : 'Sign in'}
                </Link>
                {auth.isAuthenticated && (
                  <button
                    type="button"
                    onClick={() => {
                      void auth.clearSession();
                    }}
                    className="rounded-xl border border-white/15 bg-white/6 px-4 py-2 text-sm font-medium text-white/85 transition hover:bg-white/10"
                    data-testid="auth-logout"
                  >
                    Sign out
                  </button>
                )}
              </div>
            </section>
          </div>

          <Outlet />
        </main>
      </div>
    </div>
  );
}
