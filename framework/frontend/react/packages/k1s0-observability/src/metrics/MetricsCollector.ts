import type {
  PerformanceMetric,
  WebVitals,
  ObservabilityConfig,
} from '../types.js';

/**
 * メトリクスリスナー
 */
export type MetricsListener = (metric: PerformanceMetric) => void;

/**
 * メトリクス収集クラス
 *
 * - パフォーマンス計測
 * - Web Vitals 収集
 * - カスタムメトリクス
 */
export class MetricsCollector {
  private config: ObservabilityConfig;
  private metrics: PerformanceMetric[] = [];
  private listeners: Set<MetricsListener> = new Set();
  private webVitals: Partial<WebVitals> = {};

  constructor(config: ObservabilityConfig) {
    this.config = config;
  }

  /**
   * カウンターメトリクスを記録
   */
  counter(name: string, value: number = 1, tags?: Record<string, string>): void {
    this.record({
      name,
      value,
      unit: 'count',
      timestamp: Date.now(),
      tags,
    });
  }

  /**
   * 時間メトリクスを記録
   */
  timing(name: string, durationMs: number, tags?: Record<string, string>): void {
    this.record({
      name,
      value: durationMs,
      unit: 'ms',
      timestamp: Date.now(),
      tags,
    });
  }

  /**
   * サイズメトリクスを記録
   */
  size(name: string, bytes: number, tags?: Record<string, string>): void {
    this.record({
      name,
      value: bytes,
      unit: 'bytes',
      timestamp: Date.now(),
      tags,
    });
  }

  /**
   * パーセンテージメトリクスを記録
   */
  percentage(name: string, percent: number, tags?: Record<string, string>): void {
    this.record({
      name,
      value: Math.min(100, Math.max(0, percent)),
      unit: 'percent',
      timestamp: Date.now(),
      tags,
    });
  }

  /**
   * メトリクスを記録
   */
  record(metric: PerformanceMetric): void {
    this.metrics.push(metric);

    // リスナーに通知
    for (const listener of this.listeners) {
      try {
        listener(metric);
      } catch {
        // リスナーエラーは無視
      }
    }

    // バッファサイズチェック
    if (this.config.enableBatching && this.metrics.length >= this.config.batchSize) {
      this.flush();
    }
  }

  /**
   * 処理時間を計測
   */
  measureTime<T>(
    name: string,
    fn: () => T,
    tags?: Record<string, string>
  ): T {
    const start = performance.now();
    try {
      return fn();
    } finally {
      const duration = performance.now() - start;
      this.timing(name, duration, tags);
    }
  }

  /**
   * 非同期処理時間を計測
   */
  async measureTimeAsync<T>(
    name: string,
    fn: () => Promise<T>,
    tags?: Record<string, string>
  ): Promise<T> {
    const start = performance.now();
    try {
      return await fn();
    } finally {
      const duration = performance.now() - start;
      this.timing(name, duration, tags);
    }
  }

  /**
   * Web Vitals を記録
   */
  recordWebVital(name: keyof WebVitals, value: number): void {
    this.webVitals[name] = value;

    this.record({
      name: `web_vital.${name.toLowerCase()}`,
      value,
      unit: name === 'CLS' ? 'count' : 'ms',
      timestamp: Date.now(),
      tags: { vital_name: name },
    });
  }

  /**
   * Web Vitals を取得
   */
  getWebVitals(): Partial<WebVitals> {
    return { ...this.webVitals };
  }

  /**
   * Performance Observer で Web Vitals を自動収集
   */
  observeWebVitals(): () => void {
    if (typeof PerformanceObserver === 'undefined') {
      return () => {};
    }

    const observers: PerformanceObserver[] = [];

    try {
      // LCP (Largest Contentful Paint)
      const lcpObserver = new PerformanceObserver((list) => {
        const entries = list.getEntries();
        const lastEntry = entries[entries.length - 1];
        if (lastEntry) {
          this.recordWebVital('LCP', lastEntry.startTime);
        }
      });
      lcpObserver.observe({ type: 'largest-contentful-paint', buffered: true });
      observers.push(lcpObserver);
    } catch {
      // LCP not supported
    }

    try {
      // FID (First Input Delay)
      const fidObserver = new PerformanceObserver((list) => {
        const entries = list.getEntries();
        const firstEntry = entries[0] as PerformanceEventTiming | undefined;
        if (firstEntry) {
          this.recordWebVital('FID', firstEntry.processingStart - firstEntry.startTime);
        }
      });
      fidObserver.observe({ type: 'first-input', buffered: true });
      observers.push(fidObserver);
    } catch {
      // FID not supported
    }

    try {
      // CLS (Cumulative Layout Shift)
      let clsValue = 0;
      const clsObserver = new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          if (!(entry as LayoutShift).hadRecentInput) {
            clsValue += (entry as LayoutShift).value;
          }
        }
        this.recordWebVital('CLS', clsValue);
      });
      clsObserver.observe({ type: 'layout-shift', buffered: true });
      observers.push(clsObserver);
    } catch {
      // CLS not supported
    }

    try {
      // FCP (First Contentful Paint)
      const fcpObserver = new PerformanceObserver((list) => {
        const entries = list.getEntries();
        const fcpEntry = entries.find((e) => e.name === 'first-contentful-paint');
        if (fcpEntry) {
          this.recordWebVital('FCP', fcpEntry.startTime);
        }
      });
      fcpObserver.observe({ type: 'paint', buffered: true });
      observers.push(fcpObserver);
    } catch {
      // FCP not supported
    }

    // TTFB (Time to First Byte)
    try {
      const navEntry = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
      if (navEntry) {
        this.recordWebVital('TTFB', navEntry.responseStart - navEntry.requestStart);
      }
    } catch {
      // TTFB not supported
    }

    // クリーンアップ関数
    return () => {
      observers.forEach((observer) => observer.disconnect());
    };
  }

  /**
   * メトリクスリスナーを追加
   */
  onMetric(listener: MetricsListener): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  /**
   * バッファをフラッシュ
   */
  flush(): PerformanceMetric[] {
    const metrics = [...this.metrics];
    this.metrics = [];
    return metrics;
  }

  /**
   * リソースを解放
   */
  dispose(): void {
    this.listeners.clear();
    this.metrics = [];
    this.webVitals = {};
  }
}

/**
 * LayoutShift エントリの型定義
 */
interface LayoutShift extends PerformanceEntry {
  hadRecentInput: boolean;
  value: number;
}

/**
 * PerformanceEventTiming エントリの型定義
 */
interface PerformanceEventTiming extends PerformanceEntry {
  processingStart: number;
}
