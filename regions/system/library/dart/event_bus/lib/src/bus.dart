import 'dart:async';

import 'event.dart';

// ---------- EventHandler ----------

/// Typed event handler interface following DDD patterns.
abstract class EventHandler<T extends DomainEvent> {
  Future<void> handle(T event);
}

// ---------- EventBusConfig ----------

/// Configuration for EventBus.
class EventBusConfig {
  /// Maximum buffer size for pending events. Default: 1024.
  final int bufferSize;

  /// Timeout in milliseconds for each handler invocation. Default: 5000.
  final int handlerTimeoutMs;

  const EventBusConfig({
    this.bufferSize = 1024,
    this.handlerTimeoutMs = 5000,
  });
}

// ---------- EventBusError ----------

/// Error codes for EventBus operations.
enum EventBusErrorCode {
  publishFailed,
  handlerFailed,
  channelClosed,
}

/// Typed error for EventBus operations.
class EventBusError implements Exception {
  final String message;
  final EventBusErrorCode code;

  /// String representation of the error code matching the design spec.
  String get codeString {
    switch (code) {
      case EventBusErrorCode.publishFailed:
        return 'PUBLISH_FAILED';
      case EventBusErrorCode.handlerFailed:
        return 'HANDLER_FAILED';
      case EventBusErrorCode.channelClosed:
        return 'CHANNEL_CLOSED';
    }
  }

  const EventBusError(this.message, this.code);

  @override
  String toString() => 'EventBusError($codeString): $message';
}

// ---------- EventSubscription ----------

/// Represents an active subscription that can be cancelled.
class EventSubscription {
  final String eventType;
  final void Function() _onUnsubscribe;
  bool _active = true;

  EventSubscription._(this.eventType, this._onUnsubscribe);

  /// Whether this subscription is still active.
  bool get isActive => _active;

  /// Unsubscribe this handler from the event bus.
  void unsubscribe() {
    if (_active) {
      _active = false;
      _onUnsubscribe();
    }
  }
}

// ---------- EventBus ----------

/// DDD-aligned event bus with typed handlers and configurable timeouts.
class EventBus {
  final EventBusConfig _config;
  final Map<String, List<_HandlerEntry>> _handlers = {};
  bool _closed = false;

  EventBus([EventBusConfig? config])
      : _config = config ?? const EventBusConfig();

  /// Publish an event to all subscribed handlers.
  Future<void> publish<T extends DomainEvent>(T event) async {
    if (_closed) {
      throw const EventBusError(
        'Cannot publish to a closed EventBus',
        EventBusErrorCode.channelClosed,
      );
    }

    final handlers = _handlers[event.eventType];
    if (handlers == null || handlers.isEmpty) return;

    for (final entry in List.of(handlers)) {
      try {
        await entry.handler
            .handle(event)
            .timeout(
              Duration(milliseconds: _config.handlerTimeoutMs),
              onTimeout: () {
                throw EventBusError(
                  'Handler timed out after ${_config.handlerTimeoutMs}ms '
                  'for event "${event.eventType}"',
                  EventBusErrorCode.handlerFailed,
                );
              },
            );
      } on EventBusError {
        rethrow;
      } catch (err) {
        throw EventBusError(
          'Handler failed for event "${event.eventType}": $err',
          EventBusErrorCode.handlerFailed,
        );
      }
    }
  }

  /// Subscribe a typed handler to events of [eventType].
  EventSubscription subscribe<T extends DomainEvent>(
    String eventType,
    EventHandler<T> handler,
  ) {
    if (_closed) {
      throw const EventBusError(
        'Cannot subscribe to a closed EventBus',
        EventBusErrorCode.channelClosed,
      );
    }

    final entry = _HandlerEntry(handler);
    (_handlers[eventType] ??= []).add(entry);

    return EventSubscription._(eventType, () {
      _handlers[eventType]?.remove(entry);
      if (_handlers[eventType]?.isEmpty ?? false) {
        _handlers.remove(eventType);
      }
    });
  }

  /// Close the event bus, preventing further publish/subscribe.
  void close() {
    _closed = true;
    _handlers.clear();
  }
}

/// Internal wrapper for identity-based handler removal.
class _HandlerEntry {
  final EventHandler<DomainEvent> handler;
  _HandlerEntry(EventHandler handler) : handler = handler as EventHandler<DomainEvent>;
}

// ---------- Legacy InMemoryEventBus (backward compat) ----------

typedef LegacyEventHandler = Future<void> Function(Event event);

/// Legacy event bus preserving the original function-based API.
class InMemoryEventBus {
  final EventBus _bus;
  final Map<String, List<EventSubscription>> _subscriptions = {};

  InMemoryEventBus([EventBusConfig? config]) : _bus = EventBus(config);

  void subscribe(String eventType, LegacyEventHandler handler) {
    final sub = _bus.subscribe<Event>(eventType, _FnHandler(handler));
    (_subscriptions[eventType] ??= []).add(sub);
  }

  void unsubscribe(String eventType) {
    _subscriptions[eventType]?.forEach((sub) => sub.unsubscribe());
    _subscriptions.remove(eventType);
  }

  Future<void> publish(Event event) => _bus.publish(event);
}

/// Adapter wrapping a plain function as an [EventHandler].
class _FnHandler implements EventHandler<Event> {
  final LegacyEventHandler _fn;
  _FnHandler(this._fn);

  @override
  Future<void> handle(Event event) => _fn(event);
}
