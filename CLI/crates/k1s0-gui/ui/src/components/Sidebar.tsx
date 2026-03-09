import * as NavigationMenu from '@radix-ui/react-navigation-menu';
import { useLocation, useNavigate } from '@tanstack/react-router';

type Page =
  | 'dashboard'
  | 'init'
  | 'generate'
  | 'config-types'
  | 'navigation-types'
  | 'validate'
  | 'build'
  | 'test'
  | 'deploy';

const menuItems: { page: Page; path: string; label: string; icon: string }[] = [
  { page: 'dashboard', path: '/', label: 'Dashboard', icon: 'DB' },
  { page: 'init', path: '/init', label: 'Init Project', icon: 'IN' },
  { page: 'generate', path: '/generate', label: 'Generate', icon: 'GN' },
  { page: 'config-types', path: '/config-types', label: 'Config Types', icon: 'CF' },
  { page: 'navigation-types', path: '/navigation-types', label: 'Navigation Types', icon: 'NV' },
  { page: 'validate', path: '/validate', label: 'Validate', icon: 'VL' },
  { page: 'build', path: '/build', label: 'Build', icon: 'BL' },
  { page: 'test', path: '/test', label: 'Test', icon: 'TS' },
  { page: 'deploy', path: '/deploy', label: 'Deploy', icon: 'DP' },
];

export default function Sidebar() {
  const location = useLocation();
  const navigate = useNavigate();

  const isActive = (path: string) => location.pathname === path;

  return (
    <NavigationMenu.Root
      orientation="vertical"
      className="m-3 mr-0 flex w-64 flex-col glass"
      data-testid="sidebar"
    >
      <div className="border-b border-white/10 p-5 text-xl font-bold tracking-[0.2em]">
        <span className="bg-gradient-to-r from-indigo-300 via-sky-300 to-cyan-200 bg-clip-text text-transparent">
          k1s0
        </span>
      </div>
      <NavigationMenu.List className="flex flex-1 flex-col gap-1 px-2 py-3">
        {menuItems.map(({ page, path, label, icon }) => (
          <NavigationMenu.Item key={page}>
            <NavigationMenu.Link asChild>
              <button
                type="button"
                onClick={() => navigate({ to: path })}
                className={`flex w-full items-center gap-3 rounded-xl px-4 py-2.5 text-left text-sm transition-all duration-200 ${
                  isActive(path)
                    ? 'bg-white/15 font-semibold text-white shadow-lg shadow-indigo-500/10'
                    : 'text-white/60 hover:bg-white/8 hover:text-white/90'
                }`}
                data-testid={`nav-${page}`}
              >
                <span className="inline-flex h-7 w-7 items-center justify-center rounded-lg bg-white/8 text-[11px] font-semibold tracking-wide text-white/80">
                  {icon}
                </span>
                <span>{label}</span>
              </button>
            </NavigationMenu.Link>
          </NavigationMenu.Item>
        ))}
      </NavigationMenu.List>
    </NavigationMenu.Root>
  );
}
