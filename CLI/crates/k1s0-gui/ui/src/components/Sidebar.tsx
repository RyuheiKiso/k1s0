import * as NavigationMenu from '@radix-ui/react-navigation-menu';
import { useLocation, useNavigate } from '@tanstack/react-router';

type Page =
  | 'dashboard'
  | 'auth'
  | 'init'
  | 'generate'
  | 'deps'
  | 'dev'
  | 'migrate'
  | 'template-migrate'
  | 'config-types'
  | 'navigation-types'
  | 'event-codegen'
  | 'validate'
  | 'build'
  | 'test'
  | 'deploy';

const menuItems: { page: Page; path: string; label: string; shortLabel: string }[] = [
  { page: 'dashboard', path: '/', label: 'ダッシュボード', shortLabel: 'DB' },
  { page: 'auth', path: '/auth', label: '認証', shortLabel: 'AU' },
  { page: 'init', path: '/init', label: '初期化', shortLabel: 'IN' },
  { page: 'generate', path: '/generate', label: '生成', shortLabel: 'GN' },
  { page: 'deps', path: '/deps', label: '依存関係マップ', shortLabel: 'DM' },
  { page: 'dev', path: '/dev', label: 'ローカル開発', shortLabel: 'DV' },
  { page: 'migrate', path: '/migrate', label: 'DBマイグレーション', shortLabel: 'DBM' },
  {
    page: 'template-migrate',
    path: '/template-migrate',
    label: 'テンプレート移行',
    shortLabel: 'TMP',
  },
  { page: 'config-types', path: '/config-types', label: '設定型', shortLabel: 'CT' },
  {
    page: 'navigation-types',
    path: '/navigation-types',
    label: 'ナビゲーション型',
    shortLabel: 'NT',
  },
  {
    page: 'event-codegen',
    path: '/event-codegen',
    label: 'イベントコード生成',
    shortLabel: 'EV',
  },
  { page: 'validate', path: '/validate', label: '検証', shortLabel: 'VL' },
  { page: 'build', path: '/build', label: 'ビルド', shortLabel: 'BL' },
  { page: 'test', path: '/test', label: 'テスト', shortLabel: 'TS' },
  { page: 'deploy', path: '/deploy', label: 'デプロイ', shortLabel: 'DP' },
];

export default function Sidebar() {
  const location = useLocation();
  const navigate = useNavigate();

  return (
    <NavigationMenu.Root
      orientation="vertical"
      className="m-4 mb-4 hidden w-72 shrink-0 flex-col border border-[rgba(0,200,255,0.12)] bg-[rgba(5,8,15,0.55)] p-3 shadow-2xl shadow-black/20 backdrop-blur xl:flex"
      data-testid="sidebar"
    >
      {/* サイドバーヘッダー — シアンアクセント */}
      <div className="border border-[rgba(0,200,255,0.10)] bg-[rgba(0,200,255,0.04)] px-5 py-5">
        <p className="text-xs uppercase tracking-[0.32em] text-cyan-100/55 p3-eyebrow-reveal">k1s0</p>
        <h1 className="mt-3 text-2xl font-semibold text-white p3-heading-glitch">GUI コントロール</h1>
        <p className="mt-2 text-sm leading-6 text-slate-200/70">
          ワークスペースのセットアップ、検証、ビルド、テスト、デプロイを別々のツールを使わずに一括で操作できます。
        </p>
      </div>

      {/* ナビゲーションメニュー — アクティブ時にシアングロー */}
      <NavigationMenu.List className="mt-3 flex flex-1 flex-col gap-1">
        {menuItems.map((item, index) => {
          const active = location.pathname === item.path;
          return (
            <NavigationMenu.Item key={item.page}>
              <NavigationMenu.Link asChild>
                <button
                  type="button"
                  onClick={() => navigate({ to: item.path })}
                  className={`p3-nav-slide-in flex w-full items-center gap-3 px-4 py-3 text-left text-sm transition ${
                    active
                      ? 'p3-active-indicator bg-[rgba(0,200,255,0.10)] text-white shadow-lg shadow-cyan-500/10'
                      : 'text-slate-200/72 hover:bg-[rgba(0,200,255,0.06)] hover:text-white'
                  }`}
                  style={{ '--p3-stagger': index } as React.CSSProperties}
                  data-testid={`nav-${item.page}`}
                >
                  <span className="p3-badge-pulse inline-flex h-9 w-9 items-center justify-center border border-[rgba(0,200,255,0.10)] bg-[rgba(0,200,255,0.06)] text-[11px] font-semibold tracking-[0.2em] text-white/85">
                    {item.shortLabel}
                  </span>
                  <span>{item.label}</span>
                </button>
              </NavigationMenu.Link>
            </NavigationMenu.Item>
          );
        })}
      </NavigationMenu.List>
    </NavigationMenu.Root>
  );
}
