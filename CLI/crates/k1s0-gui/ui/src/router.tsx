import { createRootRoute, createRoute, createRouter } from '@tanstack/react-router';
import Layout from './components/Layout';
import DashboardPage from './pages/DashboardPage';
import AuthPage from './pages/AuthPage';
import InitPage from './pages/InitPage';
import GeneratePage from './pages/GeneratePage';
import DepsPage from './pages/DepsPage';
import DevPage from './pages/DevPage';
import MigratePage from './pages/MigratePage';
import ConfigTypesPage from './pages/ConfigTypesPage';
import NavigationTypesPage from './pages/NavigationTypesPage';
import EventCodegenPage from './pages/EventCodegenPage';
import ValidatePage from './pages/ValidatePage';
import BuildPage from './pages/BuildPage';
import TestPage from './pages/TestPage';
import DeployPage from './pages/DeployPage';

const rootRoute = createRootRoute({
  component: Layout,
});

const dashboardRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: DashboardPage,
});

const authRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/auth',
  component: AuthPage,
});

const initRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/init',
  component: InitPage,
});

const generateRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/generate',
  component: GeneratePage,
});

const depsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/deps',
  component: DepsPage,
});

const devRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/dev',
  component: DevPage,
});

const migrateRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/migrate',
  component: MigratePage,
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

const eventCodegenRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/event-codegen',
  component: EventCodegenPage,
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
  dashboardRoute,
  authRoute,
  initRoute,
  generateRoute,
  depsRoute,
  devRoute,
  migrateRoute,
  configTypesRoute,
  navigationTypesRoute,
  eventCodegenRoute,
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
