/// activity_repository_test.dart: ActivityRepositoryのユニットテスト。
/// MockHttpClientAdapterを使用してAPI通信をモックし、
/// 各メソッドが正しくリクエストを送信・レスポンスを処理することを検証する。
import 'dart:convert';

import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:activity/models/activity.dart';
import 'package:activity/repositories/activity_repository.dart';

/// テスト用のHTTPクライアントアダプター
class _MockHttpClientAdapter implements HttpClientAdapter {
  final String Function(RequestOptions) responseBodyFn;
  final int statusCode;

  _MockHttpClientAdapter({
    required this.responseBodyFn,
    this.statusCode = 200,
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

/// テスト用のサンプルアクティビティJSONデータを生成する
Map<String, dynamic> _sampleActivityData({
  String id = 'ACTIVITY-001',
  String status = 'active',
  String activityType = 'comment',
}) =>
    {
      'id': id,
      'task_id': 'TASK-001',
      'actor_id': 'USER-001',
      'activity_type': activityType,
      'content': 'テストコメント',
      'duration_minutes': null,
      'status': status,
      'metadata': null,
      'idempotency_key': null,
      'version': 1,
      'created_at': '2024-01-20T00:00:00.000Z',
      'updated_at': '2024-01-20T00:00:00.000Z',
    };

/// テスト用のDioインスタンスを生成する
Dio _createTestDio(String Function(RequestOptions) responseBodyFn) {
  final dio = Dio(BaseOptions(baseUrl: 'http://localhost:8080'));
  dio.httpClientAdapter = _MockHttpClientAdapter(responseBodyFn: responseBodyFn);
  return dio;
}

void main() {
  group('ActivityRepository.listActivities', () {
    /// アクティビティ一覧が正しく取得されることを確認する
    test('アクティビティ一覧が正しく取得される', () async {
      final dio = _createTestDio((_) => jsonEncode({
            'activities': [
              _sampleActivityData(id: 'ACTIVITY-001'),
              _sampleActivityData(id: 'ACTIVITY-002', status: 'submitted'),
            ]
          }));
      final repo = ActivityRepository(dio);

      final activities = await repo.listActivities();

      expect(activities, hasLength(2));
      expect(activities[0].id, 'ACTIVITY-001');
      expect(activities[1].id, 'ACTIVITY-002');
      expect(activities[1].status, ActivityStatus.submitted);
    });

    /// 空のアクティビティ一覧が返されたときに空リストになることを確認する
    test('空のアクティビティ一覧が返される', () async {
      final dio = _createTestDio((_) => jsonEncode({'activities': []}));
      final repo = ActivityRepository(dio);

      final activities = await repo.listActivities();

      expect(activities, isEmpty);
    });
  });

  group('ActivityRepository.getActivity', () {
    /// 指定IDのアクティビティが正しく取得されることを確認する
    test('指定IDのアクティビティが正しく取得される', () async {
      final dio = _createTestDio(
          (_) => jsonEncode(_sampleActivityData(id: 'ACTIVITY-001')));
      final repo = ActivityRepository(dio);

      final activity = await repo.getActivity('ACTIVITY-001');

      expect(activity.id, 'ACTIVITY-001');
      expect(activity.status, ActivityStatus.active);
      expect(activity.activityType, ActivityType.comment);
    });
  });

  group('ActivityRepository.createActivity', () {
    /// 新規アクティビティが正しく作成されることを確認する
    test('新規アクティビティが正しく作成される', () async {
      final dio = _createTestDio(
          (_) => jsonEncode(_sampleActivityData(id: 'ACTIVITY-NEW')));
      final repo = ActivityRepository(dio);

      const input = CreateActivityInput(
        taskId: 'TASK-001',
        actorId: 'USER-001',
        activityType: ActivityType.comment,
        content: 'テストコメント',
      );
      final activity = await repo.createActivity(input);

      expect(activity.id, 'ACTIVITY-NEW');
    });
  });

  group('ActivityRepository.submitActivity', () {
    /// アクティビティが正しく承認申請されることを確認する
    test('アクティビティが正しく承認申請される', () async {
      final dio = _createTestDio(
          (_) => jsonEncode(_sampleActivityData(id: 'ACTIVITY-001', status: 'submitted')));
      final repo = ActivityRepository(dio);

      final activity = await repo.submitActivity('ACTIVITY-001');

      expect(activity.id, 'ACTIVITY-001');
      expect(activity.status, ActivityStatus.submitted);
    });
  });

  group('ActivityRepository.approveActivity', () {
    /// アクティビティが正しく承認されることを確認する
    test('アクティビティが正しく承認される', () async {
      final dio = _createTestDio(
          (_) => jsonEncode(_sampleActivityData(id: 'ACTIVITY-001', status: 'approved')));
      final repo = ActivityRepository(dio);

      final activity = await repo.approveActivity('ACTIVITY-001');

      expect(activity.id, 'ACTIVITY-001');
      expect(activity.status, ActivityStatus.approved);
    });
  });

  group('ActivityRepository.rejectActivity', () {
    /// アクティビティが正しく却下されることを確認する
    test('アクティビティが正しく却下される', () async {
      final dio = _createTestDio(
          (_) => jsonEncode(_sampleActivityData(id: 'ACTIVITY-001', status: 'rejected')));
      final repo = ActivityRepository(dio);

      const input = RejectActivityInput(reason: '内容が不十分です');
      final activity = await repo.rejectActivity('ACTIVITY-001', input);

      expect(activity.id, 'ACTIVITY-001');
      expect(activity.status, ActivityStatus.rejected);
    });
  });
}
