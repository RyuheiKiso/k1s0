import 'package:dio/dio.dart';

import '../models/payment.dart';

/// 決済APIのリポジトリ層
/// サーバーとの通信を担当し、決済データの操作を行う
class PaymentRepository {
  /// APIリクエストに使用するDioインスタンス
  final Dio _dio;

  /// コンストラクタ: Dioインスタンスを外部から注入する（テスタビリティのため）
  PaymentRepository(this._dio);

  // ========================================
  // 決済一覧・取得操作
  // ========================================

  /// 決済一覧を取得する
  /// [orderId] で注文IDによるフィルタリングを行う
  /// [customerId] で顧客IDによるフィルタリングを行う
  /// [status] でステータスによるフィルタリングを行う
  Future<List<Payment>> listPayments({
    String? orderId,
    String? customerId,
    PaymentStatus? status,
  }) async {
    final queryParameters = <String, dynamic>{};
    if (orderId != null) queryParameters['order_id'] = orderId;
    if (customerId != null) queryParameters['customer_id'] = customerId;
    if (status != null) queryParameters['status'] = status.name;

    final response = await _dio.get(
      '/api/v1/list_payments',
      queryParameters: queryParameters,
    );
    final List<dynamic> data = response.data['payments'] as List<dynamic>;
    return data
        .map((json) => Payment.fromJson(json as Map<String, dynamic>))
        .toList();
  }

  /// 指定IDの決済を取得する
  Future<Payment> getPayment(String id) async {
    final response = await _dio.get('/api/v1/get_payment/$id');
    return Payment.fromJson(response.data as Map<String, dynamic>);
  }

  // ========================================
  // 決済操作
  // ========================================

  /// 新規決済を開始する
  /// 決済開始入力データをサーバーに送信し、作成された決済を返す
  Future<Payment> initiatePayment(InitiatePaymentInput input) async {
    final response = await _dio.post(
      '/api/v1/initiate_payment',
      data: input.toJson(),
    );
    return Payment.fromJson(response.data as Map<String, dynamic>);
  }

  /// 指定IDの決済を完了する
  Future<Payment> completePayment(String id) async {
    final response = await _dio.post('/api/v1/complete_payment/$id');
    return Payment.fromJson(response.data as Map<String, dynamic>);
  }

  /// 指定IDの決済を失敗にする
  Future<Payment> failPayment(String id) async {
    final response = await _dio.post('/api/v1/fail_payment/$id');
    return Payment.fromJson(response.data as Map<String, dynamic>);
  }

  /// 指定IDの決済を返金する
  Future<Payment> refundPayment(String id) async {
    final response = await _dio.post('/api/v1/refund_payment/$id');
    return Payment.fromJson(response.data as Map<String, dynamic>);
  }
}
