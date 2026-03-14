import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../models/payment.dart';
import '../providers/payment_provider.dart';

/// 決済一覧画面
/// 決済の検索・フィルタリング・一覧表示を行うメイン画面
class PaymentListScreen extends ConsumerStatefulWidget {
  const PaymentListScreen({super.key});

  @override
  ConsumerState<PaymentListScreen> createState() => _PaymentListScreenState();
}

class _PaymentListScreenState extends ConsumerState<PaymentListScreen> {
  /// 現在選択中のステータスフィルタ
  PaymentStatus? _selectedStatus;

  @override
  Widget build(BuildContext context) {
    /// 決済一覧の状態を監視する
    final paymentsAsync = ref.watch(paymentListProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('決済管理'),
        actions: [
          /// 一覧を再読み込みするボタン
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: '更新',
            onPressed: () =>
                ref.read(paymentListProvider.notifier).load(),
          ),
        ],
      ),
      body: Column(
        children: [
          /// ステータスフィルタチップセクション
          _buildFilterSection(),
          /// 決済一覧テーブル
          Expanded(
            child: paymentsAsync.when(
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
                      onPressed: () =>
                          ref.read(paymentListProvider.notifier).load(),
                      child: const Text('再試行'),
                    ),
                  ],
                ),
              ),
              /// データ取得成功時は決済一覧をテーブル表示する
              data: (payments) {
                if (payments.isEmpty) {
                  return const Center(child: Text('決済がありません'));
                }
                return _buildPaymentTable(payments);
              },
            ),
          ),
        ],
      ),
      /// 決済作成用のFAB
      floatingActionButton: FloatingActionButton(
        onPressed: () => context.push('/payments/new'),
        tooltip: '新規決済',
        child: const Icon(Icons.add),
      ),
    );
  }

  /// ステータスフィルタセクションを構築する
  /// 各ステータスのFilterChipを横並びで表示する
  Widget _buildFilterSection() {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: Row(
          children: [
            /// 「すべて」フィルタチップ
            FilterChip(
              label: const Text('すべて'),
              selected: _selectedStatus == null,
              onSelected: (selected) {
                setState(() => _selectedStatus = null);
                ref.read(paymentListProvider.notifier).load();
              },
            ),
            const SizedBox(width: 8),
            /// 各ステータスのフィルタチップを動的生成する
            ...PaymentStatus.values.map((status) => Padding(
                  padding: const EdgeInsets.only(right: 8),
                  child: FilterChip(
                    label: Text(status.label),
                    selected: _selectedStatus == status,
                    onSelected: (selected) {
                      setState(() =>
                          _selectedStatus = selected ? status : null);
                      ref.read(paymentListProvider.notifier).load(
                            status: selected ? status : null,
                          );
                    },
                  ),
                )),
          ],
        ),
      ),
    );
  }

  /// 決済一覧テーブルを構築する
  /// DataTableを使用して決済情報を表形式で表示する
  Widget _buildPaymentTable(List<Payment> payments) {
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: SingleChildScrollView(
        child: DataTable(
          /// テーブルのカラムヘッダーを定義する
          columns: const [
            DataColumn(label: Text('決済ID')),
            DataColumn(label: Text('注文ID')),
            DataColumn(label: Text('金額')),
            DataColumn(label: Text('ステータス')),
            DataColumn(label: Text('決済方法')),
            DataColumn(label: Text('作成日')),
          ],
          /// 決済データの各行を構築する
          rows: payments.map((payment) {
            return DataRow(
              cells: [
                /// 決済ID（タップで詳細画面へ遷移）
                DataCell(
                  Text(
                    payment.id.length > 8
                        ? '${payment.id.substring(0, 8)}...'
                        : payment.id,
                  ),
                  onTap: () => context.push('/payments/${payment.id}'),
                ),
                /// 注文ID
                DataCell(Text(payment.orderId)),
                /// 金額と通貨
                DataCell(Text(
                  '${payment.amount.toStringAsFixed(0)} ${payment.currency}',
                )),
                /// ステータスバッジ
                DataCell(_buildStatusBadge(payment.status)),
                /// 決済方法ラベル
                DataCell(Text(payment.paymentMethod.label)),
                /// 作成日時
                DataCell(Text(_formatDateTime(payment.createdAt))),
              ],
            );
          }).toList(),
        ),
      ),
    );
  }

  /// ステータスバッジウィジェットを構築する
  /// ステータスに応じた色のChipを表示する
  Widget _buildStatusBadge(PaymentStatus status) {
    /// ステータスに応じた色を決定する
    final color = switch (status) {
      PaymentStatus.pending => Colors.orange,
      PaymentStatus.processing => Colors.blue,
      PaymentStatus.completed => Colors.green,
      PaymentStatus.failed => Colors.red,
      PaymentStatus.refunded => Colors.purple,
    };

    return Chip(
      label: Text(
        status.label,
        style: const TextStyle(color: Colors.white, fontSize: 12),
      ),
      backgroundColor: color,
      padding: EdgeInsets.zero,
      materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
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
