import 'dart:typed_data';
import 'package:test/test.dart';
import 'package:building_blocks/building_blocks.dart';

void main() {
  group('Message', () => {
    test('should create with all required fields', () {
      final msg = Message(
        topic: 'orders',
        data: Uint8List.fromList([1, 2, 3]),
        metadata: {'source': 'test'},
        id: 'msg-001',
      );

      expect(msg.topic, 'orders');
      expect(msg.data, Uint8List.fromList([1, 2, 3]));
      expect(msg.metadata, {'source': 'test'});
      expect(msg.id, 'msg-001');
    });

    test('should support empty data and metadata', () {
      final msg = Message(
        topic: 'events',
        data: Uint8List(0),
        metadata: const {},
        id: 'msg-002',
      );

      expect(msg.data.length, 0);
      expect(msg.metadata, isEmpty);
    });

    test('should be immutable (final fields)', () {
      final msg = Message(
        topic: 'test',
        data: Uint8List.fromList([42]),
        metadata: const {},
        id: 'msg-003',
      );

      // Fields are final, so these should retain their values
      expect(msg.topic, 'test');
      expect(msg.id, 'msg-003');
    });
  });
}
