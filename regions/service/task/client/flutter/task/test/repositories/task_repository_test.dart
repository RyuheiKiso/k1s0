/// task_repository_test.dart: TaskRepositoryのユニットテスト。
/// MockHttpClientAdapterを使用してAPI通信をモックし、
/// 各メソッドが正しくリクエストを送信・レスポンスを処理することを検証する。
library;
import 'dart:convert';

import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:task/models/task.dart';
import 'package:task/repositories/task_repository.dart';

/// テスト用のHTTPクライアントアダプター
class _MockHttpClientAdapter implements HttpClientAdapter {
  final String Function(RequestOptions) responseBodyFn;
  final int statusCode;

  // LOW-004 監査対応: statusCode をコンストラクタで初期化し、未初期化フィールドによる実行時エラーを防ぐ
  _MockHttpClientAdapter({
    required this.responseBodyFn,
    this.statusCode = 200, // ignore: unused_element_parameter
  });

  @override
  Future<ResponseBody> fetch(
    RequestOptions options,
    Stream<List<int>>? requestStream,
    Future<void>? cancelFuture,
  ) async {
    return ResponseBody.fromString(
      responseBodyFn(options),
      statusCode,
      headers: {
        'content-type': ['application/json'],
      },
    );
  }

  @override
  void close({bool force = false}) {}
}

/// テスト用のサンプルタスクJSONデータを生成する
Map<String, dynamic> _sampleTaskData({
  String id = 'TASK-001',
  String status = 'open',
}) =>
    {
      'id': id,
      'project_id': 'PROJ-001',
      'title': 'テストタスク',
      'description': null,
      'status': status,
      'priority': 'medium',
      'assignee_id': null,
      'reporter_id': 'USER-001',
      'due_date': null,
      'labels': <String>[],
      'created_by': 'USER-001',
      'updated_by': 'USER-001',
      'version': 1,
      'created_at': '2026-01-20T00:00:00.000Z',
      'updated_at': '2026-01-20T00:00:00.000Z',
    };

/// テスト用のDioインスタンスを生成する
Dio _createTestDio(String Function(RequestOptions) responseBodyFn) {
  final dio = Dio(BaseOptions(baseUrl: 'http://localhost:8080'));
  dio.httpClientAdapter = _MockHttpClientAdapter(responseBodyFn: responseBodyFn);
  return dio;
}

void main() {
  group('TaskRepository.listTasks', () {
    /// タスク一覧が正しく取得されることを確認する
    test('タスク一覧が正しく取得される', () async {
      final dio = _createTestDio((_) => jsonEncode({
            'tasks': [
              _sampleTaskData(id: 'TASK-001'),
              _sampleTaskData(id: 'TASK-002', status: 'in_progress'),
            ]
          }));
      final repo = TaskRepository(dio);

      final tasks = await repo.listTasks();

      expect(tasks, hasLength(2));
      expect(tasks[0].id, 'TASK-001');
      expect(tasks[1].id, 'TASK-002');
      expect(tasks[1].status, TaskStatus.inProgress);
    });

    /// 空のタスク一覧が返されたときに空リストになることを確認する
    test('空のタスク一覧が返される', () async {
      final dio = _createTestDio((_) => jsonEncode({'tasks': []}));
      final repo = TaskRepository(dio);

      final tasks = await repo.listTasks();

      expect(tasks, isEmpty);
    });
  });

  group('TaskRepository.getTask', () {
    /// 指定IDのタスクが正しく取得されることを確認する
    test('指定IDのタスクが正しく取得される', () async {
      final dio = _createTestDio((_) => jsonEncode(_sampleTaskData(id: 'TASK-001')));
      final repo = TaskRepository(dio);

      final task = await repo.getTask('TASK-001');

      expect(task.id, 'TASK-001');
      expect(task.status, TaskStatus.open);
      expect(task.labels, isEmpty);
    });
  });

  group('TaskRepository.createTask', () {
    /// 新規タスクが正しく作成されることを確認する
    test('新規タスクが正しく作成される', () async {
      final dio = _createTestDio((_) => jsonEncode(_sampleTaskData(id: 'TASK-NEW')));
      final repo = TaskRepository(dio);

      const input = CreateTaskInput(
        projectId: 'PROJ-001',
        title: 'テストタスク',
        priority: TaskPriority.high,
      );
      final task = await repo.createTask(input);

      expect(task.id, 'TASK-NEW');
    });
  });

  group('TaskRepository.updateTaskStatus', () {
    /// タスクステータスが正しく更新されることを確認する
    test('タスクステータスが正しく更新される', () async {
      final dio = _createTestDio(
          (_) => jsonEncode(_sampleTaskData(id: 'TASK-001', status: 'in_progress')));
      final repo = TaskRepository(dio);

      const input = UpdateTaskStatusInput(status: TaskStatus.inProgress);
      final task = await repo.updateTaskStatus('TASK-001', input);

      expect(task.id, 'TASK-001');
      expect(task.status, TaskStatus.inProgress);
    });
  });
}
