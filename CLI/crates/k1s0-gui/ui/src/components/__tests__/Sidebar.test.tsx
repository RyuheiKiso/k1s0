import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {
  createMemoryHistory,
  createRouter,
  createRootRoute,
  createRoute,
  RouterProvider,
  Outlet,
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

  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => <div>init-page</div>,
  });

  const generateRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/generate',
    component: () => <div>generate-page</div>,
  });

  const buildRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/build',
    component: () => <div>build-page</div>,
  });

  const testRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/test',
    component: () => <div>test-page</div>,
  });

  const deployRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/deploy',
    component: () => <div>deploy-page</div>,
  });

  const routeTree = rootRoute.addChildren([
    indexRoute,
    generateRoute,
    buildRoute,
    testRoute,
    deployRoute,
  ]);

  const history = createMemoryHistory({ initialEntries: [initialPath] });
  const router = createRouter({ routeTree, history });

  return { ...render(<RouterProvider router={router} />), router };
}

describe('Sidebar', () => {
  it('should render all menu items', async () => {
    renderWithRouter();
    expect(await screen.findByTestId('nav-init')).toBeInTheDocument();
    expect(screen.getByTestId('nav-generate')).toBeInTheDocument();
    expect(screen.getByTestId('nav-build')).toBeInTheDocument();
    expect(screen.getByTestId('nav-test')).toBeInTheDocument();
    expect(screen.getByTestId('nav-deploy')).toBeInTheDocument();
  });

  it('should navigate when a menu item is clicked', async () => {
    const user = userEvent.setup();
    const { router } = renderWithRouter();
    await screen.findByTestId('nav-generate');
    await user.click(screen.getByTestId('nav-generate'));
    expect(router.state.location.pathname).toBe('/generate');
  });

  it('should highlight the current page', async () => {
    renderWithRouter('/build');
    const buildNav = await screen.findByTestId('nav-build');
    expect(buildNav.className).toContain('bg-gray-700');
    expect(screen.getByTestId('nav-init').className).not.toContain('bg-gray-700');
  });
});
