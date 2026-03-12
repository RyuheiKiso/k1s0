import 'dart:typed_data';
import 'component.dart';

class StateEntry {
  final String key;
  final Uint8List value;
  final String etag;

  const StateEntry({
    required this.key,
    required this.value,
    required this.etag,
  });
}

abstract class StateStore implements Component {
  Future<StateEntry?> get(String key);
  Future<String> set(String key, Uint8List value, {String? etag});
  Future<void> delete(String key, {String? etag});
  Future<List<StateEntry>> bulkGet(List<String> keys);
  Future<List<String>> bulkSet(List<StateEntry> entries);
}
