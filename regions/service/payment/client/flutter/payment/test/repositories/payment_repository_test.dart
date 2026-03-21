/// payment_repository_test.dart: PaymentRepositoryのユニットテスト。
/// MockHttpClientAdapterを使用してAPI通信をモックし、
/// 各メソッドが正しくリクエストを送信・レスポンスを処理することを検証する。
import 'dart:convert';

import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:payment/models/payment.dart';
import 'package:payment/repositories/payment_repository.dart';

/// テスト用のHTTPクライアントアダプター。
/// メソッドごとに異なるレスポンスを返すよう設定可能。
class _MockHttpClientAdapter implements HttpClientAdapter {
  /// リクエストのパスとメソッドに応じたレスポンス文字列を返す関数
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

/// テスト用のサンプル決済JSONデータを生成する
Map<String, dynamic> _samplePaymentData({
  String id = 'PAY-001',
  String status = 'pending',
}) =>
    {
      'id': id,
      'order_id': 'ORDER-001',
      'customer_id': 'CUST-001',
      'amount': 10000.0,
      'currency': 'JPY',
      'status': status,
      'payment_method': 'credit_card',
      'transaction_id': null,
      'failure_reason': null,
      'refund_amount': null,
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
  group('PaymentRepository.listPayments', () {
    /// 決済一覧が正しく取得されることを確認する
    test('決済一覧が正しく取得される', () async {
      final dio = _createTestDio((_) => jsonEncode({
            'payments': [
              _samplePaymentData(id: 'PAY-001'),
              _samplePaymentData(id: 'PAY-002', status: 'completed'),
            ]
          }));
      final repo = PaymentRepository(dio);

      final payments = await repo.listPayments();

      expect(payments, hasLength(2));
      expect(payments[0].id, 'PAY-001');
      expect(payments[1].id, 'PAY-002');
      expect(payments[1].status, PaymentStatus.completed);
    });

    /// 空の決済一覧が返されたときに空リストになることを確認する
    test('空の決済一覧が返される', () async {
      final dio = _createTestDio((_) => jsonEncode({'payments': []}));
      final repo = PaymentRepository(dio);

      final payments = await repo.listPayments();

      expect(payments, isEmpty);
    });
  });

  group('PaymentRepository.getPayment', () {
    /// 指定IDの決済が正しく取得されることを確認する
    test('指定IDの決済が正しく取得される', () async {
      final dio = _createTestDio((_) => jsonEncode(_samplePaymentData(id: 'PAY-001')));
      final repo = PaymentRepository(dio);

      final payment = await repo.getPayment('PAY-001');

      expect(payment.id, 'PAY-001');
      expect(payment.status, PaymentStatus.pending);
      expect(payment.amount, 10000.0);
    });
  });

  group('PaymentRepository.initiatePayment', () {
    /// 新規決済が正しく作成されることを確認する
    test('新規決済が正しく作成される', () async {
      final dio = _createTestDio((_) => jsonEncode(_samplePaymentData(id: 'PAY-NEW')));
      final repo = PaymentRepository(dio);

      const input = InitiatePaymentInput(
        orderId: 'ORDER-001',
        customerId: 'CUST-001',
        amount: 10000.0,
        paymentMethod: PaymentMethod.creditCard,
      );
      final payment = await repo.initiatePayment(input);

      expect(payment.id, 'PAY-NEW');
    });
  });

  group('PaymentRepository.completePayment', () {
    /// 決済完了が正しく処理されることを確認する
    test('決済完了が正しく処理される', () async {
      final dio = _createTestDio(
          (_) => jsonEncode(_samplePaymentData(id: 'PAY-001', status: 'completed')));
      final repo = PaymentRepository(dio);

      final payment = await repo.completePayment('PAY-001');

      expect(payment.id, 'PAY-001');
      expect(payment.status, PaymentStatus.completed);
    });
  });
}
