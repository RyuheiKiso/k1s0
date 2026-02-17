/// メトリクスのキー（HTTP リクエストカウンタ用）。
class MetricsKey {
  final String method;
  final String path;
  final String status;

  MetricsKey({
    required this.method,
    required this.path,
    required this.status,
  });

  @override
  bool operator ==(Object other) =>
      other is MetricsKey &&
      method == other.method &&
      path == other.path &&
      status == other.status;

  @override
  int get hashCode => Object.hash(method, path, status);
}

/// gRPC メトリクスのキー。
class GrpcMetricsKey {
  final String service;
  final String method;
  final String code;

  GrpcMetricsKey({
    required this.service,
    required this.method,
    required this.code,
  });

  @override
  bool operator ==(Object other) =>
      other is GrpcMetricsKey &&
      service == other.service &&
      method == other.method &&
      code == other.code;

  @override
  int get hashCode => Object.hash(service, method, code);
}

/// ヒストグラムのキー（method + path）。
class _HistogramKey {
  final String method;
  final String path;

  _HistogramKey({required this.method, required this.path});

  @override
  bool operator ==(Object other) =>
      other is _HistogramKey &&
      method == other.method &&
      path == other.path;

  @override
  int get hashCode => Object.hash(method, path);
}

/// gRPC ヒストグラムのキー（service + method）。
class _GrpcHistogramKey {
  final String service;
  final String method;

  _GrpcHistogramKey({required this.service, required this.method});

  @override
  bool operator ==(Object other) =>
      other is _GrpcHistogramKey &&
      service == other.service &&
      method == other.method;

  @override
  int get hashCode => Object.hash(service, method);
}

/// ヒストグラムデータを保持するクラス。
class _HistogramData {
  final Map<double, int> buckets;
  double sum;
  int count;

  _HistogramData(List<double> boundaries)
      : buckets = {for (final b in boundaries) b: 0},
        sum = 0,
        count = 0;

  void observe(double value) {
    count++;
    sum += value;
    for (final boundary in buckets.keys) {
      if (value <= boundary) {
        buckets[boundary] = buckets[boundary]! + 1;
      }
    }
  }
}

/// Prometheus メトリクスのヘルパークラス。
/// RED メソッド（Rate, Errors, Duration）のメトリクスを提供する。
class Metrics {
  /// サービス名。メトリクスの service ラベルに使用される。
  final String serviceName;

  /// HTTP リクエストカウンタ。
  final Map<MetricsKey, int> httpRequestsTotal = {};

  /// gRPC リクエストカウンタ。
  final Map<GrpcMetricsKey, int> grpcHandledTotal = {};

  /// HTTP レイテンシヒストグラムデータ。
  final Map<_HistogramKey, _HistogramData> httpDurationBuckets = {};

  /// gRPC レイテンシヒストグラムデータ。
  final Map<_GrpcHistogramKey, _HistogramData> _grpcDurationBuckets = {};

  /// Prometheus ヒストグラムのバケット境界。
  static const List<double> _defaultBuckets = [
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10,
  ];

  /// Metrics を初期化する。
  /// [serviceName] はメトリクスの service ラベルに使用される。
  Metrics({required this.serviceName});

  /// HTTP リクエストを記録する。
  void recordHttpRequest({
    required String method,
    required String path,
    required int statusCode,
  }) {
    final key = MetricsKey(
      method: method,
      path: path,
      status: statusCode.toString(),
    );
    httpRequestsTotal[key] = (httpRequestsTotal[key] ?? 0) + 1;
  }

  /// HTTP リクエストのレイテンシを記録する。
  void recordHttpDuration({
    required String method,
    required String path,
    required Duration duration,
  }) {
    final key = _HistogramKey(method: method, path: path);
    final data = httpDurationBuckets.putIfAbsent(
      key,
      () => _HistogramData(_defaultBuckets),
    );
    data.observe(duration.inMicroseconds / 1000000.0);
  }

  /// gRPC リクエストを記録する。
  void recordGrpcRequest({
    required String service,
    required String method,
    required String code,
  }) {
    final key = GrpcMetricsKey(
      service: service,
      method: method,
      code: code,
    );
    grpcHandledTotal[key] = (grpcHandledTotal[key] ?? 0) + 1;
  }

  /// gRPC リクエストのレイテンシを記録する。
  void recordGrpcDuration({
    required String service,
    required String method,
    required Duration duration,
  }) {
    final key = _GrpcHistogramKey(service: service, method: method);
    final data = _grpcDurationBuckets.putIfAbsent(
      key,
      () => _HistogramData(_defaultBuckets),
    );
    data.observe(duration.inMicroseconds / 1000000.0);
  }

  /// Prometheus テキストフォーマットでメトリクスを出力する。
  String toPrometheusText() {
    final buf = StringBuffer();

    // HTTP requests total
    buf.writeln(
      '# HELP http_requests_total Total number of HTTP requests',
    );
    buf.writeln('# TYPE http_requests_total counter');
    for (final entry in httpRequestsTotal.entries) {
      final k = entry.key;
      buf.writeln(
        'http_requests_total{service="$serviceName",'
        'method="${k.method}",path="${k.path}",'
        'status="${k.status}"} ${entry.value}',
      );
    }

    // HTTP request duration
    buf.writeln(
      '# HELP http_request_duration_seconds '
      'Histogram of HTTP request latency',
    );
    buf.writeln('# TYPE http_request_duration_seconds histogram');
    for (final entry in httpDurationBuckets.entries) {
      final k = entry.key;
      final data = entry.value;
      final labels = 'service="$serviceName",'
          'method="${k.method}",path="${k.path}"';
      for (final bucket in data.buckets.entries) {
        buf.writeln(
          'http_request_duration_seconds_bucket'
          '{$labels,le="${bucket.key}"} ${bucket.value}',
        );
      }
      buf.writeln(
        'http_request_duration_seconds_bucket'
        '{$labels,le="+Inf"} ${data.count}',
      );
      buf.writeln(
        'http_request_duration_seconds_sum{$labels} ${data.sum}',
      );
      buf.writeln(
        'http_request_duration_seconds_count{$labels} ${data.count}',
      );
    }

    // gRPC handled total
    buf.writeln(
      '# HELP grpc_server_handled_total '
      'Total number of RPCs completed on the server',
    );
    buf.writeln('# TYPE grpc_server_handled_total counter');
    for (final entry in grpcHandledTotal.entries) {
      final k = entry.key;
      buf.writeln(
        'grpc_server_handled_total{service="$serviceName",'
        'grpc_service="${k.service}",grpc_method="${k.method}",'
        'grpc_code="${k.code}"} ${entry.value}',
      );
    }

    // gRPC handling duration
    buf.writeln(
      '# HELP grpc_server_handling_seconds '
      'Histogram of response latency of gRPC',
    );
    buf.writeln('# TYPE grpc_server_handling_seconds histogram');
    for (final entry in _grpcDurationBuckets.entries) {
      final k = entry.key;
      final data = entry.value;
      final labels = 'service="$serviceName",'
          'grpc_service="${k.service}",grpc_method="${k.method}"';
      for (final bucket in data.buckets.entries) {
        buf.writeln(
          'grpc_server_handling_seconds_bucket'
          '{$labels,le="${bucket.key}"} ${bucket.value}',
        );
      }
      buf.writeln(
        'grpc_server_handling_seconds_bucket'
        '{$labels,le="+Inf"} ${data.count}',
      );
      buf.writeln(
        'grpc_server_handling_seconds_sum{$labels} ${data.sum}',
      );
      buf.writeln(
        'grpc_server_handling_seconds_count{$labels} ${data.count}',
      );
    }

    return buf.toString();
  }
}
