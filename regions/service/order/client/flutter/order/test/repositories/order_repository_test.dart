/// order_repository_test.dart: OrderRepositoryのユニットテスト。
/// MockHttpClientAdapterを使用してAPI通信をモックし、
/// 各メソッドが正しくリクエストを送信・レスポンスを処理することを検証する。
import 'dart:convert';

import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:order/models/order.dart';
import 'package:order/repositories/order_repository.dart';

/// テスト用のHTTPクライアントアダプター
class _MockHttpClientAdapter implements HttpClientAdapter {
  final String Function(RequestOptions) responseBodyFn;
  final int statusCode;

  _MockHttpClientAdapter({
    required this.responseBodyFn,
    this.statusCode = 200,
  });

  @override
  Future<ResponseBody> fetch(
    RequestOptions options,
    Stream<List<int>>? requestStream,
    Future<void>? cancelFuture,
  ) async {
    return ResponseBody.fromString(
      responseBodyFn(options),
      statusCode,
      headers: {
        'content-type': ['application/json'],
      },
    );
  }

  @override
  void close({bool force = false}) {}
}

/// テスト用のサンプル注文JSONデータを生成する
Map<String, dynamic> _sampleOrderData({
  String id = 'ORDER-001',
  String status = 'pending',
}) =>
    {
      'id': id,
      'customer_id': 'CUST-001',
      'status': status,
      'total_amount': 10000.0,
      'currency': 'JPY',
      'items': [
        {
          'product_id': 'PROD-001',
          'product_name': 'テスト商品',
          'quantity': 2,
          'unit_price': 5000.0,
          'subtotal': 10000.0,
        }
      ],
      'notes': null,
      'version': 1,
      'created_at': '2024-01-20T00:00:00.000Z',
      'updated_at': '2024-01-20T00:00:00.000Z',
    };

/// テスト用のDioインスタンスを生成する
Dio _createTestDio(String Function(RequestOptions) responseBodyFn) {
  final dio = Dio(BaseOptions(baseUrl: 'http://localhost:8080'));
  dio.httpClientAdapter = _MockHttpClientAdapter(responseBodyFn: responseBodyFn);
  return dio;
}

void main() {
  group('OrderRepository.listOrders', () {
    /// 注文一覧が正しく取得されることを確認する
    test('注文一覧が正しく取得される', () async {
      final dio = _createTestDio((_) => jsonEncode({
            'orders': [
              _sampleOrderData(id: 'ORDER-001'),
              _sampleOrderData(id: 'ORDER-002', status: 'confirmed'),
            ]
          }));
      final repo = OrderRepository(dio);

      final orders = await repo.listOrders();

      expect(orders, hasLength(2));
      expect(orders[0].id, 'ORDER-001');
      expect(orders[1].id, 'ORDER-002');
      expect(orders[1].status, OrderStatus.confirmed);
    });

    /// 空の注文一覧が返されたときに空リストになることを確認する
    test('空の注文一覧が返される', () async {
      final dio = _createTestDio((_) => jsonEncode({'orders': []}));
      final repo = OrderRepository(dio);

      final orders = await repo.listOrders();

      expect(orders, isEmpty);
    });
  });

  group('OrderRepository.getOrder', () {
    /// 指定IDの注文が正しく取得されることを確認する
    test('指定IDの注文が正しく取得される', () async {
      final dio = _createTestDio((_) => jsonEncode(_sampleOrderData(id: 'ORDER-001')));
      final repo = OrderRepository(dio);

      final order = await repo.getOrder('ORDER-001');

      expect(order.id, 'ORDER-001');
      expect(order.status, OrderStatus.pending);
      expect(order.items, hasLength(1));
    });
  });

  group('OrderRepository.createOrder', () {
    /// 新規注文が正しく作成されることを確認する
    test('新規注文が正しく作成される', () async {
      final dio = _createTestDio((_) => jsonEncode(_sampleOrderData(id: 'ORDER-NEW')));
      final repo = OrderRepository(dio);

      final input = CreateOrderInput(
        customerId: 'CUST-001',
        currency: 'JPY',
        items: [
          const CreateOrderItemInput(
            productId: 'PROD-001',
            productName: 'テスト商品',
            quantity: 2,
            unitPrice: 5000.0,
          ),
        ],
      );
      final order = await repo.createOrder(input);

      expect(order.id, 'ORDER-NEW');
    });
  });

  group('OrderRepository.updateOrderStatus', () {
    /// 注文ステータスが正しく更新されることを確認する
    test('注文ステータスが正しく更新される', () async {
      final dio = _createTestDio(
          (_) => jsonEncode(_sampleOrderData(id: 'ORDER-001', status: 'confirmed')));
      final repo = OrderRepository(dio);

      const input = UpdateOrderStatusInput(status: OrderStatus.confirmed);
      final order = await repo.updateOrderStatus('ORDER-001', input);

      expect(order.id, 'ORDER-001');
      expect(order.status, OrderStatus.confirmed);
    });
  });
}
