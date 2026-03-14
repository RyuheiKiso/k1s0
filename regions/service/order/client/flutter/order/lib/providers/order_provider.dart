import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';

import '../config/config_provider.dart';
import '../models/order.dart';
import '../repositories/order_repository.dart';
import '../utils/dio_client.dart';

/// DioインスタンスのProvider
/// YAML設定ファイルから読み込んだベースURLを使用してHTTPクライアントを生成する
final dioProvider = Provider<Dio>((ref) {
  /// 設定プロバイダーからAPI設定を取得する
  final config = ref.watch(appConfigProvider);
  return DioClient.create(baseUrl: config.api.baseUrl);
});

/// リポジトリのProvider
/// DioインスタンスをDIしてリポジトリを生成する
final orderRepositoryProvider = Provider<OrderRepository>((ref) {
  return OrderRepository(ref.watch(dioProvider));
});

/// 注文一覧の状態を管理するStateNotifier
/// CRUD操作とローディング/エラー状態を統一的に管理する
class OrderListNotifier extends StateNotifier<AsyncValue<List<Order>>> {
  final OrderRepository _repository;

  OrderListNotifier(this._repository) : super(const AsyncValue.loading()) {
    /// 初期化時に注文一覧を自動取得する
    load();
  }

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
/// StateNotifierProviderを使用して状態管理を行う
final orderListProvider =
    StateNotifierProvider<OrderListNotifier, AsyncValue<List<Order>>>(
  (ref) => OrderListNotifier(ref.watch(orderRepositoryProvider)),
);
