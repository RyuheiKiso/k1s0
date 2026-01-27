import 'package:hive_flutter/hive_flutter.dart';

import 'state_storage.dart';

/// Hive-based state storage implementation.
///
/// Provides fast key-value storage with optional encryption.
class HiveStorage implements StateStorage {
  HiveStorage._(this._box);

  /// Creates a HiveStorage from an existing box.
  factory HiveStorage.fromBox(Box<dynamic> box) => HiveStorage._(box);

  final Box<dynamic> _box;

  static bool _initialized = false;

  /// Initializes Hive. Must be called before creating HiveStorage instances.
  static Future<void> initialize({String? subDir}) async {
    if (_initialized) return;
    await Hive.initFlutter(subDir);
    _initialized = true;
  }

  /// Creates a new HiveStorage instance.
  ///
  /// [boxName] is the name of the Hive box to use.
  /// [encryptionCipher] is an optional encryption cipher.
  static Future<HiveStorage> create({
    required String boxName,
    HiveCipher? encryptionCipher,
  }) async {
    if (!_initialized) {
      await initialize();
    }

    final box = await Hive.openBox<dynamic>(
      boxName,
      encryptionCipher: encryptionCipher,
    );
    return HiveStorage._(box);
  }

  /// The underlying Hive box.
  Box<dynamic> get box => _box;

  @override
  Future<T?> read<T>(String key) async => _box.get(key) as T?;

  @override
  Future<void> write<T>(String key, T value) async {
    await _box.put(key, value);
  }

  @override
  Future<void> delete(String key) async {
    await _box.delete(key);
  }

  @override
  Future<void> clear() async {
    await _box.clear();
  }

  @override
  Future<bool> containsKey(String key) async => _box.containsKey(key);

  @override
  Future<List<String>> getKeys() async => _box.keys.cast<String>().toList();

  /// Closes the Hive box.
  Future<void> close() async {
    await _box.close();
  }

  /// Watches for changes to a key.
  Stream<BoxEvent> watch({String? key}) => _box.watch(key: key);
}

/// Typed storage wrapper for HiveStorage.
class TypedHiveStorage<T> {
  /// Creates a typed Hive storage.
  TypedHiveStorage({
    required this.storage,
    required this.key,
    this.defaultValue,
  });

  /// The underlying storage.
  final HiveStorage storage;

  /// The storage key.
  final String key;

  /// The default value.
  final T? defaultValue;

  /// Reads the value from storage.
  Future<T?> read() async {
    final value = await storage.read<T>(key);
    return value ?? defaultValue;
  }

  /// Writes the value to storage.
  Future<void> write(T value) async {
    await storage.write(key, value);
  }

  /// Deletes the value from storage.
  Future<void> delete() async {
    await storage.delete(key);
  }

  /// Watches for changes to the value.
  Stream<T?> watch() => storage.watch(key: key).map((event) {
        if (event.deleted) return null;
        return event.value as T?;
      });
}

/// Lazy box wrapper for large data.
class LazyHiveStorage implements StateStorage {
  LazyHiveStorage._(this._box);

  final LazyBox<dynamic> _box;

  /// Creates a new LazyHiveStorage instance.
  static Future<LazyHiveStorage> create({
    required String boxName,
    HiveCipher? encryptionCipher,
  }) async {
    if (!HiveStorage._initialized) {
      await HiveStorage.initialize();
    }

    final box = await Hive.openLazyBox<dynamic>(
      boxName,
      encryptionCipher: encryptionCipher,
    );
    return LazyHiveStorage._(box);
  }

  @override
  Future<T?> read<T>(String key) async => await _box.get(key) as T?;

  @override
  Future<void> write<T>(String key, T value) async {
    await _box.put(key, value);
  }

  @override
  Future<void> delete(String key) async {
    await _box.delete(key);
  }

  @override
  Future<void> clear() async {
    await _box.clear();
  }

  @override
  Future<bool> containsKey(String key) async => _box.containsKey(key);

  @override
  Future<List<String>> getKeys() async => _box.keys.cast<String>().toList();

  /// Closes the lazy box.
  Future<void> close() async {
    await _box.close();
  }
}
