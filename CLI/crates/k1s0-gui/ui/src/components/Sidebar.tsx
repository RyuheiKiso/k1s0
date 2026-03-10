import * as NavigationMenu from '@radix-ui/react-navigation-menu';
import { useLocation, useNavigate } from '@tanstack/react-router';

type Page =
  | 'dashboard'
  | 'auth'
  | 'init'
  | 'generate'
  | 'config-types'
  | 'navigation-types'
  | 'validate'
  | 'build'
  | 'test'
  | 'deploy';

const menuItems: { page: Page; path: string; label: string; shortLabel: string }[] = [
  { page: 'dashboard', path: '/', label: 'Dashboard', shortLabel: 'DB' },
  { page: 'auth', path: '/auth', label: 'Authentication', shortLabel: 'AU' },
  { page: 'init', path: '/init', label: 'Init', shortLabel: 'IN' },
  { page: 'generate', path: '/generate', label: 'Generate', shortLabel: 'GN' },
  { page: 'config-types', path: '/config-types', label: 'Config Types', shortLabel: 'CT' },
  {
    page: 'navigation-types',
    path: '/navigation-types',
    label: 'Navigation Types',
    shortLabel: 'NT',
  },
  { page: 'validate', path: '/validate', label: 'Validate', shortLabel: 'VL' },
  { page: 'build', path: '/build', label: 'Build', shortLabel: 'BL' },
  { page: 'test', path: '/test', label: 'Test', shortLabel: 'TS' },
  { page: 'deploy', path: '/deploy', label: 'Deploy', shortLabel: 'DP' },
];

export default function Sidebar() {
  const location = useLocation();
  const navigate = useNavigate();

  return (
    <NavigationMenu.Root
      orientation="vertical"
      className="m-4 mb-4 hidden w-72 shrink-0 flex-col rounded-[28px] border border-white/10 bg-slate-950/55 p-3 shadow-2xl shadow-black/20 backdrop-blur xl:flex"
      data-testid="sidebar"
    >
      <div className="rounded-[22px] border border-white/8 bg-white/6 px-5 py-5">
        <p className="text-xs uppercase tracking-[0.32em] text-emerald-100/55">k1s0</p>
        <h1 className="mt-3 text-2xl font-semibold text-white">GUI control surface</h1>
        <p className="mt-2 text-sm leading-6 text-slate-200/70">
          Move through workspace setup, validation, build, test, and deploy without dropping into
          separate tools.
        </p>
      </div>

      <NavigationMenu.List className="mt-3 flex flex-1 flex-col gap-1">
        {menuItems.map((item) => {
          const active = location.pathname === item.path;
          return (
            <NavigationMenu.Item key={item.page}>
              <NavigationMenu.Link asChild>
                <button
                  type="button"
                  onClick={() => navigate({ to: item.path })}
                  className={`flex w-full items-center gap-3 rounded-2xl px-4 py-3 text-left text-sm transition ${
                    active
                      ? 'bg-emerald-400/14 text-white shadow-lg shadow-emerald-500/10'
                      : 'text-slate-200/72 hover:bg-white/8 hover:text-white'
                  }`}
                  data-testid={`nav-${item.page}`}
                >
                  <span className="inline-flex h-9 w-9 items-center justify-center rounded-xl border border-white/8 bg-white/8 text-[11px] font-semibold tracking-[0.2em] text-white/85">
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
