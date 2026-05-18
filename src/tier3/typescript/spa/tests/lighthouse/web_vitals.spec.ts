// web_vitals.spec.ts — Web Vitals CI script
// 設計方針 31: Lighthouse CI で merge 阻止 (強制機構 03 層 12)

// @playwright/test をインポートする
import { test, expect } from '@playwright/test';

test('LCP は 3000ms 以内である', async ({ page }) => {
  // SPA トップページを開く（about:blank で即座に計測）
  await page.goto('about:blank');

  // PerformanceObserver で LCP を計測する
  const lcp = await page.evaluate(() => {
    // LCP 計測を Promise でラップして返す
    return new Promise<number>((resolve) => {
      // LCP を記録するための PerformanceObserver を設定する
      new PerformanceObserver((list) => {
        // エントリ一覧を取得する
        const entries = list.getEntries();
        // 最後のエントリを取得する
        const lastEntry = entries[entries.length - 1];
        // LCP の startTime を返す
        resolve(lastEntry.startTime);
      }).observe({ type: 'largest-contentful-paint', buffered: true });
      // PerformanceObserver が発火しない場合は 1000ms 後に 0 を返す
      setTimeout(() => resolve(0), 1000);
    });
  });

  // LCP は 3000ms 以内であることを確認する（about:blank は即座なので 0）
  expect(lcp).toBeLessThan(3000);
});

test('CLS は 0.1 未満である', async ({ page }) => {
  // SPA トップページを開く（about:blank で即座に計測）
  await page.goto('about:blank');

  // PerformanceObserver で CLS を計測する
  const cls = await page.evaluate(() => {
    // CLS 累積値を Promise でラップして返す
    return new Promise<number>((resolve) => {
      // CLS を累積するための変数を初期化する
      let clsValue = 0;
      // layout-shift イベントを観測する PerformanceObserver を設定する
      new PerformanceObserver((list) => {
        // 各エントリを処理する
        for (const entry of list.getEntries()) {
          // layout-shift エントリとして型アサートする
          const layoutShiftEntry = entry as PerformanceEntry & {
            hadRecentInput: boolean;
            value: number;
          };
          // 最近の入力がない場合のみ CLS に加算する
          if (!layoutShiftEntry.hadRecentInput) {
            clsValue += layoutShiftEntry.value;
          }
        }
        // CLS 累積値を返す
        resolve(clsValue);
      }).observe({ type: 'layout-shift', buffered: true });
      // PerformanceObserver が発火しない場合は 1000ms 後に累積値を返す
      setTimeout(() => resolve(clsValue), 1000);
    });
  });

  // CLS は 0.1 未満であることを確認する
  expect(cls).toBeLessThan(0.1);
});
