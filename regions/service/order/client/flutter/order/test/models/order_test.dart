/// order_test.dart: Orderモデルのユニットテスト。
/// fromJson/toJsonの往復変換、enum変換、nullable処理を検証する。
import 'package:flutter_test/flutter_test.dart';
import 'package:order/models/order.dart';

/// テスト用の基本的なOrderItem JSONデータ
Map<String, dynamic> _sampleItemJson() => {
      'product_id': 'PROD-001',
      'product_name': 'テスト商品',
      'quantity': 2,
      'unit_price': 5000.0,
      'subtotal': 10000.0,
    };

/// テスト用の基本的なOrder JSONデータ
Map<String, dynamic> _sampleOrderJson() => {
      'id': '550e8400-e29b-41d4-a716-446655440001',
      'customer_id': 'CUST-001',
      'status': 'pending',
      'total_amount': 10000.0,
      'currency': 'JPY',
      'items': [_sampleItemJson()],
      'notes': null,
      'version': 1,
      'created_at': '2024-01-20T00:00:00.000Z',
      'updated_at': '2024-01-20T00:00:00.000Z',
    };

void main() {
  group('Order.fromJson', () {
    /// 完全なJSONデータが正しくOrderインスタンスに変換されることを確認する
    test('必須フィールドが正しくパースされる', () {
      final order = Order.fromJson(_sampleOrderJson());

      expect(order.id, '550e8400-e29b-41d4-a716-446655440001');
      expect(order.customerId, 'CUST-001');
      expect(order.status, OrderStatus.pending);
      expect(order.totalAmount, 10000.0);
      expect(order.currency, 'JPY');
      expect(order.version, 1);
    });

    /// notes が null のとき正しく null が設定されることを確認する
    test('nullable の notes が null の場合に null が返る', () {
      final order = Order.fromJson(_sampleOrderJson());
      expect(order.notes, isNull);
    });

    /// notes に値がある場合に正しく設定されることを確認する
    test('notes に値がある場合に値が返る', () {
      final json = _sampleOrderJson()..['notes'] = '配送メモ';
      final order = Order.fromJson(json);
      expect(order.notes, '配送メモ');
    });

    /// items が正しく OrderItem リストに変換されることを確認する
    test('items が OrderItem リストに変換される', () {
      final order = Order.fromJson(_sampleOrderJson());

      expect(order.items, hasLength(1));
      expect(order.items[0].productId, 'PROD-001');
      expect(order.items[0].quantity, 2);
      expect(order.items[0].subtotal, 10000.0);
    });
  });

  group('Order.toJson', () {
    /// fromJson して toJson した結果が元のデータと一致することを確認する（往復変換）
    test('fromJson → toJson で往復変換が成立する', () {
      final original = _sampleOrderJson();
      final order = Order.fromJson(original);
      final json = order.toJson();

      expect(json['id'], original['id']);
      expect(json['customer_id'], original['customer_id']);
      expect(json['status'], 'pending');
      expect(json['total_amount'], original['total_amount']);
      expect(json['version'], original['version']);
    });

    /// items が正しく toJson に含まれることを確認する
    test('items が toJson に含まれる', () {
      final order = Order.fromJson(_sampleOrderJson());
      final json = order.toJson();

      final items = json['items'] as List<dynamic>;
      expect(items, hasLength(1));
      expect((items[0] as Map<String, dynamic>)['product_id'], 'PROD-001');
    });
  });

  group('OrderStatus', () {
    /// 全ステータス値が文字列から正しく変換されることを確認する
    test('全ステータス値が fromString で正しく変換される', () {
      expect(OrderStatus.fromString('pending'), OrderStatus.pending);
      expect(OrderStatus.fromString('confirmed'), OrderStatus.confirmed);
      expect(OrderStatus.fromString('processing'), OrderStatus.processing);
      expect(OrderStatus.fromString('shipped'), OrderStatus.shipped);
      expect(OrderStatus.fromString('delivered'), OrderStatus.delivered);
      expect(OrderStatus.fromString('cancelled'), OrderStatus.cancelled);
    });

    /// 不明な文字列のデフォルト値が pending になることを確認する
    test('不明な文字列は pending にフォールバックする', () {
      expect(OrderStatus.fromString('unknown_status'), OrderStatus.pending);
    });

    /// 各ステータスに日本語表示名が設定されていることを確認する
    test('各ステータスに日本語表示名が設定されている', () {
      expect(OrderStatus.pending.displayName, '保留中');
      expect(OrderStatus.shipped.displayName, '発送済み');
      expect(OrderStatus.cancelled.displayName, 'キャンセル');
    });
  });

  group('CreateOrderInput.toJson', () {
    /// 入力データが正しく JSON に変換されることを確認する
    test('入力データが正しく toJson に変換される', () {
      final input = CreateOrderInput(
        customerId: 'CUST-001',
        currency: 'JPY',
        items: [
          const CreateOrderItemInput(
            productId: 'PROD-001',
            productName: 'テスト商品',
            quantity: 1,
            unitPrice: 1000.0,
          ),
        ],
      );
      final json = input.toJson();

      expect(json['customer_id'], 'CUST-001');
      expect(json['currency'], 'JPY');
      expect((json['items'] as List<dynamic>), hasLength(1));
    });
  });

  group('UpdateOrderStatusInput.toJson', () {
    /// ステータス更新データが正しく JSON に変換されることを確認する
    test('ステータス更新データが正しく toJson に変換される', () {
      const input = UpdateOrderStatusInput(status: OrderStatus.confirmed);
      final json = input.toJson();

      expect(json['status'], 'confirmed');
    });
  });
}
