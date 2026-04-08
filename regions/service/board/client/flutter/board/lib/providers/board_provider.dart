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
/// Riverpod v3 では FamilyAsyncNotifier が廃止されたため、
/// Notifier と NotifierProvider のパターンを使用する
/// projectId は load() メソッドの引数として渡す
class BoardColumnListNotifier extends Notifier<AsyncValue<List<BoardColumn>>> {
  @override
  AsyncValue<List<BoardColumn>> build() {
    /// 初期化時はローディング状態を返す（load()の明示的な呼び出しが必要）
    return const AsyncValue.loading();
  }

  /// リポジトリをrefから取得するヘルパー
  BoardRepository get _repository => ref.read(boardRepositoryProvider);

  /// カラム一覧をサーバーから再取得する（手動リフレッシュ用）
  Future<void> load(String projectId) async {
    state = const AsyncValue.loading();
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

/// ボードカラム一覧のProvider
/// NotifierProviderを使用して状態管理を行う（task/activity と同一パターン）
final boardColumnListProvider =
    NotifierProvider<BoardColumnListNotifier, AsyncValue<List<BoardColumn>>>(
  BoardColumnListNotifier.new,
);
