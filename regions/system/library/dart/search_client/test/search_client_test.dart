import 'package:test/test.dart';
import 'package:k1s0_search_client/search_client.dart';

void main() {
  late InMemorySearchClient client;

  setUp(() {
    client = InMemorySearchClient();
  });

  group('createIndex', () {
    test('インデックスが作成されること', () async {
      await client.createIndex('products', IndexMapping());
      expect(client.documentCount('products'), equals(0));
    });
  });

  group('indexDocument', () {
    test('ドキュメントがインデックスされること', () async {
      await client.createIndex('products', IndexMapping());
      const doc = IndexDocument(
        id: 'p-1',
        fields: {'name': 'Rust Programming'},
      );
      final result = await client.indexDocument('products', doc);
      expect(result.id, equals('p-1'));
      expect(result.version, equals(1));
    });

    test('インデックスが存在しない場合に例外がスローされること', () async {
      const doc = IndexDocument(id: '1', fields: {});
      expect(
        () => client.indexDocument('nonexistent', doc),
        throwsA(isA<SearchError>()),
      );
    });
  });

  group('bulkIndex', () {
    test('ドキュメントが一括インデックスされること', () async {
      await client.createIndex('items', IndexMapping());
      final docs = [
        const IndexDocument(id: 'i-1', fields: {'name': 'Item 1'}),
        const IndexDocument(id: 'i-2', fields: {'name': 'Item 2'}),
      ];
      final result = await client.bulkIndex('items', docs);
      expect(result.successCount, equals(2));
      expect(result.failedCount, equals(0));
      expect(result.failures, isEmpty);
    });
  });

  group('search', () {
    test('ドキュメントが検索できること', () async {
      await client.createIndex('products', IndexMapping());
      await client.indexDocument('products', const IndexDocument(
        id: 'p-1',
        fields: {'name': 'Rust Programming'},
      ));
      await client.indexDocument('products', const IndexDocument(
        id: 'p-2',
        fields: {'name': 'Go Language'},
      ));

      final result = await client.search('products', const SearchQuery(
        query: 'Rust',
        facets: ['name'],
      ));
      expect(result.total, equals(1));
      expect(result.hits, hasLength(1));
      expect(result.facets, contains('name'));
    });

    test('インデックスが存在しない場合に例外がスローされること', () async {
      expect(
        () => client.search('nonexistent', const SearchQuery(query: 'test')),
        throwsA(isA<SearchError>()),
      );
    });

    test('空のクエリで全件が返ること', () async {
      await client.createIndex('items', IndexMapping());
      await client.indexDocument('items', const IndexDocument(
        id: 'i-1',
        fields: {'name': 'Item'},
      ));
      final result = await client.search('items', const SearchQuery(query: ''));
      expect(result.total, equals(1));
    });
  });

  group('deleteDocument', () {
    test('ドキュメントが削除されること', () async {
      await client.createIndex('products', IndexMapping());
      await client.indexDocument('products', const IndexDocument(
        id: 'p-1',
        fields: {'name': 'Test'},
      ));
      await client.deleteDocument('products', 'p-1');
      expect(client.documentCount('products'), equals(0));
    });
  });

  group('Filter', () {
    test('eqファクトリが正しく動作すること', () {
      final f = Filter.eq('status', 'active');
      expect(f.operator, equals('eq'));
      expect(f.field, equals('status'));
    });

    test('rangeファクトリが正しく動作すること', () {
      final f = Filter.range('price', 10, 100);
      expect(f.operator, equals('range'));
      expect(f.valueTo, equals(100));
    });
  });

  group('SearchError', () {
    test('toStringにエラーコードが含まれること', () {
      const err = SearchError('test', SearchErrorCode.indexNotFound);
      expect(err.toString(), contains('indexNotFound'));
    });
  });
}
