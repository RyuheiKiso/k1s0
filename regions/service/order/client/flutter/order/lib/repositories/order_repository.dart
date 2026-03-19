import 'package:dio/dio.dart';

import '../models/order.dart';

/// 注文APIのリポジトリ層
/// サーバーとの通信を担当し、注文データの永続化・取得を行う
class OrderRepository {
  /// APIリクエストに使用するDioインスタンス
  final Dio _dio;

  /// コンストラクタ: Dioインスタンスを外部から注入する（テスタビリティのため）
  OrderRepository(this._dio);

  // ========================================
  // 注文操作
  // ========================================

  /// 注文一覧を取得する
  /// [customerId] で顧客IDによるフィルタリングが可能
  /// [status] でステータスによるフィルタリングが可能
  Future<List<Order>> listOrders({
    String? customerId,
    OrderStatus? status,
  }) async {
    /// クエリパラメータを動的に構築する
    final queryParameters = <String, dynamic>{};
    if (customerId != null) queryParameters['customer_id'] = customerId;
    if (status != null) queryParameters['status'] = status.name;

    final response = await _dio.get(
      '/api/v1/list_orders',
      queryParameters: queryParameters,
    );
    final List<dynamic> data = (response.data as Map<String, dynamic>)['orders'] as List<dynamic>;
    return data
        .map((json) => Order.fromJson(json as Map<String, dynamic>))
        .toList();
  }

  /// 指定IDの注文を取得する
  Future<Order> getOrder(String id) async {
    final response = await _dio.get('/api/v1/get_order/$id');
    return Order.fromJson(response.data as Map<String, dynamic>);
  }

  /// 新規注文を作成する
  /// 作成された注文をレスポンスから返す
  Future<Order> createOrder(CreateOrderInput input) async {
    final response = await _dio.post(
      '/api/v1/create_order',
      data: input.toJson(),
    );
    return Order.fromJson(response.data as Map<String, dynamic>);
  }

  /// 指定IDの注文ステータスを更新する
  Future<Order> updateOrderStatus(
    String id,
    UpdateOrderStatusInput input,
  ) async {
    final response = await _dio.put(
      '/api/v1/update_order_status/$id',
      data: input.toJson(),
    );
    return Order.fromJson(response.data as Map<String, dynamic>);
  }
}
