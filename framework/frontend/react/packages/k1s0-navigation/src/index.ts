/**
 * @k1s0/navigation - 設定駆動ナビゲーションライブラリ
 *
 * config/{env}.yaml の ui.navigation セクションから
 * routes/menu/flows を読み込み、React Router へ反映する。
 *
 * @example
 * ```tsx
 * import {
 *   NavigationProvider,
 *   ConfigRouter,
 *   MenuBuilder,
 * } from '@k1s0/navigation';
 *
 * const navigationConfig = await loadConfig();
 * const screens = [
 *   { id: 'home', component: HomePage },
 *   { id: 'users.list', component: UsersListPage },
 * ];
 *
 * function App() {
 *   return (
 *     <NavigationProvider
 *       config={navigationConfig}
 *       screens={screens}
 *       auth={{ permissions: ['user:read'], flags: [] }}
 *     >
 *       <ConfigRouter />
 *     </NavigationProvider>
 *   );
 * }
 * ```
 */

// Schema
export * from './schema';

// Router
export * from './router';

// Menu
export * from './menu';

// Flows
export * from './flows';

// Guards
export * from './guards';
