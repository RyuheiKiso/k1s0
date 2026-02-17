import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@opentelemetry/api', () => {
  const mockSpan = {
    setAttribute: vi.fn(),
    setStatus: vi.fn(),
    end: vi.fn(),
  };
  const mockTracer = {
    startSpan: vi.fn().mockReturnValue(mockSpan),
  };
  return {
    trace: {
      getTracer: vi.fn().mockReturnValue(mockTracer),
    },
    SpanStatusCode: {
      OK: 0,
      ERROR: 2,
    },
    context: {
      active: vi.fn().mockReturnValue({}),
    },
  };
});

import { trace, SpanStatusCode } from '@opentelemetry/api';
import { createGrpcInterceptor } from '../src/grpcInterceptor';
import type { Metrics } from '../src/metrics';

function createMockMetrics(): Metrics {
  return {
    serviceName: 'test-service',
    recordHTTPRequest: vi.fn(),
    recordHTTPDuration: vi.fn(),
    recordGRPCRequest: vi.fn(),
    recordGRPCDuration: vi.fn(),
    getMetrics: vi.fn().mockReturnValue(''),
  } as unknown as Metrics;
}

describe('gRPC Interceptor', () => {
  let mockMetrics: Metrics;

  beforeEach(() => {
    vi.clearAllMocks();
    mockMetrics = createMockMetrics();
  });

  describe('createGrpcInterceptor', () => {
    it('interceptor 関数を返す', () => {
      const interceptor = createGrpcInterceptor(mockMetrics);
      expect(typeof interceptor).toBe('function');
    });

    it('正常リクエストをトレースする', async () => {
      const interceptor = createGrpcInterceptor(mockMetrics);
      const mockResult = { value: 'test' };

      const result = await interceptor(
        '/test.UserService/GetUser',
        { userId: '123' },
        async () => mockResult,
      );

      expect(result).toBe(mockResult);

      const tracer = trace.getTracer('k1s0-grpc');
      expect(tracer.startSpan).toHaveBeenCalledWith(
        '/test.UserService/GetUser',
        expect.any(Object),
      );

      const span = tracer.startSpan('');
      expect(span.setStatus).toHaveBeenCalledWith({
        code: SpanStatusCode.OK,
      });
      expect(span.end).toHaveBeenCalled();
    });

    it('gRPC メトリクスを記録する', async () => {
      const interceptor = createGrpcInterceptor(mockMetrics);

      await interceptor(
        '/test.UserService/GetUser',
        {},
        async () => ({}),
      );

      expect(mockMetrics.recordGRPCRequest).toHaveBeenCalledWith(
        'test.UserService',
        'GetUser',
        'OK',
      );
      expect(mockMetrics.recordGRPCDuration).toHaveBeenCalledWith(
        'test.UserService',
        'GetUser',
        expect.any(Number),
      );
    });

    it('エラーをハンドリングしてステータスを記録する', async () => {
      const interceptor = createGrpcInterceptor(mockMetrics);
      const error = new Error('connection refused');

      await expect(
        interceptor(
          '/test.OrderService/CreateOrder',
          {},
          async () => { throw error; },
        ),
      ).rejects.toThrow('connection refused');

      const tracer = trace.getTracer('k1s0-grpc');
      const span = tracer.startSpan('');
      expect(span.setStatus).toHaveBeenCalledWith({
        code: SpanStatusCode.ERROR,
        message: 'connection refused',
      });
      expect(span.end).toHaveBeenCalled();

      expect(mockMetrics.recordGRPCRequest).toHaveBeenCalledWith(
        'test.OrderService',
        'CreateOrder',
        'ERROR',
      );
    });

    it('メソッドパスからサービス名とメソッド名を抽出する', async () => {
      const interceptor = createGrpcInterceptor(mockMetrics);

      await interceptor(
        '/my.package.MyService/DoSomething',
        {},
        async () => ({}),
      );

      expect(mockMetrics.recordGRPCRequest).toHaveBeenCalledWith(
        'my.package.MyService',
        'DoSomething',
        'OK',
      );
    });

    it('不正なメソッドパスでもフォールバックする', async () => {
      const interceptor = createGrpcInterceptor(mockMetrics);

      await interceptor(
        'InvalidPath',
        {},
        async () => ({}),
      );

      expect(mockMetrics.recordGRPCRequest).toHaveBeenCalledWith(
        'unknown',
        'InvalidPath',
        'OK',
      );
    });
  });
});
