import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';

import '../config/app_config.dart';
import '../config/config_provider.dart';
import '../models/project_type.dart';
import '../repositories/project_type_repository.dart';
import '../utils/api_client.dart';

/// DioインスタンスのProvider
/// YAML設定ファイルから読み込んだベースURLを使用してHTTPクライアントを生成する
final dioProvider = Provider<Dio>((ref) {
  /// 設定プロバイダーからAPI設定を取得する
  final config = ref.watch(appConfigProvider);
  return ApiClient.create(baseUrl: config.api.baseUrl);
});

/// プロジェクトタイプリポジトリのProvider
/// DioインスタンスをDIしてリポジトリを生成する
final projectTypeRepositoryProvider = Provider<ProjectTypeRepository>((ref) {
  return ProjectTypeRepository(ref.watch(dioProvider));
});

/// プロジェクトタイプ一覧の状態を管理するNotifier
/// CRUD操作とローディング/エラー状態を統一的に管理する
class ProjectTypeListNotifier extends Notifier<AsyncValue<List<ProjectType>>> {
  @override
  AsyncValue<List<ProjectType>> build() {
    /// 初期化時にプロジェクトタイプ一覧を自動取得する
    load();
    return const AsyncValue.loading();
  }

  /// リポジトリをrefから取得するヘルパー
  ProjectTypeRepository get _repository =>
      ref.read(projectTypeRepositoryProvider);

  /// プロジェクトタイプ一覧をサーバーから取得する
  Future<void> load({bool activeOnly = false}) async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(
      () => _repository.listProjectTypes(activeOnly: activeOnly),
    );
  }

  /// 新規プロジェクトタイプを作成し、一覧を再取得する
  Future<void> create(CreateProjectTypeInput input) async {
    await _repository.createProjectType(input);
    await load();
  }

  /// プロジェクトタイプを更新し、一覧を再取得する
  Future<void> update(String id, UpdateProjectTypeInput input) async {
    await _repository.updateProjectType(id, input);
    await load();
  }

  /// プロジェクトタイプを削除し、一覧を再取得する
  Future<void> delete(String id) async {
    await _repository.deleteProjectType(id);
    await load();
  }
}

/// プロジェクトタイプ一覧のProvider
/// NotifierProviderを使用して状態管理を行う
final projectTypeListProvider =
    NotifierProvider<ProjectTypeListNotifier, AsyncValue<List<ProjectType>>>(
  ProjectTypeListNotifier.new,
);
