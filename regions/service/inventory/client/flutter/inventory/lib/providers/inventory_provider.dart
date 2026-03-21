import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';
import 'package:system_client/system_client.dart';

import '../config/config_provider.dart';
import '../models/inventory.dart';
import '../repositories/inventory_repository.dart';

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
final inventoryRepositoryProvider = Provider<InventoryRepository>((ref) {
  return InventoryRepository(ref.watch(dioProvider));
});

/// 在庫一覧の状態を管理するNotifier
/// 在庫操作（引当・引当解除・更新）とローディング/エラー状態を統一的に管理する
class InventoryListNotifier extends Notifier<AsyncValue<List<InventoryItem>>> {
  @override
  AsyncValue<List<InventoryItem>> build() {
    /// 初期化時に在庫一覧を自動取得する
    load();
    return const AsyncValue.loading();
  }

  /// リポジトリをrefから取得するヘルパー
  InventoryRepository get _repository =>
      ref.read(inventoryRepositoryProvider);

  /// 在庫一覧をサーバーから取得する
  Future<void> load() async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(
      () => _repository.listInventory(),
    );
  }

  /// 在庫を引き当て、一覧を再取得する
  /// 受注処理時に在庫を予約状態にする
  Future<void> reserveStock(StockOperation operation) async {
    await _repository.reserveStock(operation);
    await load();
  }

  /// 在庫引当を解除し、一覧を再取得する
  /// キャンセル等で引き当てた在庫を元に戻す
  Future<void> releaseStock(StockOperation operation) async {
    await _repository.releaseStock(operation);
    await load();
  }

  /// 在庫を更新し、一覧を再取得する
  /// 在庫数や発注点の手動調整に使用する
  Future<void> updateStock(String id, UpdateStockInput input) async {
    await _repository.updateStock(id, input);
    await load();
  }
}

/// 在庫一覧のProvider
/// NotifierProviderを使用して状態管理を行う
final inventoryListProvider =
    NotifierProvider<InventoryListNotifier, AsyncValue<List<InventoryItem>>>(
  InventoryListNotifier.new,
);
