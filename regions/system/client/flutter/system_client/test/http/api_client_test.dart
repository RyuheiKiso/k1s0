import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:system_client/system_client.dart';

void main() {
  group('ApiClient', () {
    test('Dio インスタンスを返す', () {
      final client = ApiClient.create(baseUrl: 'https://api.example.com');
      expect(client, isA<Dio>());
    });

    test('baseUrl が設定される', () {
      final client = ApiClient.create(baseUrl: 'https://api.example.com');
      expect(client.options.baseUrl, equals('https://api.example.com'));
    });

    test('Content-Type ヘッダーが設定される', () {
      final client = ApiClient.create(baseUrl: 'https://api.example.com');
      expect(
        client.options.headers['Content-Type'],
        equals('application/json'),
      );
    });

    test('インターセプターが設定される', () {
      final client = ApiClient.create(baseUrl: 'https://api.example.com');
      expect(client.interceptors.isNotEmpty, isTrue);
    });
  });
}
