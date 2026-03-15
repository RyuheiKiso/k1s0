import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:dio/dio.dart';

import '../config/config_provider.dart';
import '../models/payment.dart';
import '../repositories/payment_repository.dart';
import '../utils/dio_client.dart';

/// DioインスタンスのProvider
/// YAML設定ファイルから読み込んだベースURLを使用してHTTPクライアントを生成する
final dioProvider = Provider<Dio>((ref) {
  /// 設定プロバイダーからAPI設定を取得する
  final config = ref.watch(appConfigProvider);
  return DioClient.create(baseUrl: config.api.baseUrl);
});

/// リポジトリのProvider
/// DioインスタンスをDIしてリポジトリを生成する
final paymentRepositoryProvider = Provider<PaymentRepository>((ref) {
  return PaymentRepository(ref.watch(dioProvider));
});

/// 決済一覧の状態を管理するStateNotifier
/// フィルタリングとローディング/エラー状態を統一的に管理する
class PaymentListNotifier extends StateNotifier<AsyncValue<List<Payment>>> {
  final PaymentRepository _repository;

  PaymentListNotifier(this._repository) : super(const AsyncValue.loading()) {
    /// 初期化時に決済一覧を自動取得する
    load();
  }

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
/// StateNotifierProviderを使用して状態管理を行う
final paymentListProvider =
    StateNotifierProvider<PaymentListNotifier, AsyncValue<List<Payment>>>(
  (ref) => PaymentListNotifier(ref.watch(paymentRepositoryProvider)),
);
