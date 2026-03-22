/// activity_test.dart: Activityモデルのユニットテスト。
/// fromJson/toJsonの往復変換、enum変換、nullable処理を検証する。
import 'package:flutter_test/flutter_test.dart';
import 'package:activity/models/activity.dart';

/// テスト用の基本的なActivity JSONデータ
Map<String, dynamic> _sampleActivityJson() => {
      'id': '550e8400-e29b-41d4-a716-446655440001',
      'task_id': 'TASK-001',
      'actor_id': 'USER-001',
      'activity_type': 'comment',
      'content': 'テストコメントです',
      'duration_minutes': null,
      'status': 'active',
      'metadata': null,
      'idempotency_key': null,
      'version': 1,
      'created_at': '2024-01-20T00:00:00.000Z',
      'updated_at': '2024-01-20T00:00:00.000Z',
    };

void main() {
  group('Activity.fromJson', () {
    /// 完全なJSONデータが正しくActivityインスタンスに変換されることを確認する
    test('必須フィールドが正しくパースされる', () {
      final activity = Activity.fromJson(_sampleActivityJson());

      expect(activity.id, '550e8400-e29b-41d4-a716-446655440001');
      expect(activity.taskId, 'TASK-001');
      expect(activity.actorId, 'USER-001');
      expect(activity.activityType, ActivityType.comment);
      expect(activity.status, ActivityStatus.active);
      expect(activity.version, 1);
    });

    /// content が null のとき正しく null が設定されることを確認する
    test('nullable の content が null の場合に null が返る', () {
      final json = _sampleActivityJson()..['content'] = null;
      final activity = Activity.fromJson(json);
      expect(activity.content, isNull);
    });

    /// content に値がある場合に正しく設定されることを確認する
    test('content に値がある場合に値が返る', () {
      final activity = Activity.fromJson(_sampleActivityJson());
      expect(activity.content, 'テストコメントです');
    });

    /// duration_minutes が null のとき正しく null が設定されることを確認する
    test('nullable の duration_minutes が null の場合に null が返る', () {
      final activity = Activity.fromJson(_sampleActivityJson());
      expect(activity.durationMinutes, isNull);
    });

    /// duration_minutes に値がある場合に正しく設定されることを確認する
    test('duration_minutes に値がある場合に値が返る', () {
      final json = _sampleActivityJson()..['duration_minutes'] = 90;
      final activity = Activity.fromJson(json);
      expect(activity.durationMinutes, 90);
    });
  });

  group('Activity.toJson', () {
    /// fromJson して toJson した結果が元のデータと一致することを確認する（往復変換）
    test('fromJson → toJson で往復変換が成立する', () {
      final original = _sampleActivityJson();
      final activity = Activity.fromJson(original);
      final json = activity.toJson();

      expect(json['id'], original['id']);
      expect(json['task_id'], original['task_id']);
      expect(json['actor_id'], original['actor_id']);
      expect(json['activity_type'], 'comment');
      expect(json['status'], 'active');
      expect(json['version'], original['version']);
    });
  });

  group('ActivityStatus', () {
    /// 全ステータス値が文字列から正しく変換されることを確認する
    test('全ステータス値が fromString で正しく変換される', () {
      expect(ActivityStatus.fromString('active'), ActivityStatus.active);
      expect(ActivityStatus.fromString('submitted'), ActivityStatus.submitted);
      expect(ActivityStatus.fromString('approved'), ActivityStatus.approved);
      expect(ActivityStatus.fromString('rejected'), ActivityStatus.rejected);
    });

    /// 不明な文字列のデフォルト値が active になることを確認する
    test('不明な文字列は active にフォールバックする', () {
      expect(ActivityStatus.fromString('unknown_status'), ActivityStatus.active);
    });

    /// 各ステータスに日本語表示名が設定されていることを確認する
    test('各ステータスに日本語表示名が設定されている', () {
      expect(ActivityStatus.active.displayName, 'アクティブ');
      expect(ActivityStatus.submitted.displayName, '申請中');
      expect(ActivityStatus.approved.displayName, '承認済み');
      expect(ActivityStatus.rejected.displayName, '却下済み');
    });
  });

  group('ActivityType', () {
    /// 全種別値が文字列から正しく変換されることを確認する
    test('全種別値が fromString で正しく変換される', () {
      expect(ActivityType.fromString('comment'), ActivityType.comment);
      expect(ActivityType.fromString('time_entry'), ActivityType.time_entry);
      expect(ActivityType.fromString('status_change'), ActivityType.status_change);
      expect(ActivityType.fromString('assignment'), ActivityType.assignment);
    });

    /// 不明な文字列のデフォルト値が comment になることを確認する
    test('不明な文字列は comment にフォールバックする', () {
      expect(ActivityType.fromString('unknown'), ActivityType.comment);
    });

    /// 各種別に日本語表示名が設定されていることを確認する
    test('各種別に日本語表示名が設定されている', () {
      expect(ActivityType.comment.displayName, 'コメント');
      expect(ActivityType.time_entry.displayName, '作業時間');
      expect(ActivityType.status_change.displayName, 'ステータス変更');
      expect(ActivityType.assignment.displayName, '担当割当');
    });
  });

  group('CreateActivityInput.toJson', () {
    /// 入力データが正しく JSON に変換されることを確認する
    test('入力データが正しく toJson に変換される', () {
      const input = CreateActivityInput(
        taskId: 'TASK-001',
        actorId: 'USER-001',
        activityType: ActivityType.time_entry,
        content: '作業内容',
        durationMinutes: 60,
      );
      final json = input.toJson();

      expect(json['task_id'], 'TASK-001');
      expect(json['actor_id'], 'USER-001');
      expect(json['activity_type'], 'time_entry');
      expect(json['content'], '作業内容');
      expect(json['duration_minutes'], 60);
    });

    /// オプションフィールドが未指定の場合に JSON に含まれないことを確認する
    test('オプションフィールドが未指定の場合に JSON に含まれない', () {
      const input = CreateActivityInput(
        taskId: 'TASK-001',
        actorId: 'USER-001',
        activityType: ActivityType.comment,
      );
      final json = input.toJson();

      expect(json.containsKey('content'), isFalse);
      expect(json.containsKey('duration_minutes'), isFalse);
    });
  });

  group('RejectActivityInput.toJson', () {
    /// 理由ありの却下データが正しく JSON に変換されることを確認する
    test('理由ありの却下データが正しく toJson に変換される', () {
      const input = RejectActivityInput(reason: '内容が不十分です');
      final json = input.toJson();

      expect(json['reason'], '内容が不十分です');
    });

    /// 理由なしの却下データが空の JSON に変換されることを確認する
    test('理由なしの却下データが空の JSON に変換される', () {
      const input = RejectActivityInput();
      final json = input.toJson();

      expect(json.containsKey('reason'), isFalse);
    });
  });
}
