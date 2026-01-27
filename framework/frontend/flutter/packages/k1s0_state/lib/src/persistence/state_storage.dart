/// Abstract interface for state storage.
///
/// Implementations can use SharedPreferences, Hive, or other storage mechanisms.
abstract class StateStorage {
  /// Reads a value from storage.
  Future<T?> read<T>(String key);

  /// Writes a value to storage.
  Future<void> write<T>(String key, T value);

  /// Deletes a value from storage.
  Future<void> delete(String key);

  /// Clears all values from storage.
  Future<void> clear();

  /// Checks if a key exists in storage.
  Future<bool> containsKey(String key);

  /// Gets all keys in storage.
  Future<List<String>> getKeys();
}

/// A serializer for converting values to and from JSON.
abstract class StateSerializer<T> {
  /// Serializes a value to JSON.
  Map<String, dynamic> toJson(T value);

  /// Deserializes a value from JSON.
  T fromJson(Map<String, dynamic> json);
}

/// A simple string serializer.
class StringSerializer implements StateSerializer<String> {
  const StringSerializer();

  @override
  Map<String, dynamic> toJson(String value) => {'value': value};

  @override
  String fromJson(Map<String, dynamic> json) => json['value'] as String;
}

/// A simple int serializer.
class IntSerializer implements StateSerializer<int> {
  const IntSerializer();

  @override
  Map<String, dynamic> toJson(int value) => {'value': value};

  @override
  int fromJson(Map<String, dynamic> json) => json['value'] as int;
}

/// A simple double serializer.
class DoubleSerializer implements StateSerializer<double> {
  const DoubleSerializer();

  @override
  Map<String, dynamic> toJson(double value) => {'value': value};

  @override
  double fromJson(Map<String, dynamic> json) =>
      (json['value'] as num).toDouble();
}

/// A simple bool serializer.
class BoolSerializer implements StateSerializer<bool> {
  const BoolSerializer();

  @override
  Map<String, dynamic> toJson(bool value) => {'value': value};

  @override
  bool fromJson(Map<String, dynamic> json) => json['value'] as bool;
}

/// A list serializer.
class ListSerializer<T> implements StateSerializer<List<T>> {
  const ListSerializer(this.itemSerializer);

  final StateSerializer<T> itemSerializer;

  @override
  Map<String, dynamic> toJson(List<T> value) => {
        'items': value.map((e) => itemSerializer.toJson(e)).toList(),
      };

  @override
  List<T> fromJson(Map<String, dynamic> json) {
    final items = json['items'] as List<dynamic>;
    return items
        .map((e) => itemSerializer.fromJson(e as Map<String, dynamic>))
        .toList();
  }
}

/// A map serializer.
class MapSerializer<V> implements StateSerializer<Map<String, V>> {
  const MapSerializer(this.valueSerializer);

  final StateSerializer<V> valueSerializer;

  @override
  Map<String, dynamic> toJson(Map<String, V> value) => {
        'entries': value.map(
          (k, v) => MapEntry(k, valueSerializer.toJson(v)),
        ),
      };

  @override
  Map<String, V> fromJson(Map<String, dynamic> json) {
    final entries = json['entries'] as Map<String, dynamic>;
    return entries.map(
      (k, v) => MapEntry(k, valueSerializer.fromJson(v as Map<String, dynamic>)),
    );
  }
}
