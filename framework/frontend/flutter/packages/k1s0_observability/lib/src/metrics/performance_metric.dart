import 'package:freezed_annotation/freezed_annotation.dart';

part 'performance_metric.freezed.dart';
part 'performance_metric.g.dart';

/// Metric unit
enum MetricUnit {
  /// Milliseconds
  milliseconds,

  /// Bytes
  bytes,

  /// Count
  count,

  /// Percent (0-100)
  percent,
}

/// Performance metric
@freezed
class PerformanceMetric with _$PerformanceMetric {
  /// Creates a performance metric
  const factory PerformanceMetric({
    /// Metric name
    required String name,

    /// Metric value
    required double value,

    /// Metric unit
    required MetricUnit unit,

    /// Timestamp (Unix milliseconds)
    required int timestamp,

    /// Tags for grouping
    @Default({}) Map<String, String> tags,
  }) = _PerformanceMetric;

  const PerformanceMetric._();

  /// Creates a performance metric from JSON
  factory PerformanceMetric.fromJson(Map<String, dynamic> json) =>
      _$PerformanceMetricFromJson(json);

  /// Create a timing metric
  factory PerformanceMetric.timing({
    required String name,
    required double milliseconds,
    Map<String, String>? tags,
  }) =>
      PerformanceMetric(
        name: name,
        value: milliseconds,
        unit: MetricUnit.milliseconds,
        timestamp: DateTime.now().millisecondsSinceEpoch,
        tags: tags ?? {},
      );

  /// Create a counter metric
  factory PerformanceMetric.counter({
    required String name,
    required int count,
    Map<String, String>? tags,
  }) =>
      PerformanceMetric(
        name: name,
        value: count.toDouble(),
        unit: MetricUnit.count,
        timestamp: DateTime.now().millisecondsSinceEpoch,
        tags: tags ?? {},
      );

  /// Create a gauge metric
  factory PerformanceMetric.gauge({
    required String name,
    required double value,
    MetricUnit unit = MetricUnit.count,
    Map<String, String>? tags,
  }) =>
      PerformanceMetric(
        name: name,
        value: value,
        unit: unit,
        timestamp: DateTime.now().millisecondsSinceEpoch,
        tags: tags ?? {},
      );

  /// Get timestamp as DateTime
  DateTime get dateTime => DateTime.fromMillisecondsSinceEpoch(timestamp);

  /// Get formatted value with unit
  String get formattedValue {
    switch (unit) {
      case MetricUnit.milliseconds:
        return '${value.toStringAsFixed(2)}ms';
      case MetricUnit.bytes:
        if (value >= 1024 * 1024) {
          return '${(value / (1024 * 1024)).toStringAsFixed(2)}MB';
        } else if (value >= 1024) {
          return '${(value / 1024).toStringAsFixed(2)}KB';
        }
        return '${value.toInt()}B';
      case MetricUnit.count:
        return value.toInt().toString();
      case MetricUnit.percent:
        return '${value.toStringAsFixed(1)}%';
    }
  }
}
