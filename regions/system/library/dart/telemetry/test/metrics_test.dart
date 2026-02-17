import 'package:test/test.dart';

import 'package:k1s0_telemetry/telemetry.dart';

void main() {
  group('Metrics', () {
    late Metrics metrics;

    setUp(() {
      metrics = Metrics(serviceName: 'test-service');
    });

    test('should initialize with service name', () {
      expect(metrics.serviceName, 'test-service');
    });

    test('should have zero counters initially', () {
      // 初期状態ではカウンタマップが空であること
      expect(metrics.httpRequestsTotal, isEmpty);
      expect(metrics.grpcHandledTotal, isEmpty);
      expect(metrics.httpDurationBuckets, isEmpty);

      // HELP/TYPE 行は出力されるがデータ行はない
      final output = metrics.toPrometheusText();
      expect(output, contains('# HELP http_requests_total'));
      expect(output, isNot(contains('http_requests_total{')));
    });

    group('HTTP request counter', () {
      test('should increment counter for a request', () {
        metrics.recordHttpRequest(
          method: 'GET',
          path: '/api/users',
          statusCode: 200,
        );

        expect(metrics.httpRequestsTotal, isNotEmpty);
        final key = MetricsKey(method: 'GET', path: '/api/users', status: '200');
        expect(metrics.httpRequestsTotal[key], 1);
      });

      test('should increment counter for multiple requests', () {
        metrics.recordHttpRequest(
          method: 'GET',
          path: '/api/users',
          statusCode: 200,
        );
        metrics.recordHttpRequest(
          method: 'GET',
          path: '/api/users',
          statusCode: 200,
        );
        metrics.recordHttpRequest(
          method: 'POST',
          path: '/api/users',
          statusCode: 201,
        );

        final getKey =
            MetricsKey(method: 'GET', path: '/api/users', status: '200');
        final postKey =
            MetricsKey(method: 'POST', path: '/api/users', status: '201');
        expect(metrics.httpRequestsTotal[getKey], 2);
        expect(metrics.httpRequestsTotal[postKey], 1);
      });
    });

    group('Latency histogram', () {
      test('should record latency', () {
        metrics.recordHttpDuration(
          method: 'GET',
          path: '/api/users',
          duration: Duration(milliseconds: 50),
        );

        expect(metrics.httpDurationBuckets, isNotEmpty);
      });

      test('should categorize latency into correct buckets', () {
        // 5ms request -> should fall into 0.005 and higher buckets
        metrics.recordHttpDuration(
          method: 'GET',
          path: '/api/fast',
          duration: Duration(milliseconds: 5),
        );

        // 500ms request -> should fall into 0.5 and higher buckets
        metrics.recordHttpDuration(
          method: 'GET',
          path: '/api/slow',
          duration: Duration(milliseconds: 500),
        );

        final output = metrics.toPrometheusText();
        expect(output, contains('http_request_duration_seconds_bucket'));
      });

      test('should track sum and count for histogram', () {
        metrics.recordHttpDuration(
          method: 'GET',
          path: '/api/users',
          duration: Duration(milliseconds: 100),
        );
        metrics.recordHttpDuration(
          method: 'GET',
          path: '/api/users',
          duration: Duration(milliseconds: 200),
        );

        final output = metrics.toPrometheusText();
        expect(output, contains('http_request_duration_seconds_sum'));
        expect(output, contains('http_request_duration_seconds_count'));
      });
    });

    group('gRPC metrics', () {
      test('should record gRPC request', () {
        metrics.recordGrpcRequest(
          service: 'UserService',
          method: 'GetUser',
          code: 'OK',
        );

        final key = GrpcMetricsKey(
          service: 'UserService',
          method: 'GetUser',
          code: 'OK',
        );
        expect(metrics.grpcHandledTotal[key], 1);
      });

      test('should record gRPC duration', () {
        metrics.recordGrpcDuration(
          service: 'UserService',
          method: 'GetUser',
          duration: Duration(milliseconds: 30),
        );

        final output = metrics.toPrometheusText();
        expect(output, contains('grpc_server_handling_seconds'));
      });
    });

    group('Prometheus text format', () {
      test('should output valid Prometheus text format', () {
        metrics.recordHttpRequest(
          method: 'GET',
          path: '/api/users',
          statusCode: 200,
        );
        metrics.recordHttpDuration(
          method: 'GET',
          path: '/api/users',
          duration: Duration(milliseconds: 150),
        );

        final output = metrics.toPrometheusText();

        // HELP lines
        expect(output, contains('# HELP http_requests_total'));
        expect(output, contains('# TYPE http_requests_total counter'));
        expect(output, contains('# HELP http_request_duration_seconds'));
        expect(
          output,
          contains('# TYPE http_request_duration_seconds histogram'),
        );

        // Labels
        expect(output, contains('service="test-service"'));
        expect(output, contains('method="GET"'));
        expect(output, contains('path="/api/users"'));
        expect(output, contains('status="200"'));
      });

      test('should output empty metrics when nothing recorded', () {
        final output = metrics.toPrometheusText();
        // HELP/TYPE lines should still be present
        expect(output, contains('# HELP http_requests_total'));
        expect(output, contains('# HELP http_request_duration_seconds'));
        expect(output, contains('# HELP grpc_server_handled_total'));
        expect(output, contains('# HELP grpc_server_handling_seconds'));
      });
    });
  });
}
