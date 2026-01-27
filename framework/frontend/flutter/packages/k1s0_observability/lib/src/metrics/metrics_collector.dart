import 'dart:async';
import 'dart:collection';

import 'performance_metric.dart';

/// Metrics exporter interface
abstract class MetricsExporter {
  /// Export metrics
  Future<void> export(List<PerformanceMetric> metrics);

  /// Shutdown the exporter
  Future<void> shutdown();
}

/// Console metrics exporter for debugging
class ConsoleMetricsExporter implements MetricsExporter {
  @override
  Future<void> export(List<PerformanceMetric> metrics) async {
    for (final metric in metrics) {
      // ignore: avoid_print
      print(
        '[METRIC] ${metric.name}: ${metric.formattedValue} '
        '${metric.tags.isNotEmpty ? metric.tags.toString() : ""}',
      );
    }
  }

  @override
  Future<void> shutdown() async {}
}

/// Metrics collector
class MetricsCollector {
  /// Creates a metrics collector
  MetricsCollector({
    this.exporter,
    this.batchSize = 50,
    this.flushInterval = const Duration(seconds: 30),
  }) {
    if (exporter != null) {
      _startFlushTimer();
    }
  }

  /// Metrics exporter
  final MetricsExporter? exporter;

  /// Batch size for export
  final int batchSize;

  /// Flush interval
  final Duration flushInterval;

  final Queue<PerformanceMetric> _buffer = Queue<PerformanceMetric>();
  final Map<String, List<PerformanceMetric>> _aggregated = {};
  Timer? _flushTimer;
  bool _disposed = false;

  void _startFlushTimer() {
    _flushTimer = Timer.periodic(flushInterval, (_) => flush());
  }

  /// Record a metric
  void record(PerformanceMetric metric) {
    if (_disposed) return;

    _buffer.add(metric);

    // Add to aggregated map for statistics
    _aggregated.putIfAbsent(metric.name, () => []);
    _aggregated[metric.name]!.add(metric);

    // Keep only last 100 metrics per name
    if (_aggregated[metric.name]!.length > 100) {
      _aggregated[metric.name]!.removeAt(0);
    }

    if (_buffer.length >= batchSize) {
      flush();
    }
  }

  /// Record a timing
  void recordTiming(
    String name,
    double milliseconds, {
    Map<String, String>? tags,
  }) {
    record(PerformanceMetric.timing(
      name: name,
      milliseconds: milliseconds,
      tags: tags,
    ),);
  }

  /// Record a counter increment
  void recordCount(
    String name, {
    int count = 1,
    Map<String, String>? tags,
  }) {
    record(PerformanceMetric.counter(
      name: name,
      count: count,
      tags: tags,
    ),);
  }

  /// Record a gauge value
  void recordGauge(
    String name,
    double value, {
    MetricUnit unit = MetricUnit.count,
    Map<String, String>? tags,
  }) {
    record(PerformanceMetric.gauge(
      name: name,
      value: value,
      unit: unit,
      tags: tags,
    ),);
  }

  /// Measure the duration of an async operation
  Future<T> measureAsync<T>(
    String name,
    Future<T> Function() operation, {
    Map<String, String>? tags,
  }) async {
    final stopwatch = Stopwatch()..start();

    try {
      final result = await operation();
      stopwatch.stop();
      recordTiming(
        name,
        stopwatch.elapsedMilliseconds.toDouble(),
        tags: {...?tags, 'status': 'success'},
      );
      return result;
    } catch (e) {
      stopwatch.stop();
      recordTiming(
        name,
        stopwatch.elapsedMilliseconds.toDouble(),
        tags: {...?tags, 'status': 'error'},
      );
      rethrow;
    }
  }

  /// Measure the duration of a sync operation
  T measureSync<T>(
    String name,
    T Function() operation, {
    Map<String, String>? tags,
  }) {
    final stopwatch = Stopwatch()..start();

    try {
      final result = operation();
      stopwatch.stop();
      recordTiming(
        name,
        stopwatch.elapsedMilliseconds.toDouble(),
        tags: {...?tags, 'status': 'success'},
      );
      return result;
    } catch (e) {
      stopwatch.stop();
      recordTiming(
        name,
        stopwatch.elapsedMilliseconds.toDouble(),
        tags: {...?tags, 'status': 'error'},
      );
      rethrow;
    }
  }

  /// Get statistics for a metric name
  MetricStatistics? getStatistics(String name) {
    final metrics = _aggregated[name];
    if (metrics == null || metrics.isEmpty) return null;

    final values = metrics.map((m) => m.value).toList()..sort();
    final sum = values.reduce((a, b) => a + b);
    final count = values.length;
    final mean = sum / count;

    return MetricStatistics(
      name: name,
      count: count,
      sum: sum,
      mean: mean,
      min: values.first,
      max: values.last,
      median: _calculateMedian(values),
      p95: _calculatePercentile(values, 95),
      p99: _calculatePercentile(values, 99),
    );
  }

  double _calculateMedian(List<double> sortedValues) {
    final length = sortedValues.length;
    if (length.isOdd) {
      return sortedValues[length ~/ 2];
    }
    return (sortedValues[length ~/ 2 - 1] + sortedValues[length ~/ 2]) / 2;
  }

  double _calculatePercentile(List<double> sortedValues, int percentile) {
    final index = (percentile / 100 * (sortedValues.length - 1)).round();
    return sortedValues[index.clamp(0, sortedValues.length - 1)];
  }

  /// Flush metrics to exporter
  Future<void> flush() async {
    if (_disposed || exporter == null || _buffer.isEmpty) return;

    final metrics = List<PerformanceMetric>.from(_buffer);
    _buffer.clear();

    await exporter!.export(metrics);
  }

  /// Dispose the collector
  Future<void> dispose() async {
    _disposed = true;
    _flushTimer?.cancel();
    await flush();
    await exporter?.shutdown();
  }
}

/// Metric statistics
class MetricStatistics {
  /// Creates metric statistics
  const MetricStatistics({
    required this.name,
    required this.count,
    required this.sum,
    required this.mean,
    required this.min,
    required this.max,
    required this.median,
    required this.p95,
    required this.p99,
  });

  /// Metric name
  final String name;

  /// Number of samples
  final int count;

  /// Sum of all values
  final double sum;

  /// Mean (average)
  final double mean;

  /// Minimum value
  final double min;

  /// Maximum value
  final double max;

  /// Median (50th percentile)
  final double median;

  /// 95th percentile
  final double p95;

  /// 99th percentile
  final double p99;

  @override
  String toString() =>
      'MetricStatistics($name): '
      'count=$count, mean=${mean.toStringAsFixed(2)}, '
      'min=${min.toStringAsFixed(2)}, max=${max.toStringAsFixed(2)}, '
      'p95=${p95.toStringAsFixed(2)}, p99=${p99.toStringAsFixed(2)}';
}
