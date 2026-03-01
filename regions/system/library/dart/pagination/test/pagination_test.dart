import 'package:test/test.dart';

import 'package:k1s0_pagination/pagination.dart';

void main() {
  group('PageRequest', () {
    test('stores page and perPage', () {
      const req = PageRequest(page: 1, perPage: 10);
      expect(req.page, equals(1));
      expect(req.perPage, equals(10));
    });

    test('defaultRequest returns page=1, perPage=20', () {
      final req = PageRequest.defaultRequest();
      expect(req.page, equals(1));
      expect(req.perPage, equals(20));
    });

    test('offset is (page-1) * perPage', () {
      const req1 = PageRequest(page: 1, perPage: 10);
      expect(req1.offset, equals(0));

      const req2 = PageRequest(page: 3, perPage: 10);
      expect(req2.offset, equals(20));

      const req3 = PageRequest(page: 2, perPage: 25);
      expect(req3.offset, equals(25));
    });

    test('hasNext returns true when more pages exist', () {
      const req = PageRequest(page: 1, perPage: 10);
      expect(req.hasNext(25), isTrue);
      expect(req.hasNext(10), isFalse);
      expect(req.hasNext(11), isTrue);
    });

    test('hasNext returns false on last page', () {
      const req = PageRequest(page: 3, perPage: 10);
      expect(req.hasNext(30), isFalse);
      expect(req.hasNext(31), isTrue);
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

    test('meta returns PaginationMeta', () {
      const req = PageRequest(page: 2, perPage: 10);
      final resp = PageResponse<String>.create(['a'], 25, req);
      final meta = resp.meta;
      expect(meta.total, equals(25));
      expect(meta.page, equals(2));
      expect(meta.perPage, equals(10));
      expect(meta.totalPages, equals(3));
    });
  });

  group('cursor', () {
    test('encode and decode round-trip', () {
      const sortKey = '2024-01-15';
      const id = 'abc-123';
      final cursor = encodeCursor(sortKey, id);
      final result = decodeCursor(cursor);
      expect(result.sortKey, equals(sortKey));
      expect(result.id, equals(id));
    });

    test('produces base64url string', () {
      final cursor = encodeCursor('key', 'test-id');
      expect(cursor, isNotEmpty);
      expect(cursor, isNot(contains(' ')));
    });

    test('CursorRequest stores fields', () {
      const req = CursorRequest(cursor: 'abc', limit: 20);
      expect(req.cursor, equals('abc'));
      expect(req.limit, equals(20));
    });

    test('CursorMeta stores fields', () {
      const meta = CursorMeta(nextCursor: 'next', hasMore: true);
      expect(meta.nextCursor, equals('next'));
      expect(meta.hasMore, isTrue);
    });
  });

  group('validatePerPage', () {
    test('accepts valid values', () {
      expect(validatePerPage(1), equals(1));
      expect(validatePerPage(50), equals(50));
      expect(validatePerPage(100), equals(100));
    });

    test('rejects zero', () {
      expect(() => validatePerPage(0), throwsA(isA<PerPageValidationException>()));
    });

    test('rejects over max', () {
      expect(() => validatePerPage(101), throwsA(isA<PerPageValidationException>()));
    });
  });
}
