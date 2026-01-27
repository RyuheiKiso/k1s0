import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

import 'state_storage.dart';

/// SharedPreferences-based state storage implementation.
class PreferencesStorage implements StateStorage {
  PreferencesStorage._(this._prefs);

  /// Creates a PreferencesStorage from an existing SharedPreferences instance.
  factory PreferencesStorage.fromPrefs(SharedPreferences prefs) =>
      PreferencesStorage._(prefs);

  final SharedPreferences _prefs;

  /// Creates a new PreferencesStorage instance.
  static Future<PreferencesStorage> create() async {
    final prefs = await SharedPreferences.getInstance();
    return PreferencesStorage._(prefs);
  }

  @override
  Future<T?> read<T>(String key) async {
    final value = _prefs.get(key);
    if (value == null) return null;

    if (T == String) {
      return value as T;
    } else if (T == int) {
      return value as T;
    } else if (T == double) {
      return value as T;
    } else if (T == bool) {
      return value as T;
    } else if (T == List<String>) {
      return _prefs.getStringList(key) as T?;
    } else {
      // For complex types, try to decode as JSON
      if (value is String) {
        try {
          return jsonDecode(value) as T;
        } on FormatException {
          return null;
        } on Exception {
          return null;
        }
      }
      return null;
    }
  }

  @override
  Future<void> write<T>(String key, T value) async {
    if (value is String) {
      await _prefs.setString(key, value);
    } else if (value is int) {
      await _prefs.setInt(key, value);
    } else if (value is double) {
      await _prefs.setDouble(key, value);
    } else if (value is bool) {
      await _prefs.setBool(key, value);
    } else if (value is List<String>) {
      await _prefs.setStringList(key, value);
    } else {
      // For complex types, encode as JSON
      await _prefs.setString(key, jsonEncode(value));
    }
  }

  @override
  Future<void> delete(String key) async {
    await _prefs.remove(key);
  }

  @override
  Future<void> clear() async {
    await _prefs.clear();
  }

  @override
  Future<bool> containsKey(String key) async => _prefs.containsKey(key);

  @override
  Future<List<String>> getKeys() async => _prefs.getKeys().toList();
}

/// Typed storage wrapper for PreferencesStorage.
class TypedPreferencesStorage<T> {
  /// Creates a typed preferences storage.
  TypedPreferencesStorage({
    required this.storage,
    required this.key,
    required this.serializer,
    this.defaultValue,
  });

  /// The underlying storage.
  final PreferencesStorage storage;

  /// The storage key.
  final String key;

  /// The serializer for converting state to/from JSON.
  final StateSerializer<T> serializer;

  /// The default value.
  final T? defaultValue;

  /// Reads the value from storage.
  Future<T?> read() async {
    final json = await storage.read<Map<String, dynamic>>(key);
    if (json == null) return defaultValue;
    return serializer.fromJson(json);
  }

  /// Writes the value to storage.
  Future<void> write(T value) async {
    final json = serializer.toJson(value);
    await storage.write(key, json);
  }

  /// Deletes the value from storage.
  Future<void> delete() async {
    await storage.delete(key);
  }
}
