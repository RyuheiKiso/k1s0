import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/activity.dart';
import '../providers/activity_provider.dart';

/// アクティビティ詳細画面
/// 個別アクティビティの詳細情報表示と承認フロー操作を行う画面
class ActivityDetailScreen extends ConsumerStatefulWidget {
  /// 対象アクティビティのID
  final String activityId;

  const ActivityDetailScreen({super.key, required this.activityId});

  @override
  ConsumerState<ActivityDetailScreen> createState() =>
      _ActivityDetailScreenState();
}

class _ActivityDetailScreenState extends ConsumerState<ActivityDetailScreen> {
  /// 却下理由入力用コントローラー
  final _rejectReasonController = TextEditingController();

  @override
  void dispose() {
    _rejectReasonController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    /// アクティビティ一覧の状態を監視し、対象アクティビティを抽出する
    final activitiesAsync = ref.watch(activityListProvider);

    return Scaffold(
      appBar: AppBar(
        title: Text('アクティビティ詳細: ${widget.activityId}'),
        actions: [
          /// 一覧を再読み込みするボタン
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: '更新',
            onPressed: () => ref.read(activityListProvider.notifier).load(),
          ),
        ],
      ),
      body: activitiesAsync.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (error, stack) => Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('エラーが発生しました: $error'),
              const SizedBox(height: 16),
              ElevatedButton(
                onPressed: () =>
                    ref.read(activityListProvider.notifier).load(),
                child: const Text('再試行'),
              ),
            ],
          ),
        ),
        data: (activities) {
          /// アクティビティ一覧から対象IDのアクティビティを検索する
          final activity =
              activities.where((a) => a.id == widget.activityId).firstOrNull;
          if (activity == null) {
            return const Center(child: Text('アクティビティが見つかりません'));
          }
          return _buildActivityDetail(context, activity);
        },
      ),
    );
  }

  /// アクティビティ詳細の全体レイアウトを構築する
  Widget _buildActivityDetail(BuildContext context, Activity activity) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          /// アクティビティ基本情報カード
          _buildActivityInfoCard(context, activity),
          const SizedBox(height: 16),

          /// 承認フロー操作カード
          _buildApprovalFlowCard(context, activity),
        ],
      ),
    );
  }

  /// アクティビティ基本情報を表示するカードを構築する
  Widget _buildActivityInfoCard(BuildContext context, Activity activity) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'アクティビティ情報',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            _buildInfoRow('ID', activity.id),
            _buildInfoRow('タスクID', activity.taskId),
            _buildInfoRow('アクターID', activity.actorId),
            _buildInfoRow('種別', activity.activityType.displayName),
            _buildInfoRow('ステータス', activity.status.displayName),
            if (activity.content != null && activity.content!.isNotEmpty)
              _buildInfoRow('内容', activity.content!),
            if (activity.durationMinutes != null)
              _buildInfoRow('作業時間', '${activity.durationMinutes}分'),
            _buildInfoRow('バージョン', activity.version.toString()),
            _buildInfoRow('作成日時', _formatDateTime(activity.createdAt)),
            _buildInfoRow('更新日時', _formatDateTime(activity.updatedAt)),
          ],
        ),
      ),
    );
  }

  /// 承認フロー操作を行うカードを構築する
  Widget _buildApprovalFlowCard(BuildContext context, Activity activity) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              '承認フロー',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 16),

            /// active の場合は承認申請ボタンを表示する
            if (activity.status == ActivityStatus.active)
              FilledButton.icon(
                onPressed: () => _submitActivity(activity),
                icon: const Icon(Icons.send),
                label: const Text('承認申請'),
              ),

            /// submitted の場合は承認・却下ボタンを表示する
            if (activity.status == ActivityStatus.submitted) ...[
              Row(
                children: [
                  FilledButton.icon(
                    onPressed: () => _approveActivity(activity),
                    icon: const Icon(Icons.check),
                    label: const Text('承認'),
                  ),
                  const SizedBox(width: 8),
                ],
              ),
              const SizedBox(height: 12),
              /// 却下フォーム: 理由入力と却下ボタン
              Row(
                children: [
                  Expanded(
                    child: TextField(
                      controller: _rejectReasonController,
                      decoration: const InputDecoration(
                        labelText: '却下理由（任意）',
                        border: OutlineInputBorder(),
                        hintText: '却下理由を入力してください',
                      ),
                    ),
                  ),
                  const SizedBox(width: 8),
                  OutlinedButton.icon(
                    onPressed: () => _rejectActivity(activity),
                    icon: const Icon(Icons.close, color: Colors.red),
                    label: const Text(
                      '却下',
                      style: TextStyle(color: Colors.red),
                    ),
                  ),
                ],
              ),
            ],

            /// approved・rejected の場合はフロー完了メッセージを表示する
            if (activity.status == ActivityStatus.approved ||
                activity.status == ActivityStatus.rejected)
              Text(
                activity.status == ActivityStatus.approved
                    ? '承認済みです'
                    : '却下済みです',
                style: TextStyle(
                  color: activity.status == ActivityStatus.approved
                      ? Colors.teal
                      : Colors.red,
                  fontWeight: FontWeight.bold,
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
            width: 120,
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

  /// アクティビティを承認申請する
  void _submitActivity(Activity activity) {
    ref.read(activityListProvider.notifier).submit(activity.id);
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('承認申請しました')),
    );
  }

  /// アクティビティを承認する
  void _approveActivity(Activity activity) {
    ref.read(activityListProvider.notifier).approve(activity.id);
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('承認しました')),
    );
  }

  /// アクティビティを却下する
  void _rejectActivity(Activity activity) {
    final reason = _rejectReasonController.text.trim();
    final input = RejectActivityInput(reason: reason.isNotEmpty ? reason : null);
    ref.read(activityListProvider.notifier).reject(activity.id, input);
    _rejectReasonController.clear();
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('却下しました')),
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
