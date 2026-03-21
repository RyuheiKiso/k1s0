/// payment_test.dart: Paymentモデルのユニットテスト。
/// fromJson/toJsonの往復変換、enum変換、nullable処理を検証する。
import 'package:flutter_test/flutter_test.dart';
import 'package:payment/models/payment.dart';

/// テスト用の基本的なPayment JSONデータ
Map<String, dynamic> _samplePaymentJson() => {
      'id': '550e8400-e29b-41d4-a716-446655440001',
      'order_id': 'ORDER-001',
      'customer_id': 'CUST-001',
      'amount': 10000.0,
      'currency': 'JPY',
      'status': 'pending',
      'payment_method': 'credit_card',
      'transaction_id': null,
      'failure_reason': null,
      'refund_amount': null,
      'created_at': '2024-01-20T00:00:00.000Z',
      'updated_at': '2024-01-20T00:00:00.000Z',
    };

void main() {
  group('Payment.fromJson', () {
    /// 完全なJSONデータが正しくPaymentインスタンスに変換されることを確認する
    test('必須フィールドが正しくパースされる', () {
      final json = _samplePaymentJson();
      final payment = Payment.fromJson(json);

      expect(payment.id, '550e8400-e29b-41d4-a716-446655440001');
      expect(payment.orderId, 'ORDER-001');
      expect(payment.customerId, 'CUST-001');
      expect(payment.amount, 10000.0);
      expect(payment.currency, 'JPY');
      expect(payment.status, PaymentStatus.pending);
      expect(payment.paymentMethod, PaymentMethod.creditCard);
    });

    /// nullable フィールドが null のとき正しく null が設定されることを確認する
    test('nullable フィールドが null の場合に null が返る', () {
      final json = _samplePaymentJson();
      final payment = Payment.fromJson(json);

      expect(payment.transactionId, isNull);
      expect(payment.failureReason, isNull);
      expect(payment.refundAmount, isNull);
    });

    /// nullable フィールドに値がある場合に正しく設定されることを確認する
    test('nullable フィールドに値がある場合に値が返る', () {
      final json = _samplePaymentJson()
        ..['transaction_id'] = 'TXN-12345'
        ..['failure_reason'] = '残高不足'
        ..['refund_amount'] = 5000.0;
      final payment = Payment.fromJson(json);

      expect(payment.transactionId, 'TXN-12345');
      expect(payment.failureReason, '残高不足');
      expect(payment.refundAmount, 5000.0);
    });

    /// 日時フィールドが DateTime に正しく変換されることを確認する
    test('日時フィールドが DateTime に変換される', () {
      final payment = Payment.fromJson(_samplePaymentJson());

      expect(payment.createdAt, isA<DateTime>());
      expect(payment.updatedAt, isA<DateTime>());
      expect(payment.createdAt.year, 2024);
    });
  });

  group('Payment.toJson', () {
    /// fromJson して toJson した結果が元のデータと一致することを確認する（往復変換）
    test('fromJson → toJson で往復変換が成立する', () {
      final original = _samplePaymentJson();
      final payment = Payment.fromJson(original);
      final json = payment.toJson();

      expect(json['id'], original['id']);
      expect(json['order_id'], original['order_id']);
      expect(json['amount'], original['amount']);
      expect(json['status'], 'pending');
      expect(json['payment_method'], 'credit_card');
    });

    /// nullableフィールドが null のとき toJson に null が含まれることを確認する
    test('nullable フィールドが null の場合に toJson に null が含まれる', () {
      final payment = Payment.fromJson(_samplePaymentJson());
      final json = payment.toJson();

      expect(json['transaction_id'], isNull);
      expect(json['failure_reason'], isNull);
      expect(json['refund_amount'], isNull);
    });
  });

  group('PaymentStatus', () {
    /// 全ステータス値が文字列から正しく変換されることを確認する
    test('全ステータス値が fromString で正しく変換される', () {
      expect(PaymentStatus.fromString('pending'), PaymentStatus.pending);
      expect(PaymentStatus.fromString('processing'), PaymentStatus.processing);
      expect(PaymentStatus.fromString('completed'), PaymentStatus.completed);
      expect(PaymentStatus.fromString('failed'), PaymentStatus.failed);
      expect(PaymentStatus.fromString('refunded'), PaymentStatus.refunded);
    });

    /// 不明な文字列のデフォルト値が pending になることを確認する
    test('不明な文字列は pending にフォールバックする', () {
      expect(PaymentStatus.fromString('unknown_status'), PaymentStatus.pending);
    });

    /// 各ステータスに日本語ラベルが設定されていることを確認する
    test('各ステータスに日本語ラベルが設定されている', () {
      expect(PaymentStatus.pending.label, '保留中');
      expect(PaymentStatus.completed.label, '完了');
      expect(PaymentStatus.failed.label, '失敗');
    });
  });

  group('PaymentMethod', () {
    /// 全決済方法が文字列から正しく変換されることを確認する
    test('全決済方法が fromString で正しく変換される', () {
      expect(PaymentMethod.fromString('credit_card'), PaymentMethod.creditCard);
      expect(PaymentMethod.fromString('bank_transfer'), PaymentMethod.bankTransfer);
      expect(PaymentMethod.fromString('convenience_store'), PaymentMethod.convenienceStore);
      expect(PaymentMethod.fromString('e_wallet'), PaymentMethod.eWallet);
    });

    /// 各決済方法の value が正しい API 文字列を返すことを確認する
    test('各決済方法の value が正しい API 文字列を返す', () {
      expect(PaymentMethod.creditCard.value, 'credit_card');
      expect(PaymentMethod.bankTransfer.value, 'bank_transfer');
      expect(PaymentMethod.convenienceStore.value, 'convenience_store');
      expect(PaymentMethod.eWallet.value, 'e_wallet');
    });
  });

  group('InitiatePaymentInput.toJson', () {
    /// 必須フィールドのみで toJson が正しく変換されることを確認する
    test('必須フィールドのみで toJson が正しく変換される', () {
      const input = InitiatePaymentInput(
        orderId: 'ORDER-001',
        customerId: 'CUST-001',
        amount: 5000.0,
        paymentMethod: PaymentMethod.creditCard,
      );
      final json = input.toJson();

      expect(json['order_id'], 'ORDER-001');
      expect(json['customer_id'], 'CUST-001');
      expect(json['amount'], 5000.0);
      expect(json['payment_method'], 'credit_card');
      // currency は null なので含まれないこと
      expect(json.containsKey('currency'), isFalse);
    });

    /// currency が指定された場合に toJson に含まれることを確認する
    test('currency 指定時に toJson に含まれる', () {
      const input = InitiatePaymentInput(
        orderId: 'ORDER-001',
        customerId: 'CUST-001',
        amount: 5000.0,
        currency: 'USD',
        paymentMethod: PaymentMethod.eWallet,
      );
      final json = input.toJson();

      expect(json['currency'], 'USD');
    });
  });
}
