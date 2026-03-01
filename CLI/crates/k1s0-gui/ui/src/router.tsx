import { createRouter, createRootRoute, createRoute } from '@tanstack/react-router';
import Layout from './components/Layout';
import InitPage from './pages/InitPage';
import GeneratePage from './pages/GeneratePage';
import ConfigTypesPage from './pages/ConfigTypesPage';
import NavigationTypesPage from './pages/NavigationTypesPage';
import ValidatePage from './pages/ValidatePage';
import BuildPage from './pages/BuildPage';
import TestPage from './pages/TestPage';
import DeployPage from './pages/DeployPage';

const rootRoute = createRootRoute({
  component: Layout,
});

const initRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: InitPage,
});

const generateRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/generate',
  component: GeneratePage,
});

const configTypesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/config-types',
  component: ConfigTypesPage,
});

const navigationTypesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/navigation-types',
  component: NavigationTypesPage,
});

const validateRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/validate',
  component: ValidatePage,
});

const buildRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/build',
  component: BuildPage,
});

const testRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/test',
  component: TestPage,
});

const deployRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/deploy',
  component: DeployPage,
});

const routeTree = rootRoute.addChildren([
  initRoute,
  generateRoute,
  configTypesRoute,
  navigationTypesRoute,
  validateRoute,
  buildRoute,
  testRoute,
  deployRoute,
]);

export const router = createRouter({ routeTree });

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
