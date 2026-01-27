import type {
  LogLevel,
  LogEntry,
  LogSink,
  ObservabilityConfig,
  ErrorInfo,
} from '../types.js';
import { LOG_LEVEL_PRIORITY } from '../types.js';
import { generateTimestamp } from '../utils/ids.js';
import type { TracingService } from '../tracing/TracingService.js';

/**
 * コンソールログシンク
 */
export class ConsoleLogSink implements LogSink {
  private useStructuredOutput: boolean;

  constructor(useStructuredOutput: boolean = false) {
    this.useStructuredOutput = useStructuredOutput;
  }

  write(entry: LogEntry): void {
    if (this.useStructuredOutput) {
      // 構造化JSON出力
      const output = JSON.stringify(entry);

      switch (entry.level) {
        case 'DEBUG':
          console.debug(output);
          break;
        case 'INFO':
          console.info(output);
          break;
        case 'WARN':
          console.warn(output);
          break;
        case 'ERROR':
          console.error(output);
          break;
      }
    } else {
      // 人間可読形式
      const prefix = `[${entry.timestamp}] [${entry.level}] [${entry.trace_id}]`;
      const message = `${prefix} ${entry.message}`;

      switch (entry.level) {
        case 'DEBUG':
          console.debug(message, entry);
          break;
        case 'INFO':
          console.info(message, entry);
          break;
        case 'WARN':
          console.warn(message, entry);
          break;
        case 'ERROR':
          console.error(message, entry);
          break;
      }
    }
  }
}

/**
 * バッファリングログシンク
 */
export class BufferedLogSink implements LogSink {
  private buffer: LogEntry[] = [];
  private maxSize: number;
  private flushInterval: number;
  private flushTimer: ReturnType<typeof setInterval> | null = null;
  private onFlush: (entries: LogEntry[]) => Promise<void>;

  constructor(options: {
    maxSize?: number;
    flushIntervalMs?: number;
    onFlush: (entries: LogEntry[]) => Promise<void>;
  }) {
    this.maxSize = options.maxSize ?? 100;
    this.flushInterval = options.flushIntervalMs ?? 5000;
    this.onFlush = options.onFlush;

    // 定期フラッシュを開始
    this.startPeriodicFlush();
  }

  write(entry: LogEntry): void {
    this.buffer.push(entry);

    if (this.buffer.length >= this.maxSize) {
      this.flush();
    }
  }

  async flush(): Promise<void> {
    if (this.buffer.length === 0) return;

    const entries = [...this.buffer];
    this.buffer = [];

    try {
      await this.onFlush(entries);
    } catch (error) {
      // フラッシュ失敗時はコンソールに出力
      console.error('Failed to flush logs:', error);
      entries.forEach((entry) => {
        console.log(JSON.stringify(entry));
      });
    }
  }

  dispose(): void {
    if (this.flushTimer) {
      clearInterval(this.flushTimer);
      this.flushTimer = null;
    }
    // 残りのログをフラッシュ
    this.flush();
  }

  private startPeriodicFlush(): void {
    this.flushTimer = setInterval(() => {
      this.flush();
    }, this.flushInterval);
  }
}

/**
 * ロガークラス
 *
 * - 構造化ログ出力
 * - 必須フィールドの自動付与（timestamp, level, service_name, env, trace_id, span_id）
 * - 複数シンクのサポート
 * - ログレベルによるフィルタリング
 */
export class Logger {
  private config: ObservabilityConfig;
  private tracingService: TracingService | null;
  private sinks: LogSink[] = [];
  private minLevel: LogLevel;

  constructor(
    config: ObservabilityConfig,
    tracingService?: TracingService
  ) {
    this.config = config;
    this.tracingService = tracingService ?? null;
    this.minLevel = config.logLevel;

    // デフォルトでコンソールシンクを追加
    if (config.enableConsole) {
      this.addSink(new ConsoleLogSink(config.env === 'prod'));
    }
  }

  /**
   * ログシンクを追加
   */
  addSink(sink: LogSink): void {
    this.sinks.push(sink);
  }

  /**
   * ログシンクを削除
   */
  removeSink(sink: LogSink): void {
    const index = this.sinks.indexOf(sink);
    if (index !== -1) {
      this.sinks.splice(index, 1);
    }
  }

  /**
   * 最小ログレベルを設定
   */
  setMinLevel(level: LogLevel): void {
    this.minLevel = level;
  }

  /**
   * DEBUG レベルのログを出力
   */
  debug(message: string, fields?: Record<string, unknown>): void {
    this.log('DEBUG', message, fields);
  }

  /**
   * INFO レベルのログを出力
   */
  info(message: string, fields?: Record<string, unknown>): void {
    this.log('INFO', message, fields);
  }

  /**
   * WARN レベルのログを出力
   */
  warn(message: string, fields?: Record<string, unknown>): void {
    this.log('WARN', message, fields);
  }

  /**
   * ERROR レベルのログを出力
   */
  error(message: string, error?: Error | unknown, fields?: Record<string, unknown>): void {
    const errorFields: Record<string, unknown> = { ...fields };

    if (error instanceof Error) {
      errorFields.error = this.serializeError(error);
    } else if (error !== undefined) {
      errorFields.error = { message: String(error) };
    }

    this.log('ERROR', message, errorFields);
  }

  /**
   * ログを出力
   */
  log(level: LogLevel, message: string, fields?: Record<string, unknown>): void {
    // レベルフィルタリング
    if (LOG_LEVEL_PRIORITY[level] < LOG_LEVEL_PRIORITY[this.minLevel]) {
      return;
    }

    const context = this.tracingService?.getCurrentContext();

    const entry: LogEntry = {
      // 必須フィールド
      timestamp: generateTimestamp(),
      level,
      service_name: this.config.serviceName,
      env: this.config.env,
      trace_id: context?.traceId ?? 'no-trace',
      span_id: context?.spanId ?? 'no-span',
      // メッセージ
      message,
      // リクエストID（存在する場合）
      ...(context?.requestId && { request_id: context.requestId }),
      // 追加フィールド
      ...fields,
    };

    // 全シンクに出力
    for (const sink of this.sinks) {
      try {
        sink.write(entry);
      } catch {
        // シンクエラーは無視
      }
    }
  }

  /**
   * 全シンクをフラッシュ
   */
  async flush(): Promise<void> {
    await Promise.all(
      this.sinks.map(async (sink) => {
        try {
          await sink.flush?.();
        } catch {
          // フラッシュエラーは無視
        }
      })
    );
  }

  /**
   * リソースを解放
   */
  dispose(): void {
    for (const sink of this.sinks) {
      try {
        sink.dispose?.();
      } catch {
        // 解放エラーは無視
      }
    }
    this.sinks = [];
  }

  /**
   * エラーをシリアライズ
   */
  private serializeError(error: Error): ErrorInfo {
    const info: ErrorInfo = {
      name: error.name,
      message: error.message,
      stack: error.stack,
    };

    // カスタムエラーコード
    if ('code' in error && typeof error.code === 'string') {
      info.code = error.code;
    }

    // cause チェーン
    if (error.cause instanceof Error) {
      info.cause = this.serializeError(error.cause);
    }

    return info;
  }
}
