import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'state_storage.dart';

/// A notifier that automatically persists its state to storage.
///
/// Extend this class to create a notifier that automatically
/// saves and restores its state.
abstract class PersistedStateNotifier<T> extends StateNotifier<T> {
  PersistedStateNotifier(
    super.state, {
    required this.storage,
    required this.key,
  }) {
    _loadState();
  }

  /// The storage to use for persistence.
  final StateStorage storage;

  /// The key to use for storing the state.
  final String key;

  /// Converts the state to JSON for storage.
  Map<String, dynamic> toJson(T state);

  /// Converts JSON from storage to state.
  T fromJson(Map<String, dynamic> json);

  /// Loads the state from storage.
  Future<void> _loadState() async {
    final json = await storage.read<Map<String, dynamic>>(key);
    if (json != null) {
      state = fromJson(json);
    }
  }

  /// Saves the current state to storage.
  Future<void> _saveState() async {
    await storage.write(key, toJson(state));
  }

  @override
  set state(T value) {
    super.state = value;
    _saveState();
  }

  /// Clears the persisted state.
  Future<void> clearPersistedState() async {
    await storage.delete(key);
  }
}

/// A provider for persisted state.
///
/// Use this to create providers that automatically persist state.
class PersistedStateProvider<T> {
  PersistedStateProvider({
    required this.storage,
    required this.key,
    required this.serializer,
    required this.defaultValue,
  });

  final StateStorage storage;
  final String key;
  final StateSerializer<T> serializer;
  final T defaultValue;

  /// Creates a StateNotifierProvider for the persisted state.
  StateNotifierProvider<_PersistedNotifier<T>, T> createProvider() {
    return StateNotifierProvider<_PersistedNotifier<T>, T>((ref) {
      return _PersistedNotifier<T>(
        defaultValue,
        storage: storage,
        key: key,
        serializer: serializer,
      );
    });
  }
}

class _PersistedNotifier<T> extends StateNotifier<T> {
  _PersistedNotifier(
    super.state, {
    required this.storage,
    required this.key,
    required this.serializer,
  }) {
    _loadState();
  }

  final StateStorage storage;
  final String key;
  final StateSerializer<T> serializer;

  Future<void> _loadState() async {
    final json = await storage.read<Map<String, dynamic>>(key);
    if (json != null) {
      state = serializer.fromJson(json);
    }
  }

  Future<void> _saveState() async {
    await storage.write(key, serializer.toJson(state));
  }

  @override
  set state(T value) {
    super.state = value;
    _saveState();
  }

  void update(T value) {
    state = value;
  }

  Future<void> clear() async {
    await storage.delete(key);
  }
}

/// Extension for creating persisted providers.
extension PersistedProviderExtension on Ref {
  /// Creates a persisted value that automatically saves to storage.
  Future<T> persistedValue<T>({
    required StateStorage storage,
    required String key,
    required StateSerializer<T> serializer,
    required T defaultValue,
  }) async {
    final json = await storage.read<Map<String, dynamic>>(key);
    if (json != null) {
      return serializer.fromJson(json);
    }
    return defaultValue;
  }
}

/// A mixin for adding persistence to existing notifiers.
mixin StatePersistenceMixin<T> {
  StateStorage get storage;
  String get persistenceKey;

  /// Converts the state to JSON for storage.
  Map<String, dynamic> stateToJson(T state);

  /// Converts JSON from storage to state.
  T stateFromJson(Map<String, dynamic> json);

  /// Loads the state from storage.
  Future<T?> loadPersistedState() async {
    final json = await storage.read<Map<String, dynamic>>(persistenceKey);
    if (json != null) {
      return stateFromJson(json);
    }
    return null;
  }

  /// Saves the state to storage.
  Future<void> persistState(T state) async {
    await storage.write(persistenceKey, stateToJson(state));
  }

  /// Clears the persisted state.
  Future<void> clearPersistedState() async {
    await storage.delete(persistenceKey);
  }
}
