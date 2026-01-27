import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

/// A consumer widget that provides convenient callbacks for state changes.
class StateConsumer<T> extends ConsumerStatefulWidget {
  /// Creates a StateConsumer.
  const StateConsumer({
    required this.provider,
    required this.builder,
    this.onStateChanged,
    this.listenWhen,
    super.key,
  });

  /// The provider to watch.
  final ProviderListenable<T> provider;

  /// Builder for the widget.
  final Widget Function(BuildContext context, T state, WidgetRef ref) builder;

  /// Callback when state changes.
  final void Function(T previous, T current)? onStateChanged;

  /// Optional condition for when to call onStateChanged.
  final bool Function(T previous, T current)? listenWhen;

  @override
  ConsumerState<StateConsumer<T>> createState() => _StateConsumerState<T>();
}

class _StateConsumerState<T> extends ConsumerState<StateConsumer<T>> {
  T? _previousState;

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(widget.provider);

    // Handle state change callback
    if (widget.onStateChanged != null && _previousState != null) {
      final previous = _previousState as T;
      final shouldCall = widget.listenWhen?.call(previous, state) ?? true;
      if (shouldCall && previous != state) {
        WidgetsBinding.instance.addPostFrameCallback((_) {
          widget.onStateChanged!(previous, state);
        });
      }
    }

    _previousState = state;

    return widget.builder(context, state, ref);
  }
}

/// A consumer that only listens to state changes without rebuilding.
class StateListener<T> extends ConsumerStatefulWidget {
  /// Creates a StateListener.
  const StateListener({
    required this.provider,
    required this.onStateChanged,
    required this.child,
    this.listenWhen,
    super.key,
  });

  /// The provider to listen to.
  final ProviderListenable<T> provider;

  /// Callback when state changes.
  final void Function(BuildContext context, T previous, T current)
      onStateChanged;

  /// The child widget.
  final Widget child;

  /// Optional condition for when to call onStateChanged.
  final bool Function(T previous, T current)? listenWhen;

  @override
  ConsumerState<StateListener<T>> createState() => _StateListenerState<T>();
}

class _StateListenerState<T> extends ConsumerState<StateListener<T>> {
  @override
  void initState() {
    super.initState();
    // Setup listener
    ref.listenManual(
      widget.provider,
      (previous, current) {
        if (previous == null) return;
        final shouldCall =
            widget.listenWhen?.call(previous, current) ?? true;
        if (shouldCall) {
          widget.onStateChanged(context, previous, current);
        }
      },
      fireImmediately: false,
    );
  }

  @override
  Widget build(BuildContext context) => widget.child;
}

/// A widget that combines listener and builder.
class StateListenerBuilder<T> extends ConsumerStatefulWidget {
  /// Creates a StateListenerBuilder.
  const StateListenerBuilder({
    required this.provider,
    required this.builder,
    required this.listener,
    this.listenWhen,
    this.buildWhen,
    super.key,
  });

  /// The provider to watch.
  final ProviderListenable<T> provider;

  /// Builder for the widget.
  final Widget Function(BuildContext context, T state) builder;

  /// Listener callback.
  final void Function(BuildContext context, T previous, T current) listener;

  /// Condition for when to call listener.
  final bool Function(T previous, T current)? listenWhen;

  /// Condition for when to rebuild.
  final bool Function(T previous, T current)? buildWhen;

  @override
  ConsumerState<StateListenerBuilder<T>> createState() =>
      _StateListenerBuilderState<T>();
}

class _StateListenerBuilderState<T>
    extends ConsumerState<StateListenerBuilder<T>> {
  T? _previousState;
  T? _buildState;

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(widget.provider);

    // Handle listener callback
    if (_previousState != null) {
      final previous = _previousState as T;
      final shouldListen = widget.listenWhen?.call(previous, state) ?? true;
      if (shouldListen && previous != state) {
        WidgetsBinding.instance.addPostFrameCallback((_) {
          widget.listener(context, previous, state);
        });
      }
    }

    _previousState = state;

    // Check if should rebuild
    if (_buildState != null) {
      final previous = _buildState as T;
      final shouldRebuild = widget.buildWhen?.call(previous, state) ?? true;
      if (!shouldRebuild) {
        return widget.builder(context, _buildState as T);
      }
    }

    _buildState = state;
    return widget.builder(context, state);
  }
}

/// A consumer that selects a specific part of the state.
class SelectiveConsumer<S, Selected> extends ConsumerWidget {
  /// Creates a SelectiveConsumer.
  const SelectiveConsumer({
    required this.provider,
    required this.selector,
    required this.builder,
    super.key,
  });

  /// The provider to watch.
  final ProviderListenable<S> provider;

  /// Selector for the part of state to watch.
  final Selected Function(S state) selector;

  /// Builder for the widget.
  final Widget Function(BuildContext context, Selected selected, WidgetRef ref)
      builder;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final selected = ref.watch(provider.select(selector));
    return builder(context, selected, ref);
  }
}
