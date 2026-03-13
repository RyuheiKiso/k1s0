import 'dart:typed_data';
import 'package:test/test.dart';
import 'package:building_blocks/building_blocks.dart';

void main() {
  group('StateEntry', () => {
    test('should create with key, value, and etag', () {
      final entry = StateEntry(
        key: 'user:123',
        value: Uint8List.fromList([10, 20, 30]),
        etag: 'etag-abc',
      );

      expect(entry.key, 'user:123');
      expect(entry.value, Uint8List.fromList([10, 20, 30]));
      expect(entry.etag, 'etag-abc');
    });

    test('should support empty value and etag', () {
      final entry = StateEntry(
        key: 'empty',
        value: Uint8List(0),
        etag: '',
      );

      expect(entry.value.length, 0);
      expect(entry.etag, '');
    });

    test('should be immutable (final fields)', () {
      final entry = StateEntry(
        key: 'test',
        value: Uint8List.fromList([1]),
        etag: 'v1',
      );

      expect(entry.key, 'test');
      expect(entry.etag, 'v1');
    });
  });
}
