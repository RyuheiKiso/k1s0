import 'package:dio/dio.dart';

import '../models/inventory.dart';

/// 在庫管理APIのリポジトリ層
/// サーバーとの通信を担当し、在庫データの取得・操作を行う
class InventoryRepository {
  /// APIリクエストに使用するDioインスタンス
  final Dio _dio;

  /// コンストラクタ: Dioインスタンスを外部から注入する（テスタビリティのため）
  InventoryRepository(this._dio);

  // ========================================
  // 在庫参照操作
  // ========================================

  /// 在庫一覧を取得する
  /// サーバーから全在庫アイテムのリストを取得して返す
  Future<List<InventoryItem>> listInventory() async {
    final response = await _dio.get('/api/v1/inventory');
    final List<dynamic> data = (response.data as Map<String, dynamic>)['items'] as List<dynamic>;
    return data
        .map((json) => InventoryItem.fromJson(json as Map<String, dynamic>))
        .toList();
  }

  /// 指定IDの在庫アイテムを取得する
  /// 在庫詳細画面での表示に使用する
  Future<InventoryItem> getInventory(String id) async {
    final response = await _dio.get('/api/v1/inventory/$id');
    return InventoryItem.fromJson(response.data as Map<String, dynamic>);
  }

  // ========================================
  // 在庫操作
  // ========================================

  /// 在庫を引き当てる
  /// 受注処理時に在庫を予約状態にする
  Future<void> reserveStock(StockOperation operation) async {
    await _dio.post(
      '/api/v1/inventory/reserve',
      data: operation.toJson(),
    );
  }

  /// 在庫引当を解除する
  /// キャンセル等で引き当てた在庫を元に戻す
  Future<void> releaseStock(StockOperation operation) async {
    await _dio.post(
      '/api/v1/inventory/release',
      data: operation.toJson(),
    );
  }

  /// 在庫数量を更新する
  /// 在庫数や発注点の手動調整に使用する
  Future<InventoryItem> updateStock(String id, UpdateStockInput input) async {
    final response = await _dio.put(
      '/api/v1/inventory/$id/stock',
      data: input.toJson(),
    );
    return InventoryItem.fromJson(response.data as Map<String, dynamic>);
  }
}
