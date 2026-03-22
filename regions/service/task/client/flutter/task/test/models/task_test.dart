/// task_test.dart: Taskモデルのユニットテスト。
/// fromJson/toJsonの往復変換、enum変換、nullable処理を検証する。
import 'package:flutter_test/flutter_test.dart';
import 'package:task/models/task.dart';

/// テスト用の基本的なTask JSONデータ
Map<String, dynamic> _sampleTaskJson() => {
      'id': '550e8400-e29b-41d4-a716-446655440001',
      'project_id': 'PROJ-001',
      'title': 'テストタスク',
      'description': null,
      'status': 'open',
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

void main() {
  group('Task.fromJson', () {
    /// 完全なJSONデータが正しくTaskインスタンスに変換されることを確認する
    test('必須フィールドが正しくパースされる', () {
      final task = Task.fromJson(_sampleTaskJson());

      expect(task.id, '550e8400-e29b-41d4-a716-446655440001');
      expect(task.projectId, 'PROJ-001');
      expect(task.title, 'テストタスク');
      expect(task.status, TaskStatus.open);
      expect(task.priority, TaskPriority.medium);
      expect(task.version, 1);
    });

    /// description が null のとき正しく null が設定されることを確認する
    test('nullable の description が null の場合に null が返る', () {
      final task = Task.fromJson(_sampleTaskJson());
      expect(task.description, isNull);
    });

    /// description に値がある場合に正しく設定されることを確認する
    test('description に値がある場合に値が返る', () {
      final json = _sampleTaskJson()..['description'] = 'タスクの詳細説明';
      final task = Task.fromJson(json);
      expect(task.description, 'タスクの詳細説明');
    });

    /// labels が正しく List<String> に変換されることを確認する
    test('labels が List<String> に変換される', () {
      final json = _sampleTaskJson()..['labels'] = ['bug', 'frontend'];
      final task = Task.fromJson(json);

      expect(task.labels, hasLength(2));
      expect(task.labels[0], 'bug');
      expect(task.labels[1], 'frontend');
    });

    /// in_progress ステータスが正しく変換されることを確認する
    test('in_progress ステータスが正しく変換される', () {
      final json = _sampleTaskJson()..['status'] = 'in_progress';
      final task = Task.fromJson(json);
      expect(task.status, TaskStatus.inProgress);
    });
  });

  group('Task.toJson', () {
    /// fromJson して toJson した結果が元のデータと一致することを確認する（往復変換）
    test('fromJson → toJson で往復変換が成立する', () {
      final original = _sampleTaskJson();
      final task = Task.fromJson(original);
      final json = task.toJson();

      expect(json['id'], original['id']);
      expect(json['project_id'], original['project_id']);
      expect(json['title'], original['title']);
      expect(json['status'], 'open');
      expect(json['priority'], 'medium');
      expect(json['version'], original['version']);
    });

    /// in_progress ステータスが 'in_progress' として出力されることを確認する
    test('in_progress ステータスが in_progress として出力される', () {
      final task = Task.fromJson(_sampleTaskJson()..['status'] = 'in_progress');
      final json = task.toJson();

      expect(json['status'], 'in_progress');
    });
  });

  group('TaskStatus', () {
    /// 全ステータス値が文字列から正しく変換されることを確認する
    test('全ステータス値が fromString で正しく変換される', () {
      expect(TaskStatus.fromString('open'), TaskStatus.open);
      expect(TaskStatus.fromString('in_progress'), TaskStatus.inProgress);
      expect(TaskStatus.fromString('review'), TaskStatus.review);
      expect(TaskStatus.fromString('done'), TaskStatus.done);
      expect(TaskStatus.fromString('cancelled'), TaskStatus.cancelled);
    });

    /// 不明な文字列のデフォルト値が open になることを確認する
    test('不明な文字列は open にフォールバックする', () {
      expect(TaskStatus.fromString('unknown_status'), TaskStatus.open);
    });

    /// 各ステータスに日本語表示名が設定されていることを確認する
    test('各ステータスに日本語表示名が設定されている', () {
      expect(TaskStatus.open.displayName, 'オープン');
      expect(TaskStatus.inProgress.displayName, '進行中');
      expect(TaskStatus.done.displayName, '完了');
      expect(TaskStatus.cancelled.displayName, 'キャンセル');
    });

    /// apiValue が正しい文字列を返すことを確認する
    test('apiValue が正しい文字列を返す', () {
      expect(TaskStatus.inProgress.apiValue, 'in_progress');
      expect(TaskStatus.open.apiValue, 'open');
    });
  });

  group('TaskPriority', () {
    /// 全優先度値が文字列から正しく変換されることを確認する
    test('全優先度値が fromString で正しく変換される', () {
      expect(TaskPriority.fromString('low'), TaskPriority.low);
      expect(TaskPriority.fromString('medium'), TaskPriority.medium);
      expect(TaskPriority.fromString('high'), TaskPriority.high);
      expect(TaskPriority.fromString('critical'), TaskPriority.critical);
    });

    /// 各優先度に日本語表示名が設定されていることを確認する
    test('各優先度に日本語表示名が設定されている', () {
      expect(TaskPriority.low.displayName, '低');
      expect(TaskPriority.medium.displayName, '中');
      expect(TaskPriority.high.displayName, '高');
      expect(TaskPriority.critical.displayName, '緊急');
    });
  });

  group('CreateTaskInput.toJson', () {
    /// 入力データが正しく JSON に変換されることを確認する
    test('入力データが正しく toJson に変換される', () {
      const input = CreateTaskInput(
        projectId: 'PROJ-001',
        title: 'テストタスク',
        priority: TaskPriority.high,
      );
      final json = input.toJson();

      expect(json['project_id'], 'PROJ-001');
      expect(json['title'], 'テストタスク');
      expect(json['priority'], 'high');
    });

    /// optionalフィールドが未指定の場合にJSONから除外されることを確認する
    test('optional フィールドが未指定の場合に JSON から除外される', () {
      const input = CreateTaskInput(
        projectId: 'PROJ-001',
        title: 'テストタスク',
      );
      final json = input.toJson();

      expect(json.containsKey('description'), isFalse);
      expect(json.containsKey('assignee_id'), isFalse);
    });
  });

  group('UpdateTaskStatusInput.toJson', () {
    /// ステータス更新データが正しく JSON に変換されることを確認する
    test('ステータス更新データが正しく toJson に変換される', () {
      const input = UpdateTaskStatusInput(status: TaskStatus.inProgress);
      final json = input.toJson();

      expect(json['status'], 'in_progress');
    });
  });
}
