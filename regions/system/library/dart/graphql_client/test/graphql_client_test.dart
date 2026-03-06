import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:http/testing.dart' as http_testing;
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

  group('ClientError', () {
    test('request variant', () {
      final err = ClientError.request('connection refused');
      expect(err.kind, equals(ClientErrorKind.request));
      expect(err.message, equals('connection refused'));
      expect(err.toString(), equals('RequestError: connection refused'));
    });

    test('deserialization variant', () {
      final err = ClientError.deserialization('invalid json');
      expect(err.kind, equals(ClientErrorKind.deserialization));
      expect(
          err.toString(), equals('DeserializationError: invalid json'));
    });

    test('graphQl variant', () {
      final err = ClientError.graphQl('field not found');
      expect(err.kind, equals(ClientErrorKind.graphQl));
      expect(err.toString(), equals('GraphQlError: field not found'));
    });

    test('notFound variant', () {
      final err = ClientError.notFound('user 123');
      expect(err.kind, equals(ClientErrorKind.notFound));
      expect(err.toString(), equals('NotFoundError: user 123'));
    });

    test('is an Exception', () {
      final err = ClientError.request('test');
      expect(err, isA<Exception>());
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

  group('GraphQlHttpClient', () {
    http_testing.MockClient mockClient(
      Future<http.Response> Function(http.Request) handler,
    ) {
      return http_testing.MockClient(handler);
    }

    test('execute sends POST and parses response', () async {
      final mock = mockClient((request) async {
        expect(request.method, equals('POST'));
        expect(request.headers['Content-Type'], equals('application/json'));
        final body = jsonDecode(request.body) as Map<String, dynamic>;
        expect(body['query'], equals('{ users { id } }'));
        return http.Response(
          jsonEncode({
            'data': {'id': '1', 'name': 'Alice'},
          }),
          200,
        );
      });

      final httpClient = GraphQlHttpClient(
        'http://localhost:8080/graphql',
        httpClient: mock,
      );

      final result = await httpClient.execute(
        const GraphQlQuery(query: '{ users { id } }'),
        (json) => json,
      );

      expect(result.hasErrors, isFalse);
      expect(result.data?['name'], equals('Alice'));
    });

    test('execute sends variables and operationName', () async {
      final mock = mockClient((request) async {
        final body = jsonDecode(request.body) as Map<String, dynamic>;
        expect(body['variables'], equals({'id': '1'}));
        expect(body['operationName'], equals('GetUser'));
        return http.Response(
          jsonEncode({
            'data': {'id': '1'},
          }),
          200,
        );
      });

      final httpClient = GraphQlHttpClient(
        'http://localhost:8080/graphql',
        httpClient: mock,
      );

      await httpClient.execute(
        const GraphQlQuery(
          query: 'query GetUser(\$id: ID!) { user(id: \$id) { id } }',
          variables: {'id': '1'},
          operationName: 'GetUser',
        ),
        (json) => json,
      );
    });

    test('execute passes custom headers', () async {
      final mock = mockClient((request) async {
        expect(request.headers['Authorization'], equals('Bearer token'));
        return http.Response(
          jsonEncode({
            'data': {'ok': true},
          }),
          200,
        );
      });

      final httpClient = GraphQlHttpClient(
        'http://localhost:8080/graphql',
        headers: {'Authorization': 'Bearer token'},
        httpClient: mock,
      );

      await httpClient.execute(
        const GraphQlQuery(query: '{ me { id } }'),
        (json) => json,
      );
    });

    test('execute throws ClientError.notFound on 404', () async {
      final mock = mockClient((request) async {
        return http.Response('Not Found', 404);
      });

      final httpClient = GraphQlHttpClient(
        'http://localhost:8080/graphql',
        httpClient: mock,
      );

      expect(
        () => httpClient.execute(
          const GraphQlQuery(query: '{ users { id } }'),
          (json) => json,
        ),
        throwsA(isA<ClientError>().having(
          (e) => e.kind,
          'kind',
          ClientErrorKind.notFound,
        )),
      );
    });

    test('execute throws ClientError.request on 500', () async {
      final mock = mockClient((request) async {
        return http.Response('Internal Server Error', 500);
      });

      final httpClient = GraphQlHttpClient(
        'http://localhost:8080/graphql',
        httpClient: mock,
      );

      expect(
        () => httpClient.execute(
          const GraphQlQuery(query: '{ users { id } }'),
          (json) => json,
        ),
        throwsA(isA<ClientError>().having(
          (e) => e.kind,
          'kind',
          ClientErrorKind.request,
        )),
      );
    });

    test('execute throws ClientError.deserialization on invalid JSON',
        () async {
      final mock = mockClient((request) async {
        return http.Response('not json', 200);
      });

      final httpClient = GraphQlHttpClient(
        'http://localhost:8080/graphql',
        httpClient: mock,
      );

      expect(
        () => httpClient.execute(
          const GraphQlQuery(query: '{ users { id } }'),
          (json) => json,
        ),
        throwsA(isA<ClientError>().having(
          (e) => e.kind,
          'kind',
          ClientErrorKind.deserialization,
        )),
      );
    });

    test('execute returns GraphQL errors from response', () async {
      final mock = mockClient((request) async {
        return http.Response(
          jsonEncode({
            'errors': [
              {
                'message': 'Unauthorized',
                'locations': [
                  {'line': 1, 'column': 3}
                ],
                'path': ['user'],
              }
            ],
          }),
          200,
        );
      });

      final httpClient = GraphQlHttpClient(
        'http://localhost:8080/graphql',
        httpClient: mock,
      );

      final result = await httpClient.execute(
        const GraphQlQuery(query: '{ user { id } }'),
        (json) => json,
      );

      expect(result.hasErrors, isTrue);
      expect(result.errors!.first.message, equals('Unauthorized'));
      expect(result.errors!.first.locations, hasLength(1));
      expect(result.errors!.first.locations!.first.line, equals(1));
      expect(result.errors!.first.path, equals(['user']));
    });

    test('executeMutation works the same as execute', () async {
      final mock = mockClient((request) async {
        return http.Response(
          jsonEncode({
            'data': {'id': '2'},
          }),
          200,
        );
      });

      final httpClient = GraphQlHttpClient(
        'http://localhost:8080/graphql',
        httpClient: mock,
      );

      final result = await httpClient.executeMutation(
        const GraphQlQuery(
          query: 'mutation { createUser { id } }',
          operationName: 'CreateUser',
        ),
        (json) => json,
      );

      expect(result.hasErrors, isFalse);
      expect(result.data?['id'], equals('2'));
    });

    test('subscribe throws ClientError.request', () {
      final httpClient = GraphQlHttpClient('http://localhost:8080/graphql');

      expect(
        () => httpClient.subscribe(
          const GraphQlQuery(query: 'subscription { onEvent { id } }'),
          (json) => json,
        ),
        throwsA(isA<ClientError>().having(
          (e) => e.kind,
          'kind',
          ClientErrorKind.request,
        )),
      );
    });

    test('execute throws ClientError.deserialization when data is null',
        () async {
      final mock = mockClient((request) async {
        return http.Response(jsonEncode({}), 200);
      });

      final httpClient = GraphQlHttpClient(
        'http://localhost:8080/graphql',
        httpClient: mock,
      );

      expect(
        () => httpClient.execute(
          const GraphQlQuery(query: '{ users { id } }'),
          (json) => json,
        ),
        throwsA(isA<ClientError>().having(
          (e) => e.kind,
          'kind',
          ClientErrorKind.deserialization,
        )),
      );
    });
  });
}
