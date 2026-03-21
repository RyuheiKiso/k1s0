import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';
import 'package:system_client/system_client.dart';

import '../config/config_provider.dart';
import '../models/payment.dart';
import '../repositories/payment_repository.dart';

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
final paymentRepositoryProvider = Provider<PaymentRepository>((ref) {
  return PaymentRepository(ref.watch(dioProvider));
});

/// 決済一覧の状態を管理するNotifier
/// フィルタリングとローディング/エラー状態を統一的に管理する
class PaymentListNotifier extends Notifier<AsyncValue<List<Payment>>> {
  @override
  AsyncValue<List<Payment>> build() {
    /// 初期化時に決済一覧を自動取得する
    load();
    return const AsyncValue.loading();
  }

  /// リポジトリをrefから取得するヘルパー
  PaymentRepository get _repository =>
      ref.read(paymentRepositoryProvider);

  /// 決済一覧をサーバーから取得する
  /// オプションのフィルタ条件で絞り込みを行う
  Future<void> load({
    String? orderId,
    String? customerId,
    PaymentStatus? status,
  }) async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(
      () => _repository.listPayments(
        orderId: orderId,
        customerId: customerId,
        status: status,
      ),
    );
  }

  /// 新規決済を開始し、一覧を再取得する
  Future<void> initiate(InitiatePaymentInput input) async {
    await _repository.initiatePayment(input);
    await load();
  }

  /// 決済を完了し、一覧を再取得する
  Future<void> complete(String id) async {
    await _repository.completePayment(id);
    await load();
  }

  /// 決済を失敗にし、一覧を再取得する
  Future<void> fail(String id) async {
    await _repository.failPayment(id);
    await load();
  }

  /// 決済を返金し、一覧を再取得する
  Future<void> refund(String id) async {
    await _repository.refundPayment(id);
    await load();
  }
}

/// 決済一覧のProvider
/// NotifierProviderを使用して状態管理を行う
final paymentListProvider =
    NotifierProvider<PaymentListNotifier, AsyncValue<List<Payment>>>(
  PaymentListNotifier.new,
);
