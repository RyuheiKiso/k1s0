import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';
import 'package:system_client/system_client.dart';

import '../config/config_provider.dart';
import '../models/activity.dart';
import '../repositories/activity_repository.dart';

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
final activityRepositoryProvider = Provider<ActivityRepository>((ref) {
  return ActivityRepository(ref.watch(dioProvider));
});

/// アクティビティ一覧の状態を管理するNotifier
/// CRUD操作・承認フローとローディング/エラー状態を統一的に管理する
class ActivityListNotifier extends Notifier<AsyncValue<List<Activity>>> {
  @override
  AsyncValue<List<Activity>> build() {
    /// 初期化時にアクティビティ一覧を自動取得する
    load();
    return const AsyncValue.loading();
  }

  /// リポジトリをrefから取得するヘルパー
  ActivityRepository get _repository =>
      ref.read(activityRepositoryProvider);

  /// アクティビティ一覧をサーバーから取得する
  /// [taskId] でタスクIDによるフィルタリングが可能
  /// [actorId] でアクターIDによるフィルタリングが可能
  /// [activityType] で種別によるフィルタリングが可能
  Future<void> load({
    String? taskId,
    String? actorId,
    ActivityType? activityType,
  }) async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(
      () => _repository.listActivities(
        taskId: taskId,
        actorId: actorId,
        activityType: activityType,
      ),
    );
  }

  /// 新規アクティビティを作成し、一覧を再取得する
  Future<void> create(CreateActivityInput input) async {
    await _repository.createActivity(input);
    await load();
  }

  /// アクティビティを承認申請し、一覧を再取得する
  Future<void> submit(String id) async {
    await _repository.submitActivity(id);
    await load();
  }

  /// アクティビティを承認し、一覧を再取得する
  Future<void> approve(String id) async {
    await _repository.approveActivity(id);
    await load();
  }

  /// アクティビティを却下し、一覧を再取得する
  Future<void> reject(String id, RejectActivityInput input) async {
    await _repository.rejectActivity(id, input);
    await load();
  }
}

/// アクティビティ一覧のProvider
/// NotifierProviderを使用して状態管理を行う
final activityListProvider =
    NotifierProvider<ActivityListNotifier, AsyncValue<List<Activity>>>(
  ActivityListNotifier.new,
);
