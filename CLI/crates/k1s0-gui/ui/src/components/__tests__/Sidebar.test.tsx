import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {
  Outlet,
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
} from '@tanstack/react-router';
import Sidebar from '../Sidebar';

function renderWithRouter(initialPath = '/') {
  const rootRoute = createRootRoute({
    component: () => (
      <div>
        <Sidebar />
        <Outlet />
      </div>
    ),
  });

  const routeDefinitions = [
    ['/', 'dashboard-page'],
    ['/auth', 'auth-page'],
    ['/init', 'init-page'],
    ['/generate', 'generate-page'],
    ['/deps', 'deps-page'],
    ['/dev', 'dev-page'],
    ['/migrate', 'migrate-page'],
    ['/template-migrate', 'template-migrate-page'],
    ['/config-types', 'config-types-page'],
    ['/navigation-types', 'navigation-types-page'],
    ['/event-codegen', 'event-codegen-page'],
    ['/validate', 'validate-page'],
    ['/build', 'build-page'],
    ['/test', 'test-page'],
    ['/deploy', 'deploy-page'],
  ] as const;

  const routes = routeDefinitions.map(([path, text]) =>
    createRoute({
      getParentRoute: () => rootRoute,
      path,
      component: () => <div>{text}</div>,
    }),
  );

  const history = createMemoryHistory({ initialEntries: [initialPath] });
  const router = createRouter({ routeTree: rootRoute.addChildren(routes), history });

  return { ...render(<RouterProvider router={router} />), router };
}

describe('Sidebar', () => {
  it('renders the primary navigation items', async () => {
    renderWithRouter();
    expect(await screen.findByTestId('nav-dashboard')).toBeInTheDocument();
    expect(screen.getByTestId('nav-auth')).toBeInTheDocument();
    expect(screen.getByTestId('nav-init')).toBeInTheDocument();
    expect(screen.getByTestId('nav-generate')).toBeInTheDocument();
    expect(screen.getByTestId('nav-deps')).toBeInTheDocument();
    expect(screen.getByTestId('nav-dev')).toBeInTheDocument();
    expect(screen.getByTestId('nav-migrate')).toBeInTheDocument();
    expect(screen.getByTestId('nav-template-migrate')).toBeInTheDocument();
    expect(screen.getByTestId('nav-config-types')).toBeInTheDocument();
    expect(screen.getByTestId('nav-navigation-types')).toBeInTheDocument();
    expect(screen.getByTestId('nav-event-codegen')).toBeInTheDocument();
    expect(screen.getByTestId('nav-validate')).toBeInTheDocument();
    expect(screen.getByTestId('nav-build')).toBeInTheDocument();
    expect(screen.getByTestId('nav-test')).toBeInTheDocument();
    expect(screen.getByTestId('nav-deploy')).toBeInTheDocument();
  });

  it('navigates to the selected route', async () => {
    const user = userEvent.setup();
    const { router } = renderWithRouter('/');

    await user.click(await screen.findByTestId('nav-generate'));

    expect(router.state.location.pathname).toBe('/generate');
  });

  it('marks the active route', async () => {
    renderWithRouter('/build');
    const buildNav = await screen.findByTestId('nav-build');
    expect(buildNav.className).toContain('bg-[rgba(0,200,255,0.10)]');
    expect(screen.getByTestId('nav-dashboard').className).not.toContain('bg-[rgba(0,200,255,0.10)]');
  });
});
