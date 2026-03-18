import { lazy } from 'react';
import { createRootRoute, createRoute, createRouter } from '@tanstack/react-router';
import Layout from './components/Layout';

/// 各ページコンポーネントを遅延読み込みして初期バンドルサイズを削減する
const DashboardPage = lazy(() => import('./pages/DashboardPage'));
const AuthPage = lazy(() => import('./pages/AuthPage'));
const InitPage = lazy(() => import('./pages/InitPage'));
const GeneratePage = lazy(() => import('./pages/GeneratePage'));
const DepsPage = lazy(() => import('./pages/DepsPage'));
const DevPage = lazy(() => import('./pages/DevPage'));
const MigratePage = lazy(() => import('./pages/MigratePage'));
const TemplateMigratePage = lazy(() => import('./pages/TemplateMigratePage'));
const ConfigTypesPage = lazy(() => import('./pages/ConfigTypesPage'));
const NavigationTypesPage = lazy(() => import('./pages/NavigationTypesPage'));
const EventCodegenPage = lazy(() => import('./pages/EventCodegenPage'));
const ValidatePage = lazy(() => import('./pages/ValidatePage'));
const BuildPage = lazy(() => import('./pages/BuildPage'));
const TestPage = lazy(() => import('./pages/TestPage'));
const DeployPage = lazy(() => import('./pages/DeployPage'));

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

const templateMigrateRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/template-migrate',
  component: TemplateMigratePage,
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
  templateMigrateRoute,
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
