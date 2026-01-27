/**
 * Logger のテスト
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { Logger, ConsoleLogSink, BufferedLogSink } from '../../src/logging/Logger';
import { TracingService } from '../../src/tracing/TracingService';
import type { ObservabilityConfig, LogEntry, LogLevel } from '../../src/types';

const createTestConfig = (): ObservabilityConfig => ({
  serviceName: 'test-service',
  env: 'dev',
  version: '1.0.0',
  samplingRate: 1.0,
  logLevel: 'DEBUG',
  enableConsole: false, // テストではコンソール出力を無効化
  enableBatching: false,
  batchSize: 10,
  batchIntervalMs: 5000,
});

describe('Logger', () => {
  let logger: Logger;

  beforeEach(() => {
    vi.clearAllMocks();
    logger = new Logger(createTestConfig());
  });

  afterEach(() => {
    logger.dispose();
  });

  describe('ログメソッド', () => {
    it('debug メソッドが機能すること', () => {
      const sink = { write: vi.fn() };
      logger.addSink(sink);

      logger.debug('Debug message', { extra: 'data' });

      expect(sink.write).toHaveBeenCalled();
      const entry = sink.write.mock.calls[0][0] as LogEntry;
      expect(entry.level).toBe('DEBUG');
      expect(entry.message).toBe('Debug message');
      expect(entry.extra).toBe('data');
    });

    it('info メソッドが機能すること', () => {
      const sink = { write: vi.fn() };
      logger.addSink(sink);

      logger.info('Info message');

      const entry = sink.write.mock.calls[0][0] as LogEntry;
      expect(entry.level).toBe('INFO');
    });

    it('warn メソッドが機能すること', () => {
      const sink = { write: vi.fn() };
      logger.addSink(sink);

      logger.warn('Warning message');

      const entry = sink.write.mock.calls[0][0] as LogEntry;
      expect(entry.level).toBe('WARN');
    });

    it('error メソッドが機能すること', () => {
      const sink = { write: vi.fn() };
      logger.addSink(sink);

      logger.error('Error message', new Error('Test error'));

      const entry = sink.write.mock.calls[0][0] as LogEntry;
      expect(entry.level).toBe('ERROR');
      expect(entry.error).toBeDefined();
      expect((entry.error as any).name).toBe('Error');
      expect((entry.error as any).message).toBe('Test error');
    });
  });

  describe('必須フィールド', () => {
    it('全ての必須フィールドが含まれること', () => {
      const sink = { write: vi.fn() };
      logger.addSink(sink);

      logger.info('Test message');

      const entry = sink.write.mock.calls[0][0] as LogEntry;
      expect(entry.timestamp).toBeDefined();
      expect(entry.level).toBeDefined();
      expect(entry.service_name).toBe('test-service');
      expect(entry.env).toBe('dev');
      expect(entry.trace_id).toBeDefined();
      expect(entry.span_id).toBeDefined();
    });
  });

  describe('TracingService との連携', () => {
    it('TracingService のコンテキストを使用すること', () => {
      const tracingService = new TracingService(createTestConfig());
      const loggerWithTracing = new Logger(createTestConfig(), tracingService);
      const sink = { write: vi.fn() };
      loggerWithTracing.addSink(sink);

      const context = tracingService.startContext('request-123');
      loggerWithTracing.info('Test message');

      const entry = sink.write.mock.calls[0][0] as LogEntry;
      expect(entry.trace_id).toBe(context.traceId);
      expect(entry.span_id).toBe(context.spanId);
      expect(entry.request_id).toBe('request-123');

      loggerWithTracing.dispose();
      tracingService.dispose();
    });
  });

  describe('ログレベルフィルタリング', () => {
    it('最小ログレベル以下のログは出力されないこと', () => {
      const config = { ...createTestConfig(), logLevel: 'WARN' as LogLevel };
      const filteredLogger = new Logger(config);
      const sink = { write: vi.fn() };
      filteredLogger.addSink(sink);

      filteredLogger.debug('Debug message');
      filteredLogger.info('Info message');
      filteredLogger.warn('Warn message');
      filteredLogger.error('Error message');

      expect(sink.write).toHaveBeenCalledTimes(2);
      expect(sink.write.mock.calls[0][0].level).toBe('WARN');
      expect(sink.write.mock.calls[1][0].level).toBe('ERROR');

      filteredLogger.dispose();
    });

    it('setMinLevel でログレベルを変更できること', () => {
      const sink = { write: vi.fn() };
      logger.addSink(sink);

      logger.setMinLevel('ERROR');
      logger.debug('Debug');
      logger.info('Info');
      logger.warn('Warn');
      logger.error('Error');

      expect(sink.write).toHaveBeenCalledTimes(1);
      expect(sink.write.mock.calls[0][0].level).toBe('ERROR');
    });
  });

  describe('シンク管理', () => {
    it('シンクを追加できること', () => {
      const sink = { write: vi.fn() };
      logger.addSink(sink);

      logger.info('Test');

      expect(sink.write).toHaveBeenCalled();
    });

    it('シンクを削除できること', () => {
      const sink = { write: vi.fn() };
      logger.addSink(sink);
      logger.removeSink(sink);

      logger.info('Test');

      expect(sink.write).not.toHaveBeenCalled();
    });

    it('複数のシンクに出力すること', () => {
      const sink1 = { write: vi.fn() };
      const sink2 = { write: vi.fn() };
      logger.addSink(sink1);
      logger.addSink(sink2);

      logger.info('Test');

      expect(sink1.write).toHaveBeenCalled();
      expect(sink2.write).toHaveBeenCalled();
    });
  });

  describe('エラーのシリアライズ', () => {
    it('Error オブジェクトを正しくシリアライズすること', () => {
      const sink = { write: vi.fn() };
      logger.addSink(sink);

      const error = new Error('Test error');
      error.name = 'TestError';
      logger.error('Error occurred', error);

      const entry = sink.write.mock.calls[0][0] as LogEntry;
      expect((entry.error as any).name).toBe('TestError');
      expect((entry.error as any).message).toBe('Test error');
      expect((entry.error as any).stack).toBeDefined();
    });

    it('cause チェーンをシリアライズすること', () => {
      const sink = { write: vi.fn() };
      logger.addSink(sink);

      const cause = new Error('Root cause');
      const error = new Error('Wrapper error', { cause });
      logger.error('Error occurred', error);

      const entry = sink.write.mock.calls[0][0] as LogEntry;
      expect((entry.error as any).cause).toBeDefined();
      expect((entry.error as any).cause.message).toBe('Root cause');
    });

    it('Error 以外の値もログに含めること', () => {
      const sink = { write: vi.fn() };
      logger.addSink(sink);

      logger.error('Error occurred', 'string error');

      const entry = sink.write.mock.calls[0][0] as LogEntry;
      expect((entry.error as any).message).toBe('string error');
    });
  });

  describe('flush', () => {
    it('全シンクをフラッシュすること', async () => {
      const sink = {
        write: vi.fn(),
        flush: vi.fn().mockResolvedValue(undefined),
      };
      logger.addSink(sink);

      await logger.flush();

      expect(sink.flush).toHaveBeenCalled();
    });
  });

  describe('dispose', () => {
    it('全シンクを dispose すること', () => {
      const sink = {
        write: vi.fn(),
        dispose: vi.fn(),
      };
      logger.addSink(sink);

      logger.dispose();

      expect(sink.dispose).toHaveBeenCalled();
    });
  });
});

describe('ConsoleLogSink', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('適切なコンソールメソッドを呼び出すこと', () => {
    const sink = new ConsoleLogSink(false);

    const createEntry = (level: LogLevel): LogEntry => ({
      timestamp: '2024-01-01T00:00:00.000Z',
      level,
      service_name: 'test',
      env: 'dev',
      trace_id: 'trace-123',
      span_id: 'span-123',
      message: `${level} message`,
    });

    sink.write(createEntry('DEBUG'));
    expect(console.debug).toHaveBeenCalled();

    sink.write(createEntry('INFO'));
    expect(console.info).toHaveBeenCalled();

    sink.write(createEntry('WARN'));
    expect(console.warn).toHaveBeenCalled();

    sink.write(createEntry('ERROR'));
    expect(console.error).toHaveBeenCalled();
  });

  it('構造化出力モードで JSON を出力すること', () => {
    const sink = new ConsoleLogSink(true);

    const entry: LogEntry = {
      timestamp: '2024-01-01T00:00:00.000Z',
      level: 'INFO',
      service_name: 'test',
      env: 'dev',
      trace_id: 'trace-123',
      span_id: 'span-123',
      message: 'Test message',
    };

    sink.write(entry);

    expect(console.info).toHaveBeenCalledWith(JSON.stringify(entry));
  });
});

describe('BufferedLogSink', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('バッファにログを追加すること', async () => {
    const onFlush = vi.fn().mockResolvedValue(undefined);
    const sink = new BufferedLogSink({ onFlush, maxSize: 10 });

    const entry: LogEntry = {
      timestamp: '2024-01-01T00:00:00.000Z',
      level: 'INFO',
      service_name: 'test',
      env: 'dev',
      trace_id: 'trace-123',
      span_id: 'span-123',
      message: 'Test message',
    };

    sink.write(entry);

    // まだフラッシュされていない
    expect(onFlush).not.toHaveBeenCalled();

    sink.dispose();
  });

  it('最大サイズに達するとフラッシュすること', async () => {
    const onFlush = vi.fn().mockResolvedValue(undefined);
    const sink = new BufferedLogSink({ onFlush, maxSize: 2 });

    const entry: LogEntry = {
      timestamp: '2024-01-01T00:00:00.000Z',
      level: 'INFO',
      service_name: 'test',
      env: 'dev',
      trace_id: 'trace-123',
      span_id: 'span-123',
      message: 'Test message',
    };

    sink.write(entry);
    sink.write(entry);

    await vi.runAllTimersAsync();

    expect(onFlush).toHaveBeenCalled();
    expect(onFlush.mock.calls[0][0]).toHaveLength(2);

    sink.dispose();
  });

  it('定期的にフラッシュすること', async () => {
    const onFlush = vi.fn().mockResolvedValue(undefined);
    const sink = new BufferedLogSink({
      onFlush,
      maxSize: 100,
      flushIntervalMs: 1000,
    });

    const entry: LogEntry = {
      timestamp: '2024-01-01T00:00:00.000Z',
      level: 'INFO',
      service_name: 'test',
      env: 'dev',
      trace_id: 'trace-123',
      span_id: 'span-123',
      message: 'Test message',
    };

    sink.write(entry);

    await vi.advanceTimersByTimeAsync(1000);

    expect(onFlush).toHaveBeenCalled();

    sink.dispose();
  });

  it('dispose 時に残りのログをフラッシュすること', async () => {
    const onFlush = vi.fn().mockResolvedValue(undefined);
    const sink = new BufferedLogSink({ onFlush, maxSize: 100 });

    const entry: LogEntry = {
      timestamp: '2024-01-01T00:00:00.000Z',
      level: 'INFO',
      service_name: 'test',
      env: 'dev',
      trace_id: 'trace-123',
      span_id: 'span-123',
      message: 'Test message',
    };

    sink.write(entry);
    sink.dispose();

    await vi.runAllTimersAsync();

    expect(onFlush).toHaveBeenCalled();
  });
});
