import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../models/board_column.dart';
import '../providers/board_provider.dart';

/// WIP制限編集画面
/// 指定カラムのWIP制限を編集するフォームを表示する
class WipLimitScreen extends ConsumerStatefulWidget {
  /// 編集対象のプロジェクトID
  final String projectId;
  /// 編集対象のステータスコード
  final String statusCode;

  const WipLimitScreen({
    super.key,
    required this.projectId,
    required this.statusCode,
  });

  @override
  ConsumerState<WipLimitScreen> createState() => _WipLimitScreenState();
}

class _WipLimitScreenState extends ConsumerState<WipLimitScreen> {
  /// フォームバリデーション用のキー
  final _formKey = GlobalKey<FormState>();

  /// WIP制限入力用コントローラー
  late TextEditingController _wipLimitController;

  /// 現在のカラムデータ（プロバイダーから取得した初期値設定に使用）
  BoardColumn? _currentColumn;

  @override
  void initState() {
    super.initState();
    _wipLimitController = TextEditingController();
  }

  @override
  void dispose() {
    _wipLimitController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    /// ボードカラム一覧の状態を監視する
    /// 非ファミリープロバイダーのため引数なしで参照する
    final columnsAsync = ref.watch(boardColumnListProvider);

    return Scaffold(
      appBar: AppBar(
        title: Text('WIP制限編集: ${widget.statusCode}'),
      ),
      body: columnsAsync.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (error, stack) => Center(
          child: Text('エラーが発生しました: $error'),
        ),
        data: (columns) {
          /// カラム一覧から対象カラムを検索する
          final column = columns
              .where((c) => c.statusCode == widget.statusCode)
              .firstOrNull;

          if (column == null) {
            return const Center(child: Text('カラムが見つかりません'));
          }

          /// 初回ロード時にのみコントローラーの初期値を設定する
          if (_currentColumn == null) {
            _currentColumn = column;
            _wipLimitController.text = column.wipLimit.toString();
          }

          return _buildForm(context, column);
        },
      ),
    );
  }

  /// WIP制限編集フォームを構築する
  Widget _buildForm(BuildContext context, BoardColumn column) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Form(
        key: _formKey,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            /// 現在のカラム情報を表示するカード
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'カラム情報',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    const SizedBox(height: 8),
                    _buildInfoRow('プロジェクトID', column.projectId),
                    _buildInfoRow('ステータスコード', column.statusCode),
                    _buildInfoRow('現在のタスク数', '${column.taskCount}'),
                    _buildInfoRow(
                      '現在のWIP制限',
                      column.wipLimit > 0 ? '${column.wipLimit}' : '無制限',
                    ),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),

            /// WIP制限入力カード
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'WIP制限を設定',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    const SizedBox(height: 8),
                    /// 補足説明テキスト
                    Text(
                      '0 を設定すると無制限になります。',
                      style: TextStyle(
                        fontSize: 12,
                        color: Colors.grey.shade600,
                      ),
                    ),
                    const SizedBox(height: 16),
                    /// WIP制限入力フィールド
                    TextFormField(
                      controller: _wipLimitController,
                      decoration: const InputDecoration(
                        labelText: 'WIP制限',
                        hintText: '0 = 無制限',
                        border: OutlineInputBorder(),
                      ),
                      keyboardType: TextInputType.number,
                      validator: (value) {
                        if (value == null || value.trim().isEmpty) {
                          return 'WIP制限は必須です';
                        }
                        final limit = int.tryParse(value.trim());
                        if (limit == null || limit < 0) {
                          return '0以上の整数を入力してください';
                        }
                        return null;
                      },
                    ),
                    const SizedBox(height: 16),
                    /// 更新ボタンと戻るボタン
                    Row(
                      mainAxisAlignment: MainAxisAlignment.end,
                      children: [
                        TextButton(
                          onPressed: () => context.pop(),
                          child: const Text('キャンセル'),
                        ),
                        const SizedBox(width: 8),
                        FilledButton(
                          onPressed: () => _updateWipLimit(context, column),
                          child: const Text('更新'),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  /// 情報表示用の行ウィジェットを構築する
  Widget _buildInfoRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 140,
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

  /// WIP制限を更新してボード画面に戻る
  void _updateWipLimit(BuildContext context, BoardColumn column) {
    if (!_formKey.currentState!.validate()) return;

    final limit = int.parse(_wipLimitController.text.trim());
    final input = UpdateWipLimitInput(wipLimit: limit);

    ref
        .read(boardColumnListProvider.notifier)
        .updateWipLimit(widget.projectId, column.statusCode, input);

    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(
          'WIP制限を ${limit > 0 ? limit.toString() : '無制限'} に更新しました',
        ),
      ),
    );

    context.pop();
  }
}
