import 'dart:typed_data';
import 'package:test/test.dart';
import 'package:k1s0_building_blocks/building_blocks.dart';

void main() {
  group('BindingData', () {
    test('should create with data and metadata', () {
      final bd = BindingData(
        data: Uint8List.fromList([1, 2, 3]),
        metadata: {'source': 'queue'},
      );

      expect(bd.data, Uint8List.fromList([1, 2, 3]));
      expect(bd.metadata, {'source': 'queue'});
    });

    test('should support empty data and metadata', () {
      final bd = BindingData(
        data: Uint8List(0),
        metadata: const {},
      );

      expect(bd.data.length, 0);
      expect(bd.metadata, isEmpty);
    });
  });

  group('BindingResponse', () {
    test('should create with data and metadata', () {
      final resp = BindingResponse(
        data: Uint8List.fromList([10, 20]),
        metadata: {'status': 'ok'},
      );

      expect(resp.data, Uint8List.fromList([10, 20]));
      expect(resp.metadata, {'status': 'ok'});
    });

    test('should support empty response', () {
      final resp = BindingResponse(
        data: Uint8List(0),
        metadata: const {},
      );

      expect(resp.data.length, 0);
      expect(resp.metadata, isEmpty);
    });
  });
}
