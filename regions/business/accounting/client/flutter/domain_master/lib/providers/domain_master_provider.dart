import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';

import '../models/domain_master.dart';
import '../repositories/domain_master_repository.dart';
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
final domainMasterRepositoryProvider = Provider<DomainMasterRepository>((ref) {
  return DomainMasterRepository(ref.watch(dioProvider));
});

/// カテゴリ一覧の状態を管理するStateNotifier
/// CRUD操作とローディング/エラー状態を統一的に管理する
class CategoryListNotifier extends StateNotifier<AsyncValue<List<MasterCategory>>> {
  final DomainMasterRepository _repository;

  CategoryListNotifier(this._repository) : super(const AsyncValue.loading()) {
    /// 初期化時にカテゴリ一覧を自動取得する
    load();
  }

  /// カテゴリ一覧をサーバーから取得する
  Future<void> load({bool activeOnly = false}) async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(
      () => _repository.listCategories(activeOnly: activeOnly),
    );
  }

  /// 新規カテゴリを作成し、一覧を再取得する
  Future<void> create(CreateCategoryInput input) async {
    await _repository.createCategory(input);
    await load();
  }

  /// カテゴリを更新し、一覧を再取得する
  Future<void> update(String code, UpdateCategoryInput input) async {
    await _repository.updateCategory(code, input);
    await load();
  }

  /// カテゴリを削除し、一覧を再取得する
  Future<void> delete(String code) async {
    await _repository.deleteCategory(code);
    await load();
  }
}

/// カテゴリ一覧のProvider
/// StateNotifierProviderを使用して状態管理を行う
final categoryListProvider =
    StateNotifierProvider<CategoryListNotifier, AsyncValue<List<MasterCategory>>>(
  (ref) => CategoryListNotifier(ref.watch(domainMasterRepositoryProvider)),
);

/// アイテム一覧の状態を管理するStateNotifier
/// カテゴリコードに紐づくアイテムのCRUD操作を管理する
class ItemListNotifier extends StateNotifier<AsyncValue<List<MasterItem>>> {
  final DomainMasterRepository _repository;
  final String _categoryCode;

  ItemListNotifier(this._repository, this._categoryCode)
      : super(const AsyncValue.loading()) {
    /// 初期化時にアイテム一覧を自動取得する
    load();
  }

  /// アイテム一覧をサーバーから取得する
  Future<void> load({bool activeOnly = false}) async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(
      () => _repository.listItems(_categoryCode, activeOnly: activeOnly),
    );
  }

  /// 新規アイテムを作成し、一覧を再取得する
  Future<void> create(CreateItemInput input) async {
    await _repository.createItem(_categoryCode, input);
    await load();
  }

  /// アイテムを更新し、一覧を再取得する
  Future<void> update(String itemCode, UpdateItemInput input) async {
    await _repository.updateItem(_categoryCode, itemCode, input);
    await load();
  }

  /// アイテムを削除し、一覧を再取得する
  Future<void> delete(String itemCode) async {
    await _repository.deleteItem(_categoryCode, itemCode);
    await load();
  }
}

/// アイテム一覧のProviderファミリー
/// カテゴリコードごとに独立した状態を管理する
final itemListProvider = StateNotifierProvider.family<ItemListNotifier,
    AsyncValue<List<MasterItem>>, String>(
  (ref, categoryCode) =>
      ItemListNotifier(ref.watch(domainMasterRepositoryProvider), categoryCode),
);

/// バージョン履歴の状態を管理するStateNotifier
class VersionListNotifier
    extends StateNotifier<AsyncValue<List<MasterItemVersion>>> {
  final DomainMasterRepository _repository;
  final String _categoryCode;
  final String _itemCode;

  VersionListNotifier(this._repository, this._categoryCode, this._itemCode)
      : super(const AsyncValue.loading()) {
    /// 初期化時にバージョン履歴を自動取得する
    load();
  }

  /// バージョン履歴をサーバーから取得する
  Future<void> load() async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(
      () => _repository.listVersions(_categoryCode, _itemCode),
    );
  }
}

/// バージョン履歴のProviderファミリー
/// カテゴリコードとアイテムコードの組み合わせで状態を管理する
final versionListProvider = StateNotifierProvider.family<VersionListNotifier,
    AsyncValue<List<MasterItemVersion>>, ({String categoryCode, String itemCode})>(
  (ref, params) => VersionListNotifier(
    ref.watch(domainMasterRepositoryProvider),
    params.categoryCode,
    params.itemCode,
  ),
);

/// テナント拡張の状態を管理するStateNotifier
class TenantExtensionNotifier
    extends StateNotifier<AsyncValue<TenantMasterExtension?>> {
  final DomainMasterRepository _repository;

  TenantExtensionNotifier(this._repository)
      : super(const AsyncValue.data(null));

  /// テナント拡張情報をサーバーから取得する
  Future<void> load(String tenantId, String itemId) async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(
      () => _repository.getTenantExtension(tenantId, itemId),
    );
  }

  /// テナント拡張情報を更新する
  Future<void> update(
    String tenantId,
    String itemId,
    UpdateTenantExtensionInput input,
  ) async {
    await _repository.updateTenantExtension(tenantId, itemId, input);
    await load(tenantId, itemId);
  }

  /// テナント拡張情報を削除する
  Future<void> delete(String tenantId, String itemId) async {
    await _repository.deleteTenantExtension(tenantId, itemId);
    state = const AsyncValue.data(null);
  }
}

/// テナント拡張のProvider
final tenantExtensionProvider = StateNotifierProvider<TenantExtensionNotifier,
    AsyncValue<TenantMasterExtension?>>(
  (ref) => TenantExtensionNotifier(ref.watch(domainMasterRepositoryProvider)),
);
