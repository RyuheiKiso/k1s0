import * as NavigationMenu from '@radix-ui/react-navigation-menu';
import { useLocation, useNavigate } from '@tanstack/react-router';

type Page = 'init' | 'generate' | 'config-types' | 'navigation-types' | 'validate' | 'build' | 'test' | 'deploy';

const menuItems: { page: Page; path: string; label: string; icon: string }[] = [
  { page: 'init', path: '/', label: 'プロジェクト初期化', icon: '⚡' },
  { page: 'generate', path: '/generate', label: 'ひな形生成', icon: '🔧' },
  { page: 'config-types', path: '/config-types', label: '設定スキーマ型生成', icon: '⚙️' },
  { page: 'navigation-types', path: '/navigation-types', label: 'ナビゲーション型生成', icon: '🗺️' },
  { page: 'validate', path: '/validate', label: 'バリデーション', icon: '✅' },
  { page: 'build', path: '/build', label: 'ビルド', icon: '📦' },
  { page: 'test', path: '/test', label: 'テスト実行', icon: '🧪' },
  { page: 'deploy', path: '/deploy', label: 'デプロイ', icon: '🚀' },
];

export default function Sidebar() {
  const location = useLocation();
  const navigate = useNavigate();

  const isActive = (path: string) => {
    if (path === '/') {
      return location.pathname === '/';
    }
    return location.pathname === path;
  };

  return (
    <NavigationMenu.Root
      orientation="vertical"
      className="w-60 flex flex-col m-3 mr-0 glass"
      data-testid="sidebar"
    >
      <div className="p-5 text-xl font-bold border-b border-white/10 tracking-wider">
        <span className="bg-gradient-to-r from-indigo-400 to-purple-400 bg-clip-text text-transparent">
          k1s0
        </span>
      </div>
      <NavigationMenu.List className="flex-1 py-3 flex flex-col gap-1 px-2">
        {menuItems.map(({ page, path, label, icon }) => (
          <NavigationMenu.Item key={page}>
            <NavigationMenu.Link asChild>
              <button
                onClick={() => navigate({ to: path })}
                className={`w-full text-left px-4 py-2.5 text-sm rounded-xl transition-all duration-200 flex items-center gap-3 ${
                  isActive(path)
                    ? 'bg-white/15 text-white font-semibold shadow-lg shadow-indigo-500/10'
                    : 'text-white/60 hover:bg-white/8 hover:text-white/90'
                }`}
                data-testid={`nav-${page}`}
              >
                <span className="text-base">{icon}</span>
                {label}
              </button>
            </NavigationMenu.Link>
          </NavigationMenu.Item>
        ))}
      </NavigationMenu.List>
    </NavigationMenu.Root>
  );
}
