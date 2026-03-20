import { Link, Outlet } from '@tanstack/react-router';
import Sidebar from './Sidebar';
import { useAuth } from '../lib/auth';
import { useWorkspace } from '../lib/workspace';

export default function Layout() {
  const auth = useAuth();
  const workspace = useWorkspace();

  return (
    <div className="min-h-screen">
      {/* CRT走査線オーバーレイ — 画面全体に薄い走査線とスイープラインを描画 */}
      <div className="p3-crt-overlay" />
      {/* P3背景: ミッドナイトブルーグラデーション + アニメーション幾何学グリッド */}
      <div className="fixed inset-0 -z-10 overflow-hidden">
        {/* ベースグラデーション */}
        <div className="absolute inset-0 bg-[linear-gradient(180deg,_#05080f_0%,_#0a1628_50%,_#0d1f35_100%)]" />
        {/* アニメーション幾何学グリッド（60px間隔） */}
        <div
          className="absolute inset-[-60px] opacity-[0.07]"
          style={{
            backgroundImage:
              'linear-gradient(rgba(0,200,255,1) 1px, transparent 1px), linear-gradient(90deg, rgba(0,200,255,1) 1px, transparent 1px)',
            backgroundSize: '60px 60px',
            animation: 'p3-grid-scroll 20s linear infinite',
          }}
        />
        {/* 対角線ストライプオーバーレイ */}
        <div
          className="absolute inset-0 opacity-[0.03]"
          style={{
            backgroundImage:
              'repeating-linear-gradient(45deg, rgba(0,200,255,1) 0, rgba(0,200,255,1) 1px, transparent 1px, transparent 20px)',
            animation: 'p3-stripe-scroll 15s linear infinite',
          }}
        />
        {/* コーナーグローアクセント */}
        <div className="absolute -left-24 -top-24 h-48 w-48 bg-[rgba(0,200,255,0.06)] blur-3xl" style={{ animation: 'p3-glow-pulse 4s ease-in-out infinite' }} />
        <div className="absolute -bottom-24 -right-24 h-56 w-56 bg-[rgba(0,200,255,0.04)] blur-3xl" style={{ animation: 'p3-glow-pulse 5s ease-in-out infinite 1s' }} />
      </div>

      <div className="flex min-h-screen">
        <Sidebar />
        <main className="flex-1 overflow-auto p-6 lg:p-8">
          {/* ワークスペース・認証パネル */}
          <div className="mb-6 grid gap-4 xl:grid-cols-[1.6fr_0.9fr]">
            <section className="glass p-5" data-testid="workspace-panel">
              <div className="mb-3 flex items-center justify-between gap-4">
                <div>
                  <p className="p3-eyebrow-reveal text-xs uppercase tracking-[0.24em] text-cyan-100/55">
                    ワークスペース
                  </p>
                  <h1 className="p3-heading-glow mt-2 text-2xl font-semibold text-white">
                    すべてのフローで共通のワークスペースルートを使用します。
                  </h1>
                </div>
                <span
                  className={`p3-badge-pulse px-3 py-1 text-xs font-medium ${
                    workspace.workspaceRoot
                      ? 'bg-cyan-400/15 text-cyan-200'
                      : 'bg-red-400/15 text-red-200'
                  }`}
                >
                  {workspace.workspaceRoot ? '準備完了' : '未設定'}
                </span>
              </div>

              <div className="flex flex-col gap-3 sm:flex-row">
                <input
                  value={workspace.draftPath}
                  onChange={(event) => workspace.setDraftPath(event.target.value)}
                  placeholder="C:/work/github/k1s0"
                  className="w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-3 py-2 text-sm text-white/90 outline-none"
                  data-testid="workspace-input"
                />
                <button
                  type="button"
                  onClick={() => {
                    void workspace.applyWorkspace();
                  }}
                  disabled={workspace.resolving}
                  className="bg-cyan-500/80 px-4 py-2 text-sm font-medium text-white transition hover:bg-cyan-500 disabled:opacity-50"
                  data-testid="workspace-apply"
                >
                  適用
                </button>
                <button
                  type="button"
                  onClick={() => {
                    void workspace.detectWorkspace();
                  }}
                  disabled={workspace.resolving}
                  className="border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-4 py-2 text-sm font-medium text-white/85 transition hover:bg-[rgba(0,200,255,0.08)] disabled:opacity-50"
                  data-testid="workspace-detect"
                >
                  自動検出
                </button>
              </div>

              <p className="mt-3 text-sm text-slate-200/70">
                {workspace.resolving
                  ? 'ワークスペースルートを解決しています...'
                  : workspace.workspaceRoot
                    ? workspace.workspaceRoot
                    : 'スキャン、生成、デリバリーアクションを実行する前に有効なワークスペースを選択してください。'}
              </p>
              {workspace.errorMessage && (
                <p className="mt-2 text-sm text-rose-300">{workspace.errorMessage}</p>
              )}
            </section>

            <section className="glass p-5" data-testid="auth-panel">
              <p className="p3-eyebrow-reveal text-xs uppercase tracking-[0.24em] text-cyan-100/55">
                認証
              </p>
              <h2 className="p3-heading-glow mt-2 text-xl font-semibold text-white">オペレーターセッション</h2>
              <p className="mt-3 text-sm text-slate-200/75">
                {auth.loading
                  ? 'セキュアなオペレーターセッションを確認しています。'
                  : auth.isAuthenticated
                  ? `${auth.session?.issuer ?? 'IDプロバイダー'} に対して認証済みです。`
                  : 'Device Authorization Grantフローでサインインしてから保護されたアクションを実行してください。'}
              </p>

              <div className="mt-4 flex items-center gap-3">
                <Link
                  to="/auth"
                  className="bg-cyan-500/80 px-4 py-2 text-sm font-medium text-white no-underline transition hover:bg-cyan-500"
                  data-testid="auth-link"
                >
                  {auth.isAuthenticated ? 'セッション確認' : 'サインイン'}
                </Link>
                {auth.isAuthenticated && (
                  <button
                    type="button"
                    onClick={() => {
                      void auth.clearSession();
                    }}
                    className="border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-4 py-2 text-sm font-medium text-white/85 transition hover:bg-[rgba(0,200,255,0.08)]"
                    data-testid="auth-logout"
                  >
                    サインアウト
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
