/// inventory_test.dart: InventoryItemモデルのユニットテスト。
/// fromJson/toJsonの往復変換、enum変換（スネークケース変換含む）を検証する。
import 'package:flutter_test/flutter_test.dart';
import 'package:inventory/models/inventory.dart';

/// テスト用の基本的なInventoryItem JSONデータ
Map<String, dynamic> _sampleInventoryJson() => {
      'id': '550e8400-e29b-41d4-a716-446655440001',
      'product_id': 'PROD-001',
      'product_name': 'テスト商品',
      'warehouse_id': 'WH-001',
      'warehouse_name': 'テスト倉庫',
      'quantity_available': 100,
      'quantity_reserved': 10,
      'reorder_point': 20,
      'status': 'in_stock',
      'version': 1,
      'created_at': '2024-01-20T00:00:00.000Z',
      'updated_at': '2024-01-20T00:00:00.000Z',
    };

void main() {
  group('InventoryItem.fromJson', () {
    /// 完全なJSONデータが正しくInventoryItemインスタンスに変換されることを確認する
    test('必須フィールドが正しくパースされる', () {
      final item = InventoryItem.fromJson(_sampleInventoryJson());

      expect(item.id, '550e8400-e29b-41d4-a716-446655440001');
      expect(item.productId, 'PROD-001');
      expect(item.productName, 'テスト商品');
      expect(item.warehouseId, 'WH-001');
      expect(item.warehouseName, 'テスト倉庫');
      expect(item.quantityAvailable, 100);
      expect(item.quantityReserved, 10);
      expect(item.reorderPoint, 20);
      expect(item.status, InventoryStatus.inStock);
      expect(item.version, 1);
    });

    /// 全ステータス値が正しく変換されることを確認する
    test('全ステータス値が正しくパースされる', () {
      for (final testCase in [
        ('in_stock', InventoryStatus.inStock),
        ('low_stock', InventoryStatus.lowStock),
        ('out_of_stock', InventoryStatus.outOfStock),
      ]) {
        final json = _sampleInventoryJson()..['status'] = testCase.$1;
        final item = InventoryItem.fromJson(json);
        expect(item.status, testCase.$2, reason: 'status=${testCase.$1}');
      }
    });

    /// 日時フィールドが DateTime に正しく変換されることを確認する
    test('日時フィールドが DateTime に変換される', () {
      final item = InventoryItem.fromJson(_sampleInventoryJson());

      expect(item.createdAt, isA<DateTime>());
      expect(item.updatedAt, isA<DateTime>());
      expect(item.createdAt.year, 2024);
    });
  });

  group('InventoryItem.toJson', () {
    /// fromJson して toJson した結果が元のデータと一致することを確認する（往復変換）
    test('fromJson → toJson で往復変換が成立する', () {
      final original = _sampleInventoryJson();
      final item = InventoryItem.fromJson(original);
      final json = item.toJson();

      expect(json['id'], original['id']);
      expect(json['product_id'], original['product_id']);
      expect(json['quantity_available'], original['quantity_available']);
      expect(json['version'], original['version']);
    });

    /// InventoryStatus が API 用のスネークケース文字列に変換されることを確認する
    test('InventoryStatus がスネークケースの文字列に変換される', () {
      final itemInStock = InventoryItem.fromJson(_sampleInventoryJson());
      expect(itemInStock.toJson()['status'], 'in_stock');

      final jsonLow = _sampleInventoryJson()..['status'] = 'low_stock';
      final itemLowStock = InventoryItem.fromJson(jsonLow);
      expect(itemLowStock.toJson()['status'], 'low_stock');

      final jsonOut = _sampleInventoryJson()..['status'] = 'out_of_stock';
      final itemOutOfStock = InventoryItem.fromJson(jsonOut);
      expect(itemOutOfStock.toJson()['status'], 'out_of_stock');
    });
  });

  group('InventoryStatus', () {
    /// 不明な文字列のデフォルト値が inStock になることを確認する
    test('不明な文字列は inStock にフォールバックする', () {
      expect(InventoryStatus.fromString('unknown_status'), InventoryStatus.inStock);
    });

    /// 各ステータスに日本語表示名が設定されていることを確認する
    test('各ステータスに日本語表示名が設定されている', () {
      expect(InventoryStatus.inStock.displayName, '在庫あり');
      expect(InventoryStatus.lowStock.displayName, '低在庫');
      expect(InventoryStatus.outOfStock.displayName, '在庫切れ');
    });
  });

  group('StockOperation.toJson', () {
    /// 在庫操作データが正しく JSON に変換されることを確認する
    test('在庫操作データが正しく toJson に変換される', () {
      const op = StockOperation(
        productId: 'PROD-001',
        warehouseId: 'WH-001',
        quantity: 5,
      );
      final json = op.toJson();

      expect(json['product_id'], 'PROD-001');
      expect(json['warehouse_id'], 'WH-001');
      expect(json['quantity'], 5);
    });
  });

  group('UpdateStockInput.toJson', () {
    /// 全フィールド指定で toJson に全て含まれることを確認する
    test('全フィールド指定で toJson に全て含まれる', () {
      const input = UpdateStockInput(quantityAvailable: 100, reorderPoint: 20);
      final json = input.toJson();

      expect(json['quantity_available'], 100);
      expect(json['reorder_point'], 20);
    });

    /// null フィールドが toJson から除外されることを確認する
    test('null フィールドが toJson から除外される', () {
      const input = UpdateStockInput(quantityAvailable: 50);
      final json = input.toJson();

      expect(json['quantity_available'], 50);
      expect(json.containsKey('reorder_point'), isFalse);
    });
  });
}
