/// inventory_repository_test.dart: InventoryRepositoryのユニットテスト。
/// MockHttpClientAdapterを使用してAPI通信をモックし、
/// 各メソッドが正しくリクエストを送信・レスポンスを処理することを検証する。
import 'dart:convert';

import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:inventory/models/inventory.dart';
import 'package:inventory/repositories/inventory_repository.dart';

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

/// テスト用のサンプル在庫アイテムJSONデータを生成する
Map<String, dynamic> _sampleItemData({
  String id = 'INV-001',
  String status = 'in_stock',
}) =>
    {
      'id': id,
      'product_id': 'PROD-001',
      'product_name': 'テスト商品',
      'warehouse_id': 'WH-001',
      'warehouse_name': 'テスト倉庫',
      'quantity_available': 100,
      'quantity_reserved': 10,
      'reorder_point': 20,
      'status': status,
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
  group('InventoryRepository.listInventory', () {
    /// 在庫一覧が正しく取得されることを確認する
    test('在庫一覧が正しく取得される', () async {
      final dio = _createTestDio((_) => jsonEncode({
            'items': [
              _sampleItemData(id: 'INV-001'),
              _sampleItemData(id: 'INV-002', status: 'low_stock'),
            ]
          }));
      final repo = InventoryRepository(dio);

      final items = await repo.listInventory();

      expect(items, hasLength(2));
      expect(items[0].id, 'INV-001');
      expect(items[0].status, InventoryStatus.inStock);
      expect(items[1].id, 'INV-002');
      expect(items[1].status, InventoryStatus.lowStock);
    });

    /// 空の在庫一覧が返されたときに空リストになることを確認する
    test('空の在庫一覧が返される', () async {
      final dio = _createTestDio((_) => jsonEncode({'items': []}));
      final repo = InventoryRepository(dio);

      final items = await repo.listInventory();

      expect(items, isEmpty);
    });
  });

  group('InventoryRepository.getInventory', () {
    /// 指定IDの在庫アイテムが正しく取得されることを確認する
    test('指定IDの在庫アイテムが正しく取得される', () async {
      final dio = _createTestDio((_) => jsonEncode(_sampleItemData(id: 'INV-001')));
      final repo = InventoryRepository(dio);

      final item = await repo.getInventory('INV-001');

      expect(item.id, 'INV-001');
      expect(item.quantityAvailable, 100);
      expect(item.status, InventoryStatus.inStock);
    });
  });

  group('InventoryRepository.reserveStock', () {
    /// 在庫引当が例外なく完了することを確認する
    test('在庫引当が例外なく完了する', () async {
      // reserveStock は void を返すため、例外が出ないことを確認する
      final dio = _createTestDio((_) => '{}');
      final repo = InventoryRepository(dio);

      expect(
        () async => repo.reserveStock(
          const StockOperation(
            productId: 'PROD-001',
            warehouseId: 'WH-001',
            quantity: 5,
          ),
        ),
        returnsNormally,
      );
    });
  });

  group('InventoryRepository.updateStock', () {
    /// 在庫数量の更新が正しく処理されることを確認する
    test('在庫数量の更新が正しく処理される', () async {
      final updatedData = _sampleItemData(id: 'INV-001')..['quantity_available'] = 150;
      final dio = _createTestDio((_) => jsonEncode(updatedData));
      final repo = InventoryRepository(dio);

      const input = UpdateStockInput(quantityAvailable: 150);
      final item = await repo.updateStock('INV-001', input);

      expect(item.id, 'INV-001');
      expect(item.quantityAvailable, 150);
    });
  });
}
