import { describe, it, expect, vi, beforeEach } from 'vitest';

// @opentelemetry/sdk-node と @opentelemetry/exporter-trace-otlp-grpc は
// テスト環境では外部依存となるためモックする。
vi.mock('@opentelemetry/sdk-node', () => {
  const mockShutdown = vi.fn().mockResolvedValue(undefined);
  const mockStart = vi.fn();
  return {
    NodeSDK: vi.fn().mockImplementation(() => ({
      start: mockStart,
      shutdown: mockShutdown,
    })),
  };
});

vi.mock('@opentelemetry/exporter-trace-otlp-grpc', () => ({
  OTLPTraceExporter: vi.fn().mockImplementation(() => ({})),
}));

vi.mock('@opentelemetry/api', () => ({
  trace: {
    getActiveSpan: vi.fn().mockReturnValue(undefined),
  },
}));

vi.mock('pino', () => {
  const mockLogger = {
    info: vi.fn(),
    warn: vi.fn(),
    error: vi.fn(),
    debug: vi.fn(),
    level: 'info',
  };
  return {
    default: vi.fn().mockReturnValue(mockLogger),
  };
});

import { initTelemetry, shutdown, type TelemetryConfig } from '../src/telemetry';
import { createLogger } from '../src/logger';
import { httpMiddleware } from '../src/middleware';
import { NodeSDK } from '@opentelemetry/sdk-node';
import pino from 'pino';

describe('telemetry', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('initTelemetry', () => {
    it('should not create SDK when traceEndpoint is not provided', () => {
      const cfg: TelemetryConfig = {
        serviceName: 'test-service',
        version: '1.0.0',
        tier: 'system',
        environment: 'dev',
        logLevel: 'debug',
      };

      initTelemetry(cfg);
      expect(NodeSDK).not.toHaveBeenCalled();
    });

    it('should create SDK when traceEndpoint is provided', () => {
      const cfg: TelemetryConfig = {
        serviceName: 'test-service',
        version: '1.0.0',
        tier: 'system',
        environment: 'dev',
        traceEndpoint: 'http://localhost:4317',
        sampleRate: 1.0,
        logLevel: 'info',
      };

      initTelemetry(cfg);
      expect(NodeSDK).toHaveBeenCalled();
    });
  });

  describe('shutdown', () => {
    it('should resolve when no SDK is initialized', async () => {
      await expect(shutdown()).resolves.toBeUndefined();
    });
  });

  describe('createLogger', () => {
    it('should create a pino logger with correct config', () => {
      const cfg: TelemetryConfig = {
        serviceName: 'test-service',
        version: '1.0.0',
        tier: 'system',
        environment: 'dev',
        logLevel: 'debug',
      };

      const logger = createLogger(cfg);
      expect(pino).toHaveBeenCalledWith(
        expect.objectContaining({
          level: 'debug',
          base: {
            service: 'test-service',
            version: '1.0.0',
            tier: 'system',
            environment: 'dev',
          },
          mixin: expect.any(Function),
        }),
      );
      expect(logger).toBeDefined();
    });

    it('should set info log level for staging', () => {
      const cfg: TelemetryConfig = {
        serviceName: 'staging-service',
        version: '2.0.0',
        tier: 'business',
        environment: 'staging',
        logLevel: 'info',
      };

      createLogger(cfg);
      expect(pino).toHaveBeenCalledWith(
        expect.objectContaining({
          level: 'info',
          base: expect.objectContaining({
            service: 'staging-service',
            environment: 'staging',
          }),
        }),
      );
    });

    it('should set warn log level for prod', () => {
      const cfg: TelemetryConfig = {
        serviceName: 'prod-service',
        version: '3.0.0',
        tier: 'service',
        environment: 'prod',
        logLevel: 'warn',
      };

      createLogger(cfg);
      expect(pino).toHaveBeenCalledWith(
        expect.objectContaining({
          level: 'warn',
          base: expect.objectContaining({
            service: 'prod-service',
            environment: 'prod',
          }),
        }),
      );
    });
  });

  describe('httpMiddleware', () => {
    it('should return a middleware function', () => {
      const cfg: TelemetryConfig = {
        serviceName: 'test-service',
        version: '1.0.0',
        tier: 'system',
        environment: 'dev',
        logLevel: 'debug',
      };

      const logger = createLogger(cfg);
      const middleware = httpMiddleware(logger);
      expect(typeof middleware).toBe('function');
    });

    it('should call next function', () => {
      const cfg: TelemetryConfig = {
        serviceName: 'test-service',
        version: '1.0.0',
        tier: 'system',
        environment: 'dev',
        logLevel: 'debug',
      };

      const logger = createLogger(cfg);
      const middleware = httpMiddleware(logger);
      const next = vi.fn();
      const req = { method: 'GET', url: '/health' } as any;
      const res = {
        statusCode: 200,
        on: vi.fn(),
      } as any;

      middleware(req, res, next);
      expect(next).toHaveBeenCalled();
    });

    it('should log on response finish', () => {
      const cfg: TelemetryConfig = {
        serviceName: 'test-service',
        version: '1.0.0',
        tier: 'system',
        environment: 'dev',
        logLevel: 'debug',
      };

      const logger = createLogger(cfg);
      const middleware = httpMiddleware(logger);
      const next = vi.fn();
      const req = { method: 'POST', url: '/api/v1/orders' } as any;
      let finishCallback: (() => void) | undefined;
      const res = {
        statusCode: 201,
        on: vi.fn((event: string, cb: () => void) => {
          if (event === 'finish') {
            finishCallback = cb;
          }
        }),
      } as any;

      middleware(req, res, next);
      expect(finishCallback).toBeDefined();

      finishCallback!();
      expect(logger.info).toHaveBeenCalledWith(
        expect.objectContaining({
          method: 'POST',
          path: '/api/v1/orders',
          status: 201,
        }),
        'Request completed',
      );
    });
  });

  describe('TelemetryConfig', () => {
    it('should accept all configuration fields', () => {
      const cfg: TelemetryConfig = {
        serviceName: 'my-service',
        version: '1.0.0',
        tier: 'business',
        environment: 'staging',
        traceEndpoint: 'http://otel-collector:4317',
        sampleRate: 0.5,
        logLevel: 'info',
      };

      expect(cfg.serviceName).toBe('my-service');
      expect(cfg.version).toBe('1.0.0');
      expect(cfg.tier).toBe('business');
      expect(cfg.environment).toBe('staging');
      expect(cfg.traceEndpoint).toBe('http://otel-collector:4317');
      expect(cfg.sampleRate).toBe(0.5);
      expect(cfg.logLevel).toBe('info');
    });

    it('should allow optional traceEndpoint and sampleRate', () => {
      const cfg: TelemetryConfig = {
        serviceName: 'simple-service',
        version: '1.0.0',
        tier: 'system',
        environment: 'dev',
        logLevel: 'debug',
      };

      expect(cfg.traceEndpoint).toBeUndefined();
      expect(cfg.sampleRate).toBeUndefined();
    });
  });
});
