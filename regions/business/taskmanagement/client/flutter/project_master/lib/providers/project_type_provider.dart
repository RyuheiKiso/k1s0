import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';
import 'package:system_client/system_client.dart' as sys;

import '../config/config_provider.dart';
import '../models/project_type.dart';
import '../repositories/project_type_repository.dart';
import '../utils/api_client.dart';

/// DioインスタンスのProvider
/// YAML設定ファイルから読み込んだベースURLを使用してHTTPクライアントを生成する
/// M-20 監査対応: csrfTokenProvider を明示的に注入して CSRF 保護を確実に有効化する
final dioProvider = Provider<Dio>((ref) {
  /// 設定プロバイダーからAPI設定を取得する
  final config = ref.watch(appConfigProvider);
  // authProvider から CSRF トークンを取得するプロバイダーを注入する
  // ref.read を使用して各リクエスト時に最新のトークンを取得する（キャッシュを避ける）
  final authNotifier = ref.read(sys.authProvider.notifier);
  return ApiClient.create(
    baseUrl: config.api.baseUrl,
    csrfTokenProvider: () async => authNotifier.csrfToken,
  );
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
