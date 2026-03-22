import 'package:dio/dio.dart';

import '../models/task.dart';

/// タスクAPIのリポジトリ層
/// サーバーとの通信を担当し、タスクデータの永続化・取得を行う
class TaskRepository {
  /// APIリクエストに使用するDioインスタンス
  final Dio _dio;

  /// コンストラクタ: Dioインスタンスを外部から注入する（テスタビリティのため）
  TaskRepository(this._dio);

  // ========================================
  // タスク操作
  // ========================================

  /// タスク一覧を取得する
  /// [projectId] でプロジェクトIDによるフィルタリングが可能
  /// [status] でステータスによるフィルタリングが可能
  /// [assigneeId] で担当者IDによるフィルタリングが可能
  Future<List<Task>> listTasks({
    String? projectId,
    TaskStatus? status,
    String? assigneeId,
  }) async {
    /// クエリパラメータを動的に構築する
    final queryParameters = <String, dynamic>{};
    if (projectId != null) queryParameters['project_id'] = projectId;
    if (status != null) queryParameters['status'] = status.apiValue;
    if (assigneeId != null) queryParameters['assignee_id'] = assigneeId;

    final response = await _dio.get(
      '/api/v1/tasks',
      queryParameters: queryParameters,
    );
    final List<dynamic> data =
        (response.data as Map<String, dynamic>)['tasks'] as List<dynamic>;
    return data
        .map((json) => Task.fromJson(json as Map<String, dynamic>))
        .toList();
  }

  /// 指定IDのタスクを取得する
  Future<Task> getTask(String id) async {
    final response = await _dio.get('/api/v1/tasks/$id');
    return Task.fromJson(response.data as Map<String, dynamic>);
  }

  /// 新規タスクを作成する
  /// 作成されたタスクをレスポンスから返す
  Future<Task> createTask(CreateTaskInput input) async {
    final response = await _dio.post(
      '/api/v1/tasks',
      data: input.toJson(),
    );
    return Task.fromJson(response.data as Map<String, dynamic>);
  }

  /// 指定IDのタスクステータスを更新する
  Future<Task> updateTaskStatus(
    String id,
    UpdateTaskStatusInput input,
  ) async {
    final response = await _dio.put(
      '/api/v1/tasks/$id/status',
      data: input.toJson(),
    );
    return Task.fromJson(response.data as Map<String, dynamic>);
  }
}
