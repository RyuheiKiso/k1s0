import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';
import 'package:system_client/system_client.dart';

import '../config/config_provider.dart';
import '../models/task.dart';
import '../repositories/task_repository.dart';

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
final taskRepositoryProvider = Provider<TaskRepository>((ref) {
  return TaskRepository(ref.watch(dioProvider));
});

/// タスク一覧の状態を管理するNotifier
/// CRUD操作とローディング/エラー状態を統一的に管理する
class TaskListNotifier extends Notifier<AsyncValue<List<Task>>> {
  @override
  AsyncValue<List<Task>> build() {
    /// 初期化時にタスク一覧を自動取得する
    load();
    return const AsyncValue.loading();
  }

  /// リポジトリをrefから取得するヘルパー
  TaskRepository get _repository => ref.read(taskRepositoryProvider);

  /// タスク一覧をサーバーから取得する
  /// [projectId] でプロジェクトIDによるフィルタリングが可能
  /// [status] でステータスによるフィルタリングが可能
  /// [assigneeId] で担当者IDによるフィルタリングが可能
  Future<void> load({
    String? projectId,
    TaskStatus? status,
    String? assigneeId,
  }) async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(
      () => _repository.listTasks(
        projectId: projectId,
        status: status,
        assigneeId: assigneeId,
      ),
    );
  }

  /// 新規タスクを作成し、一覧を再取得する
  Future<void> create(CreateTaskInput input) async {
    await _repository.createTask(input);
    await load();
  }

  /// タスクステータスを更新し、一覧を再取得する
  Future<void> updateStatus(String id, UpdateTaskStatusInput input) async {
    await _repository.updateTaskStatus(id, input);
    await load();
  }
}

/// タスク一覧のProvider
/// NotifierProviderを使用して状態管理を行う
final taskListProvider =
    NotifierProvider<TaskListNotifier, AsyncValue<List<Task>>>(
  TaskListNotifier.new,
);
