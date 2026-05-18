// service-worker.ts — PWA Service Worker（cache-first read-through cache）
// spec 11 §PWA + offline-first: 06_端末オフラインデバイス の Service Worker を物理化する
// Workbox v7 の precacheAndRoute + registerRoute で SPA の offline 動作を保証する

// Workbox の precaching モジュールをインポートする
import { precacheAndRoute } from 'workbox-precaching';
// Workbox の routing モジュールをインポートする
import { registerRoute } from 'workbox-routing';
// Workbox の cache-first 戦略をインポートする
import { CacheFirst, NetworkFirst } from 'workbox-strategies';
// Workbox の expiration plugin をインポートする
import { ExpirationPlugin } from 'workbox-expiration';
// Workbox の plugin 基底型をインポートする（exactOptionalPropertyTypes 互換 cast 用）
import type { WorkboxPlugin } from 'workbox-core';

// TypeScript の Service Worker のグローバル型宣言を拡張する
declare const self: ServiceWorkerGlobalScope;

// Vite PWA plugin が挿入する precache manifest を使用する（__WB_MANIFEST は build 時に置換される）
precacheAndRoute(self.__WB_MANIFEST || []);

// API リクエストは NetworkFirst（online 優先、offline fallback）で処理する
registerRoute(
  // /api/ パスへのリクエストを対象にする
  ({ url }) => url.pathname.startsWith('/api/'),
  // NetworkFirst 戦略: online の場合は network を優先し、offline は cache を使用する
  new NetworkFirst({
    // キャッシュ名を設定する
    cacheName: 'api-cache',
    // plugin を設定する（ExpirationPlugin を WorkboxPlugin として明示 cast する）
    plugins: [
      // 最大 50 件、最大 1 日間キャッシュを保持する（exactOptionalPropertyTypes 互換）
      new ExpirationPlugin({ maxEntries: 50, maxAgeSeconds: 86400 }) as WorkboxPlugin,
    ],
  })
);

// 静的アセット（画像・フォント）は CacheFirst で処理する
registerRoute(
  // 静的アセットの Content-Type を対象にする
  ({ request }) =>
    request.destination === 'image' || request.destination === 'font',
  // CacheFirst 戦略: cache に存在する場合は cache を優先する
  new CacheFirst({
    // キャッシュ名を設定する
    cacheName: 'static-assets',
    // plugin を設定する（ExpirationPlugin を WorkboxPlugin として明示 cast する）
    plugins: [
      // 最大 100 件、最大 30 日間キャッシュを保持する（exactOptionalPropertyTypes 互換）
      new ExpirationPlugin({ maxEntries: 100, maxAgeSeconds: 2592000 }) as WorkboxPlugin,
    ],
  })
);
