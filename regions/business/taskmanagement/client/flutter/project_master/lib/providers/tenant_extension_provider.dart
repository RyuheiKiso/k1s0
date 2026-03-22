import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/tenant_extension.dart';
import '../repositories/tenant_extension_repository.dart';
import 'project_type_provider.dart';

/// テナント拡張リポジトリのProvider
/// DioインスタンスをDIしてリポジトリを生成する
final tenantExtensionRepositoryProvider =
    Provider<TenantExtensionRepository>((ref) {
  return TenantExtensionRepository(ref.watch(dioProvider));
});

/// テナント拡張一覧の状態を管理するNotifier
/// テナントIDに紐づくステータス定義拡張のCRUD操作を管理する
class TenantExtensionListNotifier
    extends Notifier<AsyncValue<List<TenantProjectExtension>>> {
  /// familyの引数（テナントID）
  TenantExtensionListNotifier(this.arg);
  final String arg;

  @override
  AsyncValue<List<TenantProjectExtension>> build() {
    /// 初期化時にテナント拡張一覧を自動取得する
    load();
    return const AsyncValue.loading();
  }

  /// リポジトリをrefから取得するヘルパー
  TenantExtensionRepository get _repository =>
      ref.read(tenantExtensionRepositoryProvider);

  /// テナント拡張一覧をサーバーから取得する
  Future<void> load() async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(
      () => _repository.listTenantExtensions(arg),
    );
  }

  /// テナント拡張を更新（upsert）し、一覧を再取得する
  Future<void> upsert(
    String statusDefinitionId,
    UpdateTenantExtensionInput input,
  ) async {
    await _repository.upsertTenantExtension(arg, statusDefinitionId, input);
    await load();
  }

  /// テナント拡張を削除し、一覧を再取得する
  Future<void> delete(String statusDefinitionId) async {
    await _repository.deleteTenantExtension(arg, statusDefinitionId);
    await load();
  }
}

/// テナント拡張一覧のProviderファミリー
/// テナントIDごとに独立した状態を管理する
final tenantExtensionListProvider = NotifierProvider.family<
    TenantExtensionListNotifier,
    AsyncValue<List<TenantProjectExtension>>,
    String>(
  TenantExtensionListNotifier.new,
);

/// 単一テナント拡張の状態を管理するNotifier
class TenantExtensionNotifier
    extends Notifier<AsyncValue<TenantProjectExtension?>> {
  @override
  AsyncValue<TenantProjectExtension?> build() {
    return const AsyncValue.data(null);
  }

  /// リポジトリをrefから取得するヘルパー
  TenantExtensionRepository get _repository =>
      ref.read(tenantExtensionRepositoryProvider);

  /// テナント拡張情報をサーバーから取得する
  Future<void> load(String tenantId, String statusDefinitionId) async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(
      () => _repository.getTenantExtension(tenantId, statusDefinitionId),
    );
  }

  /// テナント拡張情報を更新する（upsert）
  Future<void> upsert(
    String tenantId,
    String statusDefinitionId,
    UpdateTenantExtensionInput input,
  ) async {
    await _repository.upsertTenantExtension(tenantId, statusDefinitionId, input);
    await load(tenantId, statusDefinitionId);
  }

  /// テナント拡張情報を削除する
  Future<void> delete(String tenantId, String statusDefinitionId) async {
    await _repository.deleteTenantExtension(tenantId, statusDefinitionId);
    state = const AsyncValue.data(null);
  }
}

/// 単一テナント拡張のProvider
final tenantExtensionProvider = NotifierProvider<TenantExtensionNotifier,
    AsyncValue<TenantProjectExtension?>>(
  TenantExtensionNotifier.new,
);
