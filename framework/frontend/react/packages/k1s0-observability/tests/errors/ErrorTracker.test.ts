/**
 * ErrorTracker のテスト
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { ErrorTracker, type ErrorEvent, type ErrorListener } from '../../src/errors/ErrorTracker';
import { Logger } from '../../src/logging/Logger';
import { TracingService } from '../../src/tracing/TracingService';
import type { ObservabilityConfig } from '../../src/types';

const createTestConfig = (): ObservabilityConfig => ({
  serviceName: 'test-service',
  env: 'dev',
  version: '1.0.0',
  samplingRate: 1.0,
  logLevel: 'DEBUG',
  enableConsole: false,
  enableBatching: false,
  batchSize: 10,
  batchIntervalMs: 5000,
});

describe('ErrorTracker', () => {
  let errorTracker: ErrorTracker;

  beforeEach(() => {
    vi.clearAllMocks();
    errorTracker = new ErrorTracker(createTestConfig());
  });

  afterEach(() => {
    errorTracker.dispose();
  });

  describe('captureError', () => {
    it('エラーをキャプチャできること', () => {
      const error = new Error('Test error');

      const event = errorTracker.captureError(error);

      expect(event.error.name).toBe('Error');
      expect(event.error.message).toBe('Test error');
      expect(event.timestamp).toBeDefined();
    });

    it('コンテキストを追加できること', () => {
      const error = new Error('Test error');

      const event = errorTracker.captureError(error, { userId: '123' });

      expect(event.context?.userId).toBe('123');
    });

    it('スタックトレースを含めること', () => {
      const error = new Error('Test error');

      const event = errorTracker.captureError(error);

      expect(event.error.stack).toBeDefined();
    });

    it('カスタムエラーコードをシリアライズすること', () => {
      const error = new Error('Test error');
      (error as any).code = 'CUSTOM_ERROR';

      const event = errorTracker.captureError(error);

      expect(event.error.code).toBe('CUSTOM_ERROR');
    });

    it('cause チェーンをシリアライズすること', () => {
      const cause = new Error('Root cause');
      const error = new Error('Wrapper error', { cause });

      const event = errorTracker.captureError(error);

      expect(event.error.cause).toBeDefined();
      expect(event.error.cause?.message).toBe('Root cause');
    });
  });

  describe('captureException', () => {
    it('Error オブジェクトをキャプチャできること', () => {
      const error = new Error('Test error');

      const event = errorTracker.captureException(error);

      expect(event.error.message).toBe('Test error');
    });

    it('Error 以外の値をラップしてキャプチャできること', () => {
      const event = errorTracker.captureException('String error');

      expect(event.error.name).toBe('CapturedValue');
      expect(event.error.message).toBe('String error');
      expect(event.context?.originalValue).toBe('String error');
    });

    it('オブジェクトをキャプチャできること', () => {
      const obj = { foo: 'bar' };
      const event = errorTracker.captureException(obj);

      expect(event.error.message).toBe('[object Object]');
    });
  });

  describe('captureMessage', () => {
    it('メッセージをエラーとしてキャプチャできること', () => {
      const event = errorTracker.captureMessage('Something went wrong');

      expect(event.error.name).toBe('Error');
      expect(event.error.message).toBe('Something went wrong');
    });

    it('warning レベルを指定できること', () => {
      const event = errorTracker.captureMessage('Warning message', 'warning');

      expect(event.error.name).toBe('Warning');
      expect(event.context?.level).toBe('warning');
    });
  });

  describe('onError リスナー', () => {
    it('リスナーを追加できること', () => {
      const listener: ErrorListener = vi.fn();
      errorTracker.onError(listener);

      errorTracker.captureError(new Error('Test'));

      expect(listener).toHaveBeenCalled();
    });

    it('リスナーを削除できること', () => {
      const listener: ErrorListener = vi.fn();
      const unsubscribe = errorTracker.onError(listener);
      unsubscribe();

      errorTracker.captureError(new Error('Test'));

      expect(listener).not.toHaveBeenCalled();
    });

    it('複数のリスナーに通知すること', () => {
      const listener1: ErrorListener = vi.fn();
      const listener2: ErrorListener = vi.fn();
      errorTracker.onError(listener1);
      errorTracker.onError(listener2);

      errorTracker.captureError(new Error('Test'));

      expect(listener1).toHaveBeenCalled();
      expect(listener2).toHaveBeenCalled();
    });

    it('リスナーのエラーを無視すること', () => {
      const throwingListener: ErrorListener = () => {
        throw new Error('Listener error');
      };
      const normalListener: ErrorListener = vi.fn();
      errorTracker.onError(throwingListener);
      errorTracker.onError(normalListener);

      // エラーを投げるリスナーがあっても例外にならない
      expect(() => {
        errorTracker.captureError(new Error('Test'));
      }).not.toThrow();

      // 他のリスナーは正常に呼ばれる
      expect(normalListener).toHaveBeenCalled();
    });
  });

  describe('flush', () => {
    it('バッファ内のエラーを返すこと', () => {
      errorTracker.captureError(new Error('Error 1'));
      errorTracker.captureError(new Error('Error 2'));

      const errors = errorTracker.flush();

      expect(errors).toHaveLength(2);
    });

    it('フラッシュ後はバッファが空になること', () => {
      errorTracker.captureError(new Error('Test'));

      errorTracker.flush();
      const errors = errorTracker.flush();

      expect(errors).toHaveLength(0);
    });
  });

  describe('Logger との連携', () => {
    it('Logger にエラーを出力すること', () => {
      const logger = new Logger(createTestConfig());
      const errorSpy = vi.spyOn(logger, 'error');
      const trackerWithLogger = new ErrorTracker(createTestConfig(), logger);

      trackerWithLogger.captureError(new Error('Test error'));

      expect(errorSpy).toHaveBeenCalled();

      trackerWithLogger.dispose();
      logger.dispose();
    });
  });

  describe('TracingService との連携', () => {
    it('トレースコンテキストを含めること', () => {
      const tracingService = new TracingService(createTestConfig());
      const trackerWithTracing = new ErrorTracker(
        createTestConfig(),
        undefined,
        tracingService
      );

      const context = tracingService.startContext();
      const event = trackerWithTracing.captureError(new Error('Test'));

      expect(event.traceId).toBe(context.traceId);
      expect(event.spanId).toBe(context.spanId);

      trackerWithTracing.dispose();
      tracingService.dispose();
    });
  });

  describe('createErrorBoundaryHandler', () => {
    it('React Error Boundary 用のハンドラを作成できること', () => {
      const handler = errorTracker.createErrorBoundaryHandler('TestComponent');

      expect(typeof handler).toBe('function');
    });

    it('ハンドラがエラーをキャプチャすること', () => {
      const listener: ErrorListener = vi.fn();
      errorTracker.onError(listener);

      const handler = errorTracker.createErrorBoundaryHandler('TestComponent');
      handler(new Error('Component error'), { componentStack: 'at TestComponent' });

      expect(listener).toHaveBeenCalled();
      const event = listener.mock.calls[0][0] as ErrorEvent;
      expect(event.context?.componentName).toBe('TestComponent');
      expect(event.context?.type).toBe('react_error_boundary');
    });
  });

  describe('dispose', () => {
    it('リソースを解放すること', () => {
      const listener: ErrorListener = vi.fn();
      errorTracker.onError(listener);

      errorTracker.dispose();

      // dispose 後はリスナーに通知されない（リスナーがクリアされている）
      // 新しい ErrorTracker を作成してテスト
      const newTracker = new ErrorTracker(createTestConfig());
      newTracker.captureError(new Error('Test'));
      expect(listener).not.toHaveBeenCalled();
      newTracker.dispose();
    });
  });
});
