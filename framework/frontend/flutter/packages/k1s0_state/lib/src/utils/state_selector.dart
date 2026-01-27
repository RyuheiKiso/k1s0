import 'package:flutter_riverpod/flutter_riverpod.dart';

/// A selector for optimizing state updates.
///
/// Use this to select specific parts of a state to avoid unnecessary rebuilds.
class StateSelector<State, Selected> {
  /// Creates a state selector with the given selector function.
  StateSelector(this._selector, {this.equals});

  final Selected Function(State state) _selector;

  /// Optional equality function for comparing selected values.
  final bool Function(Selected previous, Selected current)? equals;

  /// Selects from a provider.
  ProviderListenable<Selected> select(ProviderListenable<State> provider) =>
      provider.select(_selector);

  /// Creates a derived provider that selects from another provider.
  Provider<Selected> createProvider(ProviderListenable<State> source) =>
      Provider((ref) => ref.watch(source.select(_selector)));

  /// Compares two selected values.
  bool areEqual(Selected previous, Selected current) {
    if (equals != null) {
      return equals!(previous, current);
    }
    return previous == current;
  }
}

/// Builder for creating state selectors.
class StateSelectorBuilder<State> {
  /// Creates a state selector builder.
  const StateSelectorBuilder();

  /// Creates a selector for a specific part of the state.
  StateSelector<State, Selected> select<Selected>(
    Selected Function(State state) selector, {
    bool Function(Selected previous, Selected current)? equals,
  }) =>
      StateSelector(selector, equals: equals);

  /// Creates a selector that combines multiple values.
  StateSelector<State, (S1, S2)> combine2<S1, S2>(
    S1 Function(State state) selector1,
    S2 Function(State state) selector2,
  ) =>
      StateSelector(
        (state) => (selector1(state), selector2(state)),
      );

  /// Creates a selector that combines three values.
  StateSelector<State, (S1, S2, S3)> combine3<S1, S2, S3>(
    S1 Function(State state) selector1,
    S2 Function(State state) selector2,
    S3 Function(State state) selector3,
  ) =>
      StateSelector(
        (state) => (selector1(state), selector2(state), selector3(state)),
      );
}

/// Extension for easier selector creation.
extension ProviderSelectorExtension<T> on ProviderListenable<T> {
  /// Creates a selector for this provider.
  ProviderListenable<R> selectWith<R>(StateSelector<T, R> selector) =>
      selector.select(this);
}

/// A memoized selector that caches the result.
class MemoizedSelector<State, Selected> {
  /// Creates a memoized selector.
  MemoizedSelector(this._selector);

  final Selected Function(State state) _selector;
  State? _lastState;
  Selected? _lastResult;

  /// Selects from the state, returning cached result if state unchanged.
  Selected call(State state) {
    if (identical(state, _lastState)) {
      return _lastResult as Selected;
    }
    _lastState = state;
    _lastResult = _selector(state);
    return _lastResult as Selected;
  }

  /// Clears the cache.
  void clear() {
    _lastState = null;
    _lastResult = null;
  }
}

/// Creates a memoized selector.
MemoizedSelector<State, Selected> createMemoizedSelector<State, Selected>(
  Selected Function(State state) selector,
) =>
    MemoizedSelector(selector);

/// A computed value that depends on multiple providers.
class ComputedValue<T> {
  /// Creates a computed value.
  ComputedValue(this._compute);

  final T Function(Ref ref) _compute;

  /// Creates a provider for this computed value.
  Provider<T> toProvider() => Provider(_compute);

  /// Creates an auto-dispose provider for this computed value.
  AutoDisposeProvider<T> toAutoDisposeProvider() => Provider.autoDispose(_compute);
}

/// Creates a computed value.
ComputedValue<T> computed<T>(T Function(Ref ref) compute) => ComputedValue(compute);
