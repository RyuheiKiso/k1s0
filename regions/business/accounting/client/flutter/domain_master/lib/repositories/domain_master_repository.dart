import 'package:dio/dio.dart';

import '../models/domain_master.dart';

/// ドメインマスタAPIのリポジトリ層
/// サーバーとの通信を担当し、データの永続化・取得を行う
class DomainMasterRepository {
  /// APIリクエストに使用するDioインスタンス
  final Dio _dio;

  /// コンストラクタ: Dioインスタンスを外部から注入する（テスタビリティのため）
  DomainMasterRepository(this._dio);

  // ========================================
  // カテゴリ操作
  // ========================================

  /// カテゴリ一覧を取得する
  /// [activeOnly] がtrueの場合、有効なカテゴリのみを返す
  Future<List<MasterCategory>> listCategories({bool activeOnly = false}) async {
    final response = await _dio.get(
      '/api/v1/categories',
      queryParameters: {'active_only': activeOnly},
    );
    final List<dynamic> data = response.data['categories'] as List<dynamic>;
    return data
        .map((json) => MasterCategory.fromJson(json as Map<String, dynamic>))
        .toList();
  }

  /// 新規カテゴリを作成する
  /// 作成されたカテゴリをレスポンスから返す
  Future<MasterCategory> createCategory(CreateCategoryInput input) async {
    final response = await _dio.post(
      '/api/v1/categories',
      data: input.toJson(),
    );
    return MasterCategory.fromJson(response.data as Map<String, dynamic>);
  }

  /// 指定コードのカテゴリを取得する
  Future<MasterCategory> getCategory(String code) async {
    final response = await _dio.get('/api/v1/categories/$code');
    return MasterCategory.fromJson(response.data as Map<String, dynamic>);
  }

  /// 指定コードのカテゴリを更新する
  Future<MasterCategory> updateCategory(
    String code,
    UpdateCategoryInput input,
  ) async {
    final response = await _dio.put(
      '/api/v1/categories/$code',
      data: input.toJson(),
    );
    return MasterCategory.fromJson(response.data as Map<String, dynamic>);
  }

  /// 指定コードのカテゴリを削除する
  Future<void> deleteCategory(String code) async {
    await _dio.delete('/api/v1/categories/$code');
  }

  // ========================================
  // アイテム操作
  // ========================================

  /// 指定カテゴリのアイテム一覧を取得する
  /// [activeOnly] がtrueの場合、有効なアイテムのみを返す
  Future<List<MasterItem>> listItems(
    String categoryCode, {
    bool activeOnly = false,
  }) async {
    final response = await _dio.get(
      '/api/v1/categories/$categoryCode/items',
      queryParameters: {'active_only': activeOnly},
    );
    final List<dynamic> data = response.data['items'] as List<dynamic>;
    return data
        .map((json) => MasterItem.fromJson(json as Map<String, dynamic>))
        .toList();
  }

  /// 指定カテゴリに新規アイテムを作成する
  Future<MasterItem> createItem(
    String categoryCode,
    CreateItemInput input,
  ) async {
    final response = await _dio.post(
      '/api/v1/categories/$categoryCode/items',
      data: input.toJson(),
    );
    return MasterItem.fromJson(response.data as Map<String, dynamic>);
  }

  /// 指定カテゴリの特定アイテムを取得する
  Future<MasterItem> getItem(String categoryCode, String itemCode) async {
    final response = await _dio.get(
      '/api/v1/categories/$categoryCode/items/$itemCode',
    );
    return MasterItem.fromJson(response.data as Map<String, dynamic>);
  }

  /// 指定カテゴリの特定アイテムを更新する
  Future<MasterItem> updateItem(
    String categoryCode,
    String itemCode,
    UpdateItemInput input,
  ) async {
    final response = await _dio.put(
      '/api/v1/categories/$categoryCode/items/$itemCode',
      data: input.toJson(),
    );
    return MasterItem.fromJson(response.data as Map<String, dynamic>);
  }

  /// 指定カテゴリの特定アイテムを削除する
  Future<void> deleteItem(String categoryCode, String itemCode) async {
    await _dio.delete(
      '/api/v1/categories/$categoryCode/items/$itemCode',
    );
  }

  // ========================================
  // バージョン履歴操作
  // ========================================

  /// 指定アイテムのバージョン履歴を取得する
  /// 変更の監査証跡として使用する
  Future<List<MasterItemVersion>> listVersions(
    String categoryCode,
    String itemCode,
  ) async {
    final response = await _dio.get(
      '/api/v1/categories/$categoryCode/items/$itemCode/versions',
    );
    final List<dynamic> data = response.data['versions'] as List<dynamic>;
    return data
        .map((json) =>
            MasterItemVersion.fromJson(json as Map<String, dynamic>))
        .toList();
  }

  // ========================================
  // テナント拡張操作
  // ========================================

  /// テナント固有のアイテム拡張情報を取得する
  Future<TenantMasterExtension> getTenantExtension(
    String tenantId,
    String itemId,
  ) async {
    final response = await _dio.get(
      '/api/v1/tenants/$tenantId/items/$itemId',
    );
    return TenantMasterExtension.fromJson(
        response.data as Map<String, dynamic>);
  }

  /// テナント固有のアイテム拡張情報を更新する
  Future<TenantMasterExtension> updateTenantExtension(
    String tenantId,
    String itemId,
    UpdateTenantExtensionInput input,
  ) async {
    final response = await _dio.put(
      '/api/v1/tenants/$tenantId/items/$itemId',
      data: input.toJson(),
    );
    return TenantMasterExtension.fromJson(
        response.data as Map<String, dynamic>);
  }

  /// テナント固有のアイテム拡張情報を削除する
  Future<void> deleteTenantExtension(
    String tenantId,
    String itemId,
  ) async {
    await _dio.delete('/api/v1/tenants/$tenantId/items/$itemId');
  }

  /// テナント固有のカテゴリ内アイテム一覧を取得する
  /// テナント拡張が適用されたアイテムを返す
  Future<List<MasterItem>> listTenantItems(
    String tenantId,
    String categoryCode,
  ) async {
    final response = await _dio.get(
      '/api/v1/tenants/$tenantId/categories/$categoryCode/items',
    );
    final List<dynamic> data = response.data['items'] as List<dynamic>;
    return data
        .map((json) => MasterItem.fromJson(json as Map<String, dynamic>))
        .toList();
  }
}
