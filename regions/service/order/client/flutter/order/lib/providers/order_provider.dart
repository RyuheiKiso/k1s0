import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';
import 'package:system_client/system_client.dart';

import '../config/config_provider.dart';
import '../models/order.dart';
import '../repositories/order_repository.dart';

/// DioインスタンスのProvider
/// system_client の ApiClient.create() を使用して CSRF 契約を正しく実装する
/// authProvider から CSRF トークンを取得してインターセプターに渡す
final dioProvider = Provider<Dio>((ref) {
  final config = ref.watch(appConfigProvider);
  final authNotifier = ref.read(authProvider.notifier);
  final sessionInterceptor = kIsWeb ? null : SessionCookieInterceptor();
  return ApiClient.create(
    baseUrl: config.api.baseUrl,
    // authProvider が /auth/session JSON から取得した CSRF トークンを注入する
    csrfTokenProvider: () async => authNotifier.csrfToken,
    sessionCookieInterceptor: sessionInterceptor,
  );
});

/// リポジトリのProvider
/// DioインスタンスをDIしてリポジトリを生成する
final orderRepositoryProvider = Provider<OrderRepository>((ref) {
  return OrderRepository(ref.watch(dioProvider));
});

/// 注文一覧の状態を管理するNotifier
/// CRUD操作とローディング/エラー状態を統一的に管理する
class OrderListNotifier extends Notifier<AsyncValue<List<Order>>> {
  @override
  AsyncValue<List<Order>> build() {
    /// 初期化時に注文一覧を自動取得する
    load();
    return const AsyncValue.loading();
  }

  /// リポジトリをrefから取得するヘルパー
  OrderRepository get _repository =>
      ref.read(orderRepositoryProvider);

  /// 注文一覧をサーバーから取得する
  /// [customerId] で顧客IDによるフィルタリングが可能
  /// [status] でステータスによるフィルタリングが可能
  Future<void> load({String? customerId, OrderStatus? status}) async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(
      () => _repository.listOrders(
        customerId: customerId,
        status: status,
      ),
    );
  }

  /// 新規注文を作成し、一覧を再取得する
  Future<void> create(CreateOrderInput input) async {
    await _repository.createOrder(input);
    await load();
  }

  /// 注文ステータスを更新し、一覧を再取得する
  Future<void> updateStatus(String id, UpdateOrderStatusInput input) async {
    await _repository.updateOrderStatus(id, input);
    await load();
  }
}

/// 注文一覧のProvider
/// NotifierProviderを使用して状態管理を行う
final orderListProvider =
    NotifierProvider<OrderListNotifier, AsyncValue<List<Order>>>(
  OrderListNotifier.new,
);
