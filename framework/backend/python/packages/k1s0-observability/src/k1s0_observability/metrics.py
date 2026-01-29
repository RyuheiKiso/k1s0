"""Custom metrics helpers."""

from __future__ import annotations

from opentelemetry import metrics

_meter: metrics.Meter | None = None


def _get_meter(name: str = "k1s0") -> metrics.Meter:
    """Get or create the default meter."""
    global _meter  # noqa: PLW0603
    if _meter is None:
        _meter = metrics.get_meter(name)
    return _meter


def create_counter(
    name: str,
    description: str = "",
    unit: str = "",
) -> metrics.Counter:
    """Create an OpenTelemetry counter metric.

    Args:
        name: Metric name (e.g., "requests_total").
        description: Human-readable description.
        unit: Metric unit (e.g., "1", "ms").

    Returns:
        An OpenTelemetry Counter.
    """
    return _get_meter().create_counter(name=name, description=description, unit=unit)


def create_histogram(
    name: str,
    description: str = "",
    unit: str = "",
) -> metrics.Histogram:
    """Create an OpenTelemetry histogram metric.

    Args:
        name: Metric name (e.g., "request_duration").
        description: Human-readable description.
        unit: Metric unit (e.g., "ms").

    Returns:
        An OpenTelemetry Histogram.
    """
    return _get_meter().create_histogram(name=name, description=description, unit=unit)
