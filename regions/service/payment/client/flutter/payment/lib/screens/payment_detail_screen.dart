import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/payment.dart';
import '../providers/payment_provider.dart';

/// 決済詳細画面
/// 特定の決済情報の閲覧とステータス操作を行う画面
class PaymentDetailScreen extends ConsumerStatefulWidget {
  /// 対象決済のID
  final String paymentId;

  const PaymentDetailScreen({super.key, required this.paymentId});

  @override
  ConsumerState<PaymentDetailScreen> createState() =>
      _PaymentDetailScreenState();
}

class _PaymentDetailScreenState extends ConsumerState<PaymentDetailScreen> {
  /// 決済データの非同期状態
  AsyncValue<Payment?> _paymentAsync = const AsyncValue.loading();

  @override
  void initState() {
    super.initState();
    /// 初期化時に決済データを取得する
    _loadPayment();
  }

  /// 決済データをサーバーから取得する
  Future<void> _loadPayment() async {
    setState(() => _paymentAsync = const AsyncValue.loading());
    try {
      final repository = ref.read(paymentRepositoryProvider);
      final payment = await repository.getPayment(widget.paymentId);
      setState(() => _paymentAsync = AsyncValue.data(payment));
    } catch (e, stack) {
      setState(() => _paymentAsync = AsyncValue.error(e, stack));
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('決済詳細'),
        actions: [
          /// 詳細を再読み込みするボタン
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: '更新',
            onPressed: _loadPayment,
          ),
        ],
      ),
      body: _paymentAsync.when(
        /// ローディング中はプログレスインジケーターを表示する
        loading: () => const Center(child: CircularProgressIndicator()),
        /// エラー時はエラーメッセージとリトライボタンを表示する
        error: (error, stack) => Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('エラーが発生しました: $error'),
              const SizedBox(height: 16),
              ElevatedButton(
                onPressed: _loadPayment,
                child: const Text('再試行'),
              ),
            ],
          ),
        ),
        /// データ取得成功時は決済詳細を表示する
        data: (payment) {
          if (payment == null) {
            return const Center(child: Text('決済が見つかりません'));
          }
          return _buildPaymentDetail(payment);
        },
      ),
    );
  }

  /// 決済詳細ウィジェットを構築する
  Widget _buildPaymentDetail(Payment payment) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          /// 決済情報カード
          _buildInfoCard(payment),
          const SizedBox(height: 16),
          /// アクションボタンカード（ステータスに応じて表示）
          _buildActionCard(payment),
        ],
      ),
    );
  }

  /// 決済情報カードを構築する
  /// 決済の全情報を一覧表示する
  Widget _buildInfoCard(Payment payment) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              '決済情報',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            _buildInfoRow('決済ID', payment.id),
            _buildInfoRow('注文ID', payment.orderId),
            _buildInfoRow('顧客ID', payment.customerId),
            _buildInfoRow(
              '金額',
              '${payment.amount.toStringAsFixed(0)} ${payment.currency}',
            ),
            _buildInfoRow('ステータス', payment.status.label),
            _buildInfoRow('決済方法', payment.paymentMethod.label),
            if (payment.transactionId != null)
              _buildInfoRow('トランザクションID', payment.transactionId!),
            if (payment.failureReason != null)
              _buildInfoRow('失敗理由', payment.failureReason!),
            if (payment.refundAmount != null)
              _buildInfoRow(
                '返金額',
                '${payment.refundAmount!.toStringAsFixed(0)} ${payment.currency}',
              ),
            _buildInfoRow('作成日時', _formatDateTime(payment.createdAt)),
            _buildInfoRow('更新日時', _formatDateTime(payment.updatedAt)),
          ],
        ),
      ),
    );
  }

  /// アクションボタンカードを構築する
  /// 決済ステータスに応じて利用可能な操作ボタンを表示する
  Widget _buildActionCard(Payment payment) {
    /// 現在のステータスで利用可能なアクションがない場合は空ウィジェットを返す
    final hasActions = payment.status == PaymentStatus.pending ||
        payment.status == PaymentStatus.processing ||
        payment.status == PaymentStatus.completed;

    if (!hasActions) {
      return const SizedBox.shrink();
    }

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              '操作',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 16),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                /// 保留中または処理中の場合: 完了ボタンを表示する
                if (payment.status == PaymentStatus.pending ||
                    payment.status == PaymentStatus.processing)
                  FilledButton.icon(
                    onPressed: () => _showConfirmDialog(
                      title: '決済完了',
                      message: 'この決済を完了しますか？',
                      onConfirm: () => _completePayment(payment.id),
                    ),
                    icon: const Icon(Icons.check_circle),
                    label: const Text('完了'),
                  ),

                /// 保留中または処理中の場合: 失敗ボタンを表示する
                if (payment.status == PaymentStatus.pending ||
                    payment.status == PaymentStatus.processing)
                  FilledButton.icon(
                    style: FilledButton.styleFrom(
                      backgroundColor: Colors.red,
                    ),
                    onPressed: () => _showConfirmDialog(
                      title: '決済失敗',
                      message: 'この決済を失敗にしますか？',
                      onConfirm: () => _failPayment(payment.id),
                    ),
                    icon: const Icon(Icons.cancel),
                    label: const Text('失敗'),
                  ),

                /// 完了の場合: 返金ボタンを表示する
                if (payment.status == PaymentStatus.completed)
                  FilledButton.icon(
                    style: FilledButton.styleFrom(
                      backgroundColor: Colors.purple,
                    ),
                    onPressed: () => _showConfirmDialog(
                      title: '返金',
                      message: 'この決済を返金しますか？',
                      onConfirm: () => _refundPayment(payment.id),
                    ),
                    icon: const Icon(Icons.undo),
                    label: const Text('返金'),
                  ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  /// 確認ダイアログを表示する
  /// アクション実行前にユーザーに確認を求める
  void _showConfirmDialog({
    required String title,
    required String message,
    required VoidCallback onConfirm,
  }) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(title),
        content: Text(message),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('キャンセル'),
          ),
          FilledButton(
            onPressed: () {
              Navigator.of(context).pop();
              onConfirm();
            },
            child: const Text('実行'),
          ),
        ],
      ),
    );
  }

  /// 決済を完了する
  Future<void> _completePayment(String id) async {
    try {
      await ref.read(paymentRepositoryProvider).completePayment(id);
      await _loadPayment();
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('決済を完了しました')),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('エラー: $e')),
        );
      }
    }
  }

  /// 決済を失敗にする
  Future<void> _failPayment(String id) async {
    try {
      await ref.read(paymentRepositoryProvider).failPayment(id);
      await _loadPayment();
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('決済を失敗にしました')),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('エラー: $e')),
        );
      }
    }
  }

  /// 決済を返金する
  Future<void> _refundPayment(String id) async {
    try {
      await ref.read(paymentRepositoryProvider).refundPayment(id);
      await _loadPayment();
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('返金を実行しました')),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('エラー: $e')),
        );
      }
    }
  }

  /// 情報表示用の行ウィジェットを構築する
  Widget _buildInfoRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 160,
            child: Text(
              label,
              style: const TextStyle(fontWeight: FontWeight.bold),
            ),
          ),
          Expanded(child: Text(value)),
        ],
      ),
    );
  }

  /// DateTimeを読みやすい日本語形式にフォーマットする
  String _formatDateTime(DateTime dateTime) {
    return '${dateTime.year}/${dateTime.month.toString().padLeft(2, '0')}/'
        '${dateTime.day.toString().padLeft(2, '0')} '
        '${dateTime.hour.toString().padLeft(2, '0')}:'
        '${dateTime.minute.toString().padLeft(2, '0')}';
  }
}
