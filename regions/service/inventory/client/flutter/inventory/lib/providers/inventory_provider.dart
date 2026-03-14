import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';

import '../models/inventory.dart';
import '../repositories/inventory_repository.dart';
import '../utils/dio_client.dart';

/// DioインスタンスのProvider
/// アプリ全体で共有されるHTTPクライアントを提供する
final dioProvider = Provider<Dio>((ref) {
  /// 環境変数またはデフォルトのベースURLを使用する
  return DioClient.create(baseUrl: const String.fromEnvironment(
    'API_BASE_URL',
    defaultValue: 'http://localhost:8080',
  ));
});

/// リポジトリのProvider
/// DioインスタンスをDIしてリポジトリを生成する
final inventoryRepositoryProvider = Provider<InventoryRepository>((ref) {
  return InventoryRepository(ref.watch(dioProvider));
});

/// 在庫一覧の状態を管理するStateNotifier
/// 在庫操作（引当・引当解除・更新）とローディング/エラー状態を統一的に管理する
class InventoryListNotifier extends StateNotifier<AsyncValue<List<InventoryItem>>> {
  final InventoryRepository _repository;

  InventoryListNotifier(this._repository) : super(const AsyncValue.loading()) {
    /// 初期化時に在庫一覧を自動取得する
    load();
  }

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
/// StateNotifierProviderを使用して状態管理を行う
final inventoryListProvider =
    StateNotifierProvider<InventoryListNotifier, AsyncValue<List<InventoryItem>>>(
  (ref) => InventoryListNotifier(ref.watch(inventoryRepositoryProvider)),
);
