import 'package:test/test.dart';
import 'package:k1s0_graphql_client/graphql_client.dart';

void main() {
  late InMemoryGraphQlClient client;

  setUp(() {
    client = InMemoryGraphQlClient();
  });

  group('GraphQlQuery', () {
    test('creates with required fields', () {
      const q = GraphQlQuery(query: '{ users { id } }');
      expect(q.query, equals('{ users { id } }'));
      expect(q.variables, isNull);
      expect(q.operationName, isNull);
    });

    test('creates with all fields', () {
      const q = GraphQlQuery(
        query: 'query GetUser(\$id: ID!) { user(id: \$id) { name } }',
        variables: {'id': '1'},
        operationName: 'GetUser',
      );
      expect(q.operationName, equals('GetUser'));
      expect(q.variables, equals({'id': '1'}));
    });
  });

  group('GraphQlResponse', () {
    test('hasErrors is false when no errors', () {
      const resp = GraphQlResponse<String>(data: 'ok');
      expect(resp.hasErrors, isFalse);
    });

    test('hasErrors is true with errors', () {
      const resp = GraphQlResponse<String>(
        errors: [GraphQlError(message: 'fail')],
      );
      expect(resp.hasErrors, isTrue);
    });

    test('hasErrors is false with empty list', () {
      const resp = GraphQlResponse<String>(errors: []);
      expect(resp.hasErrors, isFalse);
    });
  });

  group('ErrorLocation', () {
    test('stores line and column', () {
      const loc = ErrorLocation(3, 5);
      expect(loc.line, equals(3));
      expect(loc.column, equals(5));
    });
  });

  group('GraphQlError', () {
    test('creates with message', () {
      const err = GraphQlError(message: 'Not found');
      expect(err.message, equals('Not found'));
      expect(err.locations, isNull);
      expect(err.path, isNull);
    });

    test('creates with locations and path', () {
      const err = GraphQlError(
        message: 'err',
        locations: [ErrorLocation(1, 2)],
        path: ['user', 0, 'name'],
      );
      expect(err.locations, hasLength(1));
      expect(err.path, hasLength(3));
    });
  });

  group('InMemoryGraphQlClient', () {
    test('execute returns configured response', () async {
      client.setResponse('GetUser', {
        'data': {'id': '1', 'name': 'Alice'},
      });
      final result = await client.execute(
        const GraphQlQuery(
          query: 'query GetUser { user { id name } }',
          operationName: 'GetUser',
        ),
        (json) => json,
      );
      expect(result.hasErrors, isFalse);
      expect(result.data?['name'], equals('Alice'));
    });

    test('execute returns error for unconfigured operation', () async {
      final result = await client.execute(
        const GraphQlQuery(query: '{ unknown }', operationName: 'Unknown'),
        (json) => json,
      );
      expect(result.hasErrors, isTrue);
      expect(result.data, isNull);
    });

    test('executeMutation returns configured response', () async {
      client.setResponse('CreateUser', {
        'data': {'id': '2', 'name': 'Bob'},
      });
      final result = await client.executeMutation(
        const GraphQlQuery(
          query: 'mutation CreateUser { createUser { id name } }',
          operationName: 'CreateUser',
        ),
        (json) => json,
      );
      expect(result.hasErrors, isFalse);
      expect(result.data?['id'], equals('2'));
    });

    test('execute falls back to query text when no operationName', () async {
      client.setResponse('{ me { id } }', {
        'data': {'id': '42'},
      });
      final result = await client.execute(
        const GraphQlQuery(query: '{ me { id } }'),
        (json) => json,
      );
      expect(result.data?['id'], equals('42'));
    });

    test('returns errors from response', () async {
      client.setResponse('Fail', {
        'errors': [
          {'message': 'Unauthorized'},
        ],
      });
      final result = await client.execute(
        const GraphQlQuery(query: 'query Fail { fail }', operationName: 'Fail'),
        (json) => json,
      );
      expect(result.hasErrors, isTrue);
      expect(result.errors!.first.message, equals('Unauthorized'));
      expect(result.data, isNull);
    });

    test('subscribe emits registered events', () async {
      client.setSubscriptionEvents('OnUserCreated', [
        {'id': '1', 'name': 'Alice'},
        {'id': '2', 'name': 'Bob'},
      ]);

      const subscription = GraphQlQuery(
        query: 'subscription { userCreated { id name } }',
        operationName: 'OnUserCreated',
      );

      final results =
          await client.subscribe(subscription, (json) => json).toList();
      expect(results, hasLength(2));
      expect(results[0].data, isNotNull);
      expect(results[1].data, isNotNull);
    });
  });
}
