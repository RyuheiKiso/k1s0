import 'package:test/test.dart';
import 'package:k1s0_search_client/search_client.dart';

void main() {
  late InMemorySearchClient client;

  setUp(() {
    client = InMemorySearchClient();
  });

  group('createIndex', () {
    test('creates an index', () async {
      await client.createIndex('products', IndexMapping());
      expect(client.documentCount('products'), equals(0));
    });
  });

  group('indexDocument', () {
    test('indexes a document', () async {
      await client.createIndex('products', IndexMapping());
      final doc = IndexDocument(
        id: 'p-1',
        fields: {'name': 'Rust Programming'},
      );
      final result = await client.indexDocument('products', doc);
      expect(result.id, equals('p-1'));
      expect(result.version, equals(1));
    });

    test('throws on missing index', () async {
      final doc = IndexDocument(id: '1', fields: {});
      expect(
        () => client.indexDocument('nonexistent', doc),
        throwsA(isA<SearchError>()),
      );
    });
  });

  group('bulkIndex', () {
    test('bulk indexes documents', () async {
      await client.createIndex('items', IndexMapping());
      final docs = [
        IndexDocument(id: 'i-1', fields: {'name': 'Item 1'}),
        IndexDocument(id: 'i-2', fields: {'name': 'Item 2'}),
      ];
      final result = await client.bulkIndex('items', docs);
      expect(result.successCount, equals(2));
      expect(result.failedCount, equals(0));
      expect(result.failures, isEmpty);
    });
  });

  group('search', () {
    test('searches documents', () async {
      await client.createIndex('products', IndexMapping());
      await client.indexDocument('products', IndexDocument(
        id: 'p-1',
        fields: {'name': 'Rust Programming'},
      ));
      await client.indexDocument('products', IndexDocument(
        id: 'p-2',
        fields: {'name': 'Go Language'},
      ));

      final result = await client.search('products', SearchQuery(
        query: 'Rust',
        facets: ['name'],
      ));
      expect(result.total, equals(1));
      expect(result.hits, hasLength(1));
      expect(result.facets, contains('name'));
    });

    test('throws on missing index', () async {
      expect(
        () => client.search('nonexistent', SearchQuery(query: 'test')),
        throwsA(isA<SearchError>()),
      );
    });

    test('empty query returns all', () async {
      await client.createIndex('items', IndexMapping());
      await client.indexDocument('items', IndexDocument(
        id: 'i-1',
        fields: {'name': 'Item'},
      ));
      final result = await client.search('items', SearchQuery(query: ''));
      expect(result.total, equals(1));
    });
  });

  group('deleteDocument', () {
    test('deletes a document', () async {
      await client.createIndex('products', IndexMapping());
      await client.indexDocument('products', IndexDocument(
        id: 'p-1',
        fields: {'name': 'Test'},
      ));
      await client.deleteDocument('products', 'p-1');
      expect(client.documentCount('products'), equals(0));
    });
  });

  group('Filter', () {
    test('eq factory', () {
      final f = Filter.eq('status', 'active');
      expect(f.operator, equals('eq'));
      expect(f.field, equals('status'));
    });

    test('range factory', () {
      final f = Filter.range('price', 10, 100);
      expect(f.operator, equals('range'));
      expect(f.valueTo, equals(100));
    });
  });

  group('SearchError', () {
    test('toString contains code', () {
      final err = SearchError('test', SearchErrorCode.indexNotFound);
      expect(err.toString(), contains('indexNotFound'));
    });
  });
}
