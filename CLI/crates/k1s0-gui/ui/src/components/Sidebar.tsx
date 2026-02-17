import * as NavigationMenu from '@radix-ui/react-navigation-menu';
import { useLocation, useNavigate } from '@tanstack/react-router';

type Page = 'init' | 'generate' | 'build' | 'test' | 'deploy';

const menuItems: { page: Page; path: string; label: string }[] = [
  { page: 'init', path: '/', label: 'プロジェクト初期化' },
  { page: 'generate', path: '/generate', label: 'ひな形生成' },
  { page: 'build', path: '/build', label: 'ビルド' },
  { page: 'test', path: '/test', label: 'テスト実行' },
  { page: 'deploy', path: '/deploy', label: 'デプロイ' },
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
    <NavigationMenu.Root orientation="vertical" className="w-56 bg-gray-900 text-gray-100 flex flex-col" data-testid="sidebar">
      <div className="p-4 text-xl font-bold border-b border-gray-700">
        k1s0
      </div>
      <NavigationMenu.List className="flex-1 py-2 flex flex-col">
        {menuItems.map(({ page, path, label }) => (
          <NavigationMenu.Item key={page}>
            <NavigationMenu.Link asChild>
              <button
                onClick={() => navigate({ to: path })}
                className={`w-full text-left px-4 py-2 text-sm hover:bg-gray-800 ${
                  isActive(path) ? 'bg-gray-700 font-semibold' : ''
                }`}
                data-testid={`nav-${page}`}
              >
                {label}
              </button>
            </NavigationMenu.Link>
          </NavigationMenu.Item>
        ))}
      </NavigationMenu.List>
    </NavigationMenu.Root>
  );
}
