import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';

import '../config/config_provider.dart';
import '../models/domain_master.dart';
import '../repositories/domain_master_repository.dart';
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
final domainMasterRepositoryProvider = Provider<DomainMasterRepository>((ref) {
  return DomainMasterRepository(ref.watch(dioProvider));
});

/// カテゴリ一覧の状態を管理するNotifier
/// CRUD操作とローディング/エラー状態を統一的に管理する
class CategoryListNotifier extends Notifier<AsyncValue<List<MasterCategory>>> {
  @override
  AsyncValue<List<MasterCategory>> build() {
    /// 初期化時にカテゴリ一覧を自動取得する
    load();
    return const AsyncValue.loading();
  }

  /// リポジトリをrefから取得するヘルパー
  DomainMasterRepository get _repository =>
      ref.read(domainMasterRepositoryProvider);

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
/// NotifierProviderを使用して状態管理を行う
final categoryListProvider =
    NotifierProvider<CategoryListNotifier, AsyncValue<List<MasterCategory>>>(
  CategoryListNotifier.new,
);

/// アイテム一覧の状態を管理するNotifier
/// カテゴリコードに紐づくアイテムのCRUD操作を管理する
/// familyプロバイダーのため、コンストラクタでカテゴリコードを受け取る
class ItemListNotifier extends Notifier<AsyncValue<List<MasterItem>>> {
  /// familyの引数（カテゴリコード）
  ItemListNotifier(this.arg);
  final String arg;

  @override
  AsyncValue<List<MasterItem>> build() {
    /// 初期化時にアイテム一覧を自動取得する
    load();
    return const AsyncValue.loading();
  }

  /// リポジトリをrefから取得するヘルパー
  DomainMasterRepository get _repository =>
      ref.read(domainMasterRepositoryProvider);

  /// アイテム一覧をサーバーから取得する
  Future<void> load({bool activeOnly = false}) async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(
      () => _repository.listItems(arg, activeOnly: activeOnly),
    );
  }

  /// 新規アイテムを作成し、一覧を再取得する
  Future<void> create(CreateItemInput input) async {
    await _repository.createItem(arg, input);
    await load();
  }

  /// アイテムを更新し、一覧を再取得する
  Future<void> update(String itemCode, UpdateItemInput input) async {
    await _repository.updateItem(arg, itemCode, input);
    await load();
  }

  /// アイテムを削除し、一覧を再取得する
  Future<void> delete(String itemCode) async {
    await _repository.deleteItem(arg, itemCode);
    await load();
  }
}

/// アイテム一覧のProviderファミリー
/// カテゴリコードごとに独立した状態を管理する
final itemListProvider = NotifierProvider.family<ItemListNotifier,
    AsyncValue<List<MasterItem>>, String>(
  ItemListNotifier.new,
);

/// バージョン履歴の状態を管理するNotifier
/// familyプロバイダーのため、コンストラクタでカテゴリコードとアイテムコードのレコードを受け取る
class VersionListNotifier
    extends Notifier<AsyncValue<List<MasterItemVersion>>> {
  /// familyの引数（カテゴリコードとアイテムコードの組み合わせ）
  VersionListNotifier(this.arg);
  final ({String categoryCode, String itemCode}) arg;

  @override
  AsyncValue<List<MasterItemVersion>> build() {
    /// 初期化時にバージョン履歴を自動取得する
    load();
    return const AsyncValue.loading();
  }

  /// リポジトリをrefから取得するヘルパー
  DomainMasterRepository get _repository =>
      ref.read(domainMasterRepositoryProvider);

  /// バージョン履歴をサーバーから取得する
  Future<void> load() async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(
      () => _repository.listVersions(arg.categoryCode, arg.itemCode),
    );
  }
}

/// バージョン履歴のProviderファミリー
/// カテゴリコードとアイテムコードの組み合わせで状態を管理する
final versionListProvider = NotifierProvider.family<VersionListNotifier,
    AsyncValue<List<MasterItemVersion>>, ({String categoryCode, String itemCode})>(
  VersionListNotifier.new,
);

/// テナント拡張の状態を管理するNotifier
class TenantExtensionNotifier
    extends Notifier<AsyncValue<TenantMasterExtension?>> {
  @override
  AsyncValue<TenantMasterExtension?> build() {
    return const AsyncValue.data(null);
  }

  /// リポジトリをrefから取得するヘルパー
  DomainMasterRepository get _repository =>
      ref.read(domainMasterRepositoryProvider);

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
final tenantExtensionProvider = NotifierProvider<TenantExtensionNotifier,
    AsyncValue<TenantMasterExtension?>>(
  TenantExtensionNotifier.new,
);
