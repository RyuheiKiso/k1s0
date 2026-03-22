import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../models/activity.dart';
import '../providers/activity_provider.dart';

/// アクティビティ一覧画面
/// アクティビティの検索・フィルタリング・タイムライン表示を行うメイン画面
class ActivityListScreen extends ConsumerStatefulWidget {
  const ActivityListScreen({super.key});

  @override
  ConsumerState<ActivityListScreen> createState() => _ActivityListScreenState();
}

class _ActivityListScreenState extends ConsumerState<ActivityListScreen> {
  /// 現在選択中のステータスフィルタ（nullの場合は全件表示）
  ActivityStatus? _selectedStatus;

  /// 現在選択中の種別フィルタ（nullの場合は全件表示）
  ActivityType? _selectedType;

  @override
  Widget build(BuildContext context) {
    /// アクティビティ一覧の状態を監視する
    final activitiesAsync = ref.watch(activityListProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('アクティビティ一覧'),
        actions: [
          /// 新規アクティビティ作成画面への遷移ボタン
          IconButton(
            icon: const Icon(Icons.add),
            tooltip: '新規アクティビティ',
            onPressed: () => context.push('/activities/new'),
          ),
          /// 一覧を再読み込みするボタン
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: '更新',
            onPressed: () => ref.read(activityListProvider.notifier).load(
                  activityType: _selectedType,
                ),
          ),
        ],
      ),
      body: Column(
        children: [
          /// フィルタチップを表示する
          _buildFilterChips(),
          /// アクティビティ一覧を表示する
          Expanded(
            child: activitiesAsync.when(
              /// ローディング中はプログレスインジケーターを表示する
              loading: () =>
                  const Center(child: CircularProgressIndicator()),
              /// エラー時はエラーメッセージとリトライボタンを表示する
              error: (error, stack) => Center(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text('エラーが発生しました: $error'),
                    const SizedBox(height: 16),
                    ElevatedButton(
                      onPressed: () =>
                          ref.read(activityListProvider.notifier).load(
                                activityType: _selectedType,
                              ),
                      child: const Text('再試行'),
                    ),
                  ],
                ),
              ),
              /// データ取得成功時はアクティビティをタイムライン形式で表示する
              data: (activities) {
                /// ステータスフィルタをクライアントサイドで適用する
                final filtered = _selectedStatus == null
                    ? activities
                    : activities
                        .where((a) => a.status == _selectedStatus)
                        .toList();

                if (filtered.isEmpty) {
                  return const Center(child: Text('アクティビティがありません'));
                }
                return ListView.builder(
                  itemCount: filtered.length,
                  itemBuilder: (context, index) {
                    final activity = filtered[index];
                    /// key にアクティビティIDを指定してリスト再構築時の差分検出を最適化する
                    return _ActivityTimelineTile(
                      key: ValueKey(activity.id),
                      activity: activity,
                    );
                  },
                );
              },
            ),
          ),
        ],
      ),
    );
  }

  /// フィルタチップ一覧を構築する
  Widget _buildFilterChips() {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: Row(
          children: [
            /// ステータスフィルタセクション
            const Text('ステータス: '),
            FilterChip(
              label: const Text('すべて'),
              selected: _selectedStatus == null,
              onSelected: (selected) {
                setState(() => _selectedStatus = null);
              },
            ),
            const SizedBox(width: 4),
            ...ActivityStatus.values.map((status) {
              return Padding(
                padding: const EdgeInsets.only(right: 4),
                child: FilterChip(
                  label: Text(status.displayName),
                  selected: _selectedStatus == status,
                  onSelected: (selected) {
                    setState(() {
                      _selectedStatus = selected ? status : null;
                    });
                  },
                ),
              );
            }),
            const SizedBox(width: 16),
            /// 種別フィルタセクション
            const Text('種別: '),
            FilterChip(
              label: const Text('すべて'),
              selected: _selectedType == null,
              onSelected: (selected) {
                setState(() => _selectedType = null);
                ref.read(activityListProvider.notifier).load();
              },
            ),
            const SizedBox(width: 4),
            ...ActivityType.values.map((type) {
              return Padding(
                padding: const EdgeInsets.only(right: 4),
                child: FilterChip(
                  label: Text(type.displayName),
                  selected: _selectedType == type,
                  onSelected: (selected) {
                    setState(() {
                      _selectedType = selected ? type : null;
                    });
                    ref.read(activityListProvider.notifier).load(
                          activityType: selected ? type : null,
                        );
                  },
                ),
              );
            }),
          ],
        ),
      ),
    );
  }
}

/// アクティビティタイムラインのタイルウィジェット
/// アクティビティ情報の概要表示とタップによる詳細遷移を提供する
class _ActivityTimelineTile extends StatelessWidget {
  final Activity activity;

  const _ActivityTimelineTile({super.key, required this.activity});

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
      child: ListTile(
        /// 種別に応じたアイコンをリードとして表示する
        leading: CircleAvatar(
          backgroundColor: _getTypeColor(activity.activityType),
          child: Icon(
            _getTypeIcon(activity.activityType),
            color: Colors.white,
            size: 18,
          ),
        ),
        /// アクターIDと種別をタイトルに表示する
        title: Text('${activity.actorId} — ${activity.activityType.displayName}'),
        /// 内容・作業時間・作成日をサブタイトルに表示する
        subtitle: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (activity.content != null) Text(activity.content!),
            if (activity.durationMinutes != null)
              Text('作業時間: ${activity.durationMinutes}分'),
            Text(_formatDateTime(activity.createdAt)),
          ],
        ),
        isThreeLine: true,
        /// ステータスバッジを右側に表示する
        trailing: _StatusBadge(status: activity.status),
        /// タップでアクティビティ詳細画面へ遷移する
        onTap: () => context.push('/activities/${activity.id}'),
      ),
    );
  }

  /// 種別に応じたアイコンを返す
  IconData _getTypeIcon(ActivityType type) {
    return switch (type) {
      ActivityType.comment => Icons.comment,
      ActivityType.time_entry => Icons.access_time,
      ActivityType.status_change => Icons.swap_horiz,
      ActivityType.assignment => Icons.person_add,
    };
  }

  /// 種別に応じたアイコン背景色を返す
  Color _getTypeColor(ActivityType type) {
    return switch (type) {
      ActivityType.comment => Colors.blue,
      ActivityType.time_entry => Colors.green,
      ActivityType.status_change => Colors.orange,
      ActivityType.assignment => Colors.purple,
    };
  }

  /// DateTimeを読みやすい日本語形式にフォーマットする
  String _formatDateTime(DateTime dateTime) {
    return '${dateTime.year}/${dateTime.month.toString().padLeft(2, '0')}/'
        '${dateTime.day.toString().padLeft(2, '0')} '
        '${dateTime.hour.toString().padLeft(2, '0')}:'
        '${dateTime.minute.toString().padLeft(2, '0')}';
  }
}

/// ステータスバッジウィジェット
/// アクティビティステータスに応じた色分けバッジを表示する
class _StatusBadge extends StatelessWidget {
  final ActivityStatus status;

  const _StatusBadge({required this.status});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
      decoration: BoxDecoration(
        color: _getStatusColor(status),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Text(
        status.displayName,
        style: const TextStyle(
          color: Colors.white,
          fontSize: 12,
          fontWeight: FontWeight.bold,
        ),
      ),
    );
  }

  /// ステータスに応じたバッジ背景色を返す
  Color _getStatusColor(ActivityStatus status) {
    return switch (status) {
      ActivityStatus.active => Colors.green,
      ActivityStatus.submitted => Colors.orange,
      ActivityStatus.approved => Colors.teal,
      ActivityStatus.rejected => Colors.red,
    };
  }
}
