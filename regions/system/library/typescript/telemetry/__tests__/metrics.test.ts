import { describe, it, expect, beforeEach } from 'vitest';
import { Metrics } from '../src/metrics';

describe('Metrics', () => {
  let metrics: Metrics;

  beforeEach(() => {
    metrics = new Metrics('test-service');
  });

  describe('初期化', () => {
    it('serviceName を保持する', () => {
      expect(metrics.serviceName).toBe('test-service');
    });

    it('カウンタとヒストグラムが初期化される', () => {
      expect(metrics).toBeDefined();
    });
  });

  describe('HTTP リクエストカウンタ', () => {
    it('HTTP リクエストカウンタを増加できる', () => {
      metrics.recordHTTPRequest('GET', '/api/users', 200);
      metrics.recordHTTPRequest('POST', '/api/users', 201);
      metrics.recordHTTPRequest('GET', '/api/users', 200);

      const output = metrics.getMetrics();
      expect(output).toContain('http_requests_total');
      expect(output).toContain('method="GET"');
      expect(output).toContain('method="POST"');
      expect(output).toContain('status="200"');
      expect(output).toContain('status="201"');
    });
  });

  describe('gRPC リクエストカウンタ', () => {
    it('gRPC リクエストカウンタを増加できる', () => {
      metrics.recordGRPCRequest('UserService', 'GetUser', 'OK');
      metrics.recordGRPCRequest('UserService', 'GetUser', 'NOT_FOUND');
      metrics.recordGRPCRequest('OrderService', 'CreateOrder', 'OK');

      const output = metrics.getMetrics();
      expect(output).toContain('grpc_server_handled_total');
      expect(output).toContain('grpc_service="UserService"');
      expect(output).toContain('grpc_method="GetUser"');
      expect(output).toContain('grpc_code="OK"');
      expect(output).toContain('grpc_code="NOT_FOUND"');
    });
  });

  describe('HTTP レイテンシヒストグラム', () => {
    it('HTTP レイテンシを記録できる', () => {
      metrics.recordHTTPDuration('GET', '/api/users', 0.05);
      metrics.recordHTTPDuration('GET', '/api/users', 0.15);

      const output = metrics.getMetrics();
      expect(output).toContain('http_request_duration_seconds_bucket');
      expect(output).toContain('http_request_duration_seconds_sum');
      expect(output).toContain('http_request_duration_seconds_count');
    });
  });

  describe('gRPC レイテンシヒストグラム', () => {
    it('gRPC レイテンシを記録できる', () => {
      metrics.recordGRPCDuration('UserService', 'GetUser', 0.03);
      metrics.recordGRPCDuration('OrderService', 'CreateOrder', 0.12);

      const output = metrics.getMetrics();
      expect(output).toContain('grpc_server_handling_seconds_bucket');
      expect(output).toContain('grpc_server_handling_seconds_sum');
      expect(output).toContain('grpc_server_handling_seconds_count');
    });
  });

  describe('Prometheus テキストフォーマット出力', () => {
    it('HELP と TYPE を含む Prometheus フォーマットを出力する', () => {
      metrics.recordHTTPRequest('GET', '/health', 200);
      metrics.recordGRPCRequest('HealthService', 'Check', 'OK');

      const output = metrics.getMetrics();
      expect(output).toContain('# HELP http_requests_total');
      expect(output).toContain('# TYPE http_requests_total counter');
      expect(output).toContain('# HELP grpc_server_handled_total');
      expect(output).toContain('# TYPE grpc_server_handled_total counter');
      expect(output).toContain('# HELP http_request_duration_seconds');
      expect(output).toContain('# TYPE http_request_duration_seconds histogram');
      expect(output).toContain('# HELP grpc_server_handling_seconds');
      expect(output).toContain('# TYPE grpc_server_handling_seconds histogram');
    });

    it('service ラベルが全メトリクスに含まれる', () => {
      metrics.recordHTTPRequest('GET', '/', 200);
      metrics.recordGRPCRequest('Svc', 'Method', 'OK');

      const output = metrics.getMetrics();
      expect(output).toContain('service="test-service"');
    });

    it('メトリクスが記録されていなくても出力できる', () => {
      const output = metrics.getMetrics();
      expect(typeof output).toBe('string');
    });
  });
});
