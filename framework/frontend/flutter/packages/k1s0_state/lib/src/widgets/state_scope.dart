import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../utils/state_logger.dart';

/// A widget that provides a scoped ProviderContainer.
///
/// Use this to create isolated state scopes within your app.
class StateScope extends StatefulWidget {
  const StateScope({
    required this.child,
    this.overrides = const [],
    this.observers,
    this.parent,
    super.key,
  });

  /// The child widget.
  final Widget child;

  /// Provider overrides for this scope.
  final List<Override> overrides;

  /// Observers for this scope.
  final List<ProviderObserver>? observers;

  /// Parent container (optional).
  final ProviderContainer? parent;

  @override
  State<StateScope> createState() => _StateScopeState();
}

class _StateScopeState extends State<StateScope> {
  late ProviderContainer _container;

  @override
  void initState() {
    super.initState();
    _container = ProviderContainer(
      parent: widget.parent,
      overrides: widget.overrides,
      observers: widget.observers,
    );
  }

  @override
  void dispose() {
    _container.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return UncontrolledProviderScope(
      container: _container,
      child: widget.child,
    );
  }
}

/// A widget that provides app-level state initialization.
///
/// Use this at the root of your app to set up state management.
class K1s0StateProvider extends StatelessWidget {
  const K1s0StateProvider({
    required this.child,
    this.overrides = const [],
    this.enableLogging = false,
    this.observers,
    super.key,
  });

  /// The child widget.
  final Widget child;

  /// Provider overrides.
  final List<Override> overrides;

  /// Whether to enable state logging.
  final bool enableLogging;

  /// Additional observers.
  final List<ProviderObserver>? observers;

  @override
  Widget build(BuildContext context) {
    final allObservers = <ProviderObserver>[
      if (enableLogging) StateLogger(),
      ...?observers,
    ];

    return ProviderScope(
      overrides: overrides,
      observers: allObservers.isNotEmpty ? allObservers : null,
      child: child,
    );
  }
}

/// A widget that provides state initialization.
///
/// Use this to initialize state when the widget is first built.
class StateInitializer extends ConsumerStatefulWidget {
  const StateInitializer({
    required this.child,
    required this.initialize,
    this.dispose,
    this.loading,
    super.key,
  });

  /// The child widget.
  final Widget child;

  /// Initialization callback.
  final Future<void> Function(WidgetRef ref) initialize;

  /// Disposal callback.
  final void Function(WidgetRef ref)? dispose;

  /// Loading widget.
  final Widget? loading;

  @override
  ConsumerState<StateInitializer> createState() => _StateInitializerState();
}

class _StateInitializerState extends ConsumerState<StateInitializer> {
  bool _initialized = false;
  bool _initializing = false;

  @override
  void initState() {
    super.initState();
    _init();
  }

  Future<void> _init() async {
    if (_initializing || _initialized) return;
    _initializing = true;

    await widget.initialize(ref);

    if (mounted) {
      setState(() {
        _initialized = true;
        _initializing = false;
      });
    }
  }

  @override
  void dispose() {
    widget.dispose?.call(ref);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (!_initialized) {
      return widget.loading ??
          const Scaffold(
            body: Center(
              child: CircularProgressIndicator(),
            ),
          );
    }

    return widget.child;
  }
}

/// Extension methods for WidgetRef.
extension WidgetRefExtensions on WidgetRef {
  /// Watches multiple providers and returns a tuple.
  (T1, T2) watch2<T1, T2>(
    ProviderListenable<T1> p1,
    ProviderListenable<T2> p2,
  ) {
    return (watch(p1), watch(p2));
  }

  /// Watches three providers and returns a tuple.
  (T1, T2, T3) watch3<T1, T2, T3>(
    ProviderListenable<T1> p1,
    ProviderListenable<T2> p2,
    ProviderListenable<T3> p3,
  ) {
    return (watch(p1), watch(p2), watch(p3));
  }

  /// Reads multiple providers and returns a tuple.
  (T1, T2) read2<T1, T2>(
    ProviderListenable<T1> p1,
    ProviderListenable<T2> p2,
  ) {
    return (read(p1), read(p2));
  }

  /// Reads three providers and returns a tuple.
  (T1, T2, T3) read3<T1, T2, T3>(
    ProviderListenable<T1> p1,
    ProviderListenable<T2> p2,
    ProviderListenable<T3> p3,
  ) {
    return (read(p1), read(p2), read(p3));
  }
}

/// Extension methods for Ref.
extension RefExtensions on Ref {
  /// Watches multiple providers and returns a tuple.
  (T1, T2) watch2<T1, T2>(
    ProviderListenable<T1> p1,
    ProviderListenable<T2> p2,
  ) {
    return (watch(p1), watch(p2));
  }

  /// Watches three providers and returns a tuple.
  (T1, T2, T3) watch3<T1, T2, T3>(
    ProviderListenable<T1> p1,
    ProviderListenable<T2> p2,
    ProviderListenable<T3> p3,
  ) {
    return (watch(p1), watch(p2), watch(p3));
  }

  /// Reads multiple providers and returns a tuple.
  (T1, T2) read2<T1, T2>(
    ProviderListenable<T1> p1,
    ProviderListenable<T2> p2,
  ) {
    return (read(p1), read(p2));
  }

  /// Reads three providers and returns a tuple.
  (T1, T2, T3) read3<T1, T2, T3>(
    ProviderListenable<T1> p1,
    ProviderListenable<T2> p2,
    ProviderListenable<T3> p3,
  ) {
    return (read(p1), read(p2), read(p3));
  }
}
