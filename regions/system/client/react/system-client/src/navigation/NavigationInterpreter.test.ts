import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import { NavigationInterpreter } from './NavigationInterpreter';
import type { NavigationResponse, ComponentRegistry } from './types';

const mockNavigation: NavigationResponse = {
  guards: [
    {
      id: 'auth_required',
      type: 'auth_required',
      redirect_to: '/login',
    },
    {
      id: 'admin_only',
      type: 'role_required',
      redirect_to: '/dashboard',
      roles: ['admin'],
    },
  ],
  routes: [
    {
      id: 'root',
      path: '/',
      redirect_to: '/dashboard',
    },
    {
      id: 'login',
      path: '/login',
      component_id: 'LoginPage',
      guards: [],
    },
    {
      id: 'dashboard',
      path: '/dashboard',
      component_id: 'DashboardPage',
      guards: ['auth_required'],
      transition: 'fade',
    },
    {
      id: 'admin',
      path: '/admin',
      component_id: 'AdminPage',
      guards: ['auth_required', 'admin_only'],
      children: [
        {
          id: 'admin_users',
          path: '/admin/users',
          component_id: 'AdminUsersPage',
          guards: ['auth_required', 'admin_only'],
        },
      ],
    },
  ],
};

function MockLoginPage() {
  return null;
}

const mockRegistry: ComponentRegistry = {
  LoginPage: MockLoginPage,
  DashboardPage: () => Promise.resolve({ default: MockLoginPage }),
  AdminPage: () => Promise.resolve({ default: MockLoginPage }),
  AdminUsersPage: () => Promise.resolve({ default: MockLoginPage }),
};

const server = setupServer(
  http.get('http://localhost/api/v1/navigation', () => {
    return HttpResponse.json(mockNavigation);
  }),
);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

describe('NavigationInterpreter', () => {
  it('remote modeでAPIからnavigationを取得してrouterが構築される', async () => {
    const interpreter = new NavigationInterpreter({
      mode: 'remote',
      remoteUrl: 'http://localhost/api/v1/navigation',
      componentRegistry: mockRegistry,
    });

    const result = await interpreter.build();

    expect(result.guards).toHaveLength(2);
    expect(result.routes).toHaveLength(4);
    expect(result.raw).toEqual(mockNavigation);

    // root route has redirect
    const root = result.routes[0];
    expect(root.id).toBe('root');
    expect(root.redirect_to).toBe('/dashboard');
    expect(root.component).toBeUndefined();
    expect(root.guards).toHaveLength(0);

    // login route has no guards and a direct component
    const login = result.routes[1];
    expect(login.id).toBe('login');
    expect(login.component).toBe(MockLoginPage);
    expect(login.lazyComponent).toBeUndefined();
    expect(login.guards).toHaveLength(0);

    // dashboard route has auth guard and lazy component
    const dashboard = result.routes[2];
    expect(dashboard.id).toBe('dashboard');
    expect(dashboard.component).toBeUndefined();
    expect(dashboard.lazyComponent).toBeDefined();
    expect(dashboard.guards).toHaveLength(1);
    expect(dashboard.guards[0].type).toBe('auth_required');
    expect(dashboard.transition).toBe('fade');
  });

  it('guardが正しく解決される', async () => {
    const interpreter = new NavigationInterpreter({
      mode: 'remote',
      remoteUrl: 'http://localhost/api/v1/navigation',
      componentRegistry: mockRegistry,
    });

    const result = await interpreter.build();

    // admin route has two guards
    const admin = result.routes[3];
    expect(admin.guards).toHaveLength(2);
    expect(admin.guards[0].id).toBe('auth_required');
    expect(admin.guards[0].redirect_to).toBe('/login');
    expect(admin.guards[1].id).toBe('admin_only');
    expect(admin.guards[1].type).toBe('role_required');
    expect(admin.guards[1].roles).toEqual(['admin']);
  });

  it('childrenが再帰的に解決される', async () => {
    const interpreter = new NavigationInterpreter({
      mode: 'remote',
      remoteUrl: 'http://localhost/api/v1/navigation',
      componentRegistry: mockRegistry,
    });

    const result = await interpreter.build();

    const admin = result.routes[3];
    expect(admin.children).toHaveLength(1);
    expect(admin.children[0].id).toBe('admin_users');
    expect(admin.children[0].guards).toHaveLength(2);
  });

  it('存在しないguard IDは無視される', async () => {
    server.use(
      http.get('http://localhost/api/v1/navigation', () => {
        return HttpResponse.json({
          guards: [],
          routes: [
            {
              id: 'test',
              path: '/test',
              component_id: 'LoginPage',
              guards: ['nonexistent_guard'],
            },
          ],
        });
      }),
    );

    const interpreter = new NavigationInterpreter({
      mode: 'remote',
      remoteUrl: 'http://localhost/api/v1/navigation',
      componentRegistry: mockRegistry,
    });

    const result = await interpreter.build();
    expect(result.routes[0].guards).toHaveLength(0);
  });

  it('local modeでJSONデータからrouterが構築される', async () => {
    server.use(
      http.get('http://localhost/navigation.json', () => {
        return new HttpResponse(JSON.stringify(mockNavigation), {
          headers: { 'Content-Type': 'application/json' },
        });
      }),
    );

    const interpreter = new NavigationInterpreter({
      mode: 'local',
      localConfigPath: 'http://localhost/navigation.json',
      componentRegistry: mockRegistry,
    });

    const result = await interpreter.build();
    expect(result.routes).toHaveLength(4);
    expect(result.guards).toHaveLength(2);
  });
});
