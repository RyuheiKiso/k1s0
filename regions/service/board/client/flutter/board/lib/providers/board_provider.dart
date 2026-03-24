import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';
import 'package:system_client/system_client.dart';

import '../config/config_provider.dart';
import '../models/board_column.dart';
import '../repositories/board_repository.dart';

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
final boardRepositoryProvider = Provider<BoardRepository>((ref) {
  return BoardRepository(ref.watch(dioProvider));
});

/// ボードカラム一覧の状態を管理するNotifier
/// Riverpod v3 の FamilyAsyncNotifier を使用し、AsyncValue の手動ラップを排除する
/// （報告書 4.4 対応: FamilyNotifier<AsyncValue<T>> アンチパターン修正）
class BoardColumnListNotifier extends FamilyAsyncNotifier<List<BoardColumn>, String> {
  @override
  FutureOr<List<BoardColumn>> build(String projectId) async {
    /// 初期化時にカラム一覧をサーバーから取得して返す
    /// Riverpod が自動で AsyncValue でラップするため手動ラップ不要
    return await _repository.listColumns(projectId);
  }

  /// リポジトリをrefから取得するヘルパー
  BoardRepository get _repository => ref.read(boardRepositoryProvider);

  /// カラム一覧をサーバーから再取得する（手動リフレッシュ用）
  Future<void> load(String projectId) async {
    state = const AsyncLoading();
    state = await AsyncValue.guard(
      () => _repository.listColumns(projectId),
    );
  }

  /// カラムのタスク数をインクリメントし、一覧を再取得する
  Future<void> increment(String projectId, String statusCode) async {
    await _repository.incrementColumn(IncrementColumnInput(
      projectId: projectId,
      statusCode: statusCode,
    ));
    await load(projectId);
  }

  /// カラムのタスク数をデクリメントし、一覧を再取得する
  Future<void> decrement(String projectId, String statusCode) async {
    await _repository.decrementColumn(DecrementColumnInput(
      projectId: projectId,
      statusCode: statusCode,
    ));
    await load(projectId);
  }

  /// カラムのWIP制限を更新し、一覧を再取得する
  Future<void> updateWipLimit(
    String projectId,
    String statusCode,
    UpdateWipLimitInput input,
  ) async {
    await _repository.updateWipLimit(projectId, statusCode, input);
    await load(projectId);
  }
}

/// ボードカラム一覧のProvider（プロジェクトIDをファミリーキーとして使用）
/// AsyncNotifierProviderFamily を使用し、状態は AsyncValue<List<BoardColumn>> として提供される
final boardColumnListProvider = AsyncNotifierProviderFamily<
    BoardColumnListNotifier, List<BoardColumn>, String>(
  BoardColumnListNotifier.new,
);
