import 'package:dio/dio.dart';

import '../models/activity.dart';

/// アクティビティAPIのリポジトリ層
/// サーバーとの通信を担当し、アクティビティデータの永続化・取得を行う
class ActivityRepository {
  /// APIリクエストに使用するDioインスタンス
  final Dio _dio;

  /// コンストラクタ: Dioインスタンスを外部から注入する（テスタビリティのため）
  ActivityRepository(this._dio);

  // ========================================
  // アクティビティ取得操作
  // ========================================

  /// アクティビティ一覧を取得する
  /// [taskId] でタスクIDによるフィルタリングが可能
  /// [actorId] でアクターIDによるフィルタリングが可能
  /// [activityType] で種別によるフィルタリングが可能
  Future<List<Activity>> listActivities({
    String? taskId,
    String? actorId,
    ActivityType? activityType,
  }) async {
    /// クエリパラメータを動的に構築する
    final queryParameters = <String, dynamic>{};
    if (taskId != null) queryParameters['task_id'] = taskId;
    if (actorId != null) queryParameters['actor_id'] = actorId;
    if (activityType != null) queryParameters['activity_type'] = activityType.name;

    final response = await _dio.get(
      '/api/v1/activities',
      queryParameters: queryParameters,
    );
    final List<dynamic> data =
        (response.data as Map<String, dynamic>)['activities'] as List<dynamic>;
    return data
        .map((json) => Activity.fromJson(json as Map<String, dynamic>))
        .toList();
  }

  /// 指定IDのアクティビティを取得する
  Future<Activity> getActivity(String id) async {
    final response = await _dio.get('/api/v1/activities/$id');
    return Activity.fromJson(response.data as Map<String, dynamic>);
  }

  // ========================================
  // アクティビティ作成・承認フロー操作
  // ========================================

  /// 新規アクティビティを作成する
  /// 作成されたアクティビティをレスポンスから返す
  Future<Activity> createActivity(CreateActivityInput input) async {
    final response = await _dio.post(
      '/api/v1/activities',
      data: input.toJson(),
    );
    return Activity.fromJson(response.data as Map<String, dynamic>);
  }

  /// 指定IDのアクティビティを承認申請する
  /// ステータスを submitted に遷移させる
  Future<Activity> submitActivity(String id) async {
    final response = await _dio.post('/api/v1/activities/$id/submit', data: {});
    return Activity.fromJson(response.data as Map<String, dynamic>);
  }

  /// 指定IDのアクティビティを承認する
  /// ステータスを approved に遷移させる
  Future<Activity> approveActivity(String id) async {
    final response = await _dio.post('/api/v1/activities/$id/approve', data: {});
    return Activity.fromJson(response.data as Map<String, dynamic>);
  }

  /// 指定IDのアクティビティを却下する
  /// ステータスを rejected に遷移させる
  Future<Activity> rejectActivity(String id, RejectActivityInput input) async {
    final response = await _dio.post(
      '/api/v1/activities/$id/reject',
      data: input.toJson(),
    );
    return Activity.fromJson(response.data as Map<String, dynamic>);
  }
}
