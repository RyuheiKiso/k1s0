import 'package:test/test.dart';

import 'package:k1s0_pagination/pagination.dart';

void main() {
  group('PageRequest', () {
    test('stores page and perPage', () {
      const req = PageRequest(page: 1, perPage: 10);
      expect(req.page, equals(1));
      expect(req.perPage, equals(10));
    });
  });

  group('PageResponse', () {
    test('creates with correct totalPages', () {
      const req = PageRequest(page: 1, perPage: 10);
      final resp = PageResponse<String>.create(['a', 'b'], 25, req);
      expect(resp.items, equals(['a', 'b']));
      expect(resp.total, equals(25));
      expect(resp.totalPages, equals(3));
    });

    test('handles exact division', () {
      const req = PageRequest(page: 1, perPage: 5);
      final resp = PageResponse<int>.create([1, 2, 3, 4, 5], 10, req);
      expect(resp.totalPages, equals(2));
    });

    test('handles empty items', () {
      const req = PageRequest(page: 1, perPage: 10);
      final resp = PageResponse<String>.create([], 0, req);
      expect(resp.totalPages, equals(0));
      expect(resp.items, isEmpty);
    });
  });

  group('cursor', () {
    test('encode and decode round-trip', () {
      const id = 'abc-123';
      final cursor = encodeCursor(id);
      expect(decodeCursor(cursor), equals(id));
    });

    test('produces base64url string', () {
      final cursor = encodeCursor('test-id');
      expect(cursor, isNotEmpty);
      expect(cursor, isNot(contains(' ')));
    });
  });
}
