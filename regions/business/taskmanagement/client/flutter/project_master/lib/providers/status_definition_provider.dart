import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/status_definition.dart';
import '../repositories/status_definition_repository.dart';
import 'project_type_provider.dart';

/// ステータス定義リポジトリのProvider
/// DioインスタンスをDIしてリポジトリを生成する
final statusDefinitionRepositoryProvider =
    Provider<StatusDefinitionRepository>((ref) {
  return StatusDefinitionRepository(ref.watch(dioProvider));
});

/// ステータス定義一覧の状態を管理するNotifier
/// プロジェクトタイプIDに紐づくステータス定義のCRUD操作を管理する
/// familyプロバイダーのため、コンストラクタでプロジェクトタイプIDを受け取る
class StatusDefinitionListNotifier
    extends Notifier<AsyncValue<List<StatusDefinition>>> {
  /// familyの引数（プロジェクトタイプID）
  StatusDefinitionListNotifier(this.arg);
  final String arg;

  @override
  AsyncValue<List<StatusDefinition>> build() {
    /// 初期化時にステータス定義一覧を自動取得する
    load();
    return const AsyncValue.loading();
  }

  /// リポジトリをrefから取得するヘルパー
  StatusDefinitionRepository get _repository =>
      ref.read(statusDefinitionRepositoryProvider);

  /// ステータス定義一覧をサーバーから取得する
  Future<void> load() async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(
      () => _repository.listStatusDefinitions(arg),
    );
  }

  /// 新規ステータス定義を作成し、一覧を再取得する
  Future<void> create(CreateStatusDefinitionInput input) async {
    await _repository.createStatusDefinition(arg, input);
    await load();
  }

  /// ステータス定義を更新し、一覧を再取得する
  Future<void> update(String id, UpdateStatusDefinitionInput input) async {
    await _repository.updateStatusDefinition(arg, id, input);
    await load();
  }

  /// ステータス定義を削除し、一覧を再取得する
  Future<void> delete(String id) async {
    await _repository.deleteStatusDefinition(arg, id);
    await load();
  }
}

/// ステータス定義一覧のProviderファミリー
/// プロジェクトタイプIDごとに独立した状態を管理する
final statusDefinitionListProvider = NotifierProvider.family<
    StatusDefinitionListNotifier,
    AsyncValue<List<StatusDefinition>>,
    String>(
  StatusDefinitionListNotifier.new,
);

/// バージョン履歴の状態を管理するNotifier
/// familyプロバイダーのため、コンストラクタでステータス定義IDを受け取る
class VersionListNotifier
    extends Notifier<AsyncValue<List<StatusDefinitionVersion>>> {
  /// familyの引数（ステータス定義ID）
  VersionListNotifier(this.arg);
  final String arg;

  @override
  AsyncValue<List<StatusDefinitionVersion>> build() {
    /// 初期化時にバージョン履歴を自動取得する
    load();
    return const AsyncValue.loading();
  }

  /// リポジトリをrefから取得するヘルパー
  StatusDefinitionRepository get _repository =>
      ref.read(statusDefinitionRepositoryProvider);

  /// バージョン履歴をサーバーから取得する
  Future<void> load() async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(
      () => _repository.listVersions(arg),
    );
  }
}

/// バージョン履歴のProviderファミリー
/// ステータス定義IDごとに独立した状態を管理する
final versionListProvider = NotifierProvider.family<VersionListNotifier,
    AsyncValue<List<StatusDefinitionVersion>>, String>(
  VersionListNotifier.new,
);
