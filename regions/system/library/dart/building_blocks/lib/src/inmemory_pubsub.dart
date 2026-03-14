import 'dart:async';
import 'dart:typed_data';

import 'component.dart';
import 'pubsub.dart';

/// サブスクリプション ID 生成用のグローバルカウンター。
/// 同一プロセス内で一意な ID を生成するために使用する。
int _pubsubIdCounter = 0;

/// タイムスタンプとカウンターを組み合わせたユニーク ID を生成する。
/// マイクロ秒精度のタイムスタンプにより衝突を最小化する。
String _generateId() => '${DateTime.now().microsecondsSinceEpoch}-${++_pubsubIdCounter}';

/// インメモリ実装の PubSub（発行/購読）コンポーネント。
/// メッセージをメモリ上のサブスクリプションマップで管理し、
/// publish 時に同一トピックの全ハンドラーへ同期的に配信する。
/// テスト・開発環境での利用を想定している。
class InMemoryPubSub implements PubSub {
  /// コンポーネントの識別名。
  @override
  final String name;

  /// コンポーネント種別を示す文字列。
  @override
  final String componentType = 'pubsub';

  /// コンポーネントの現在の状態（未初期化 / 準備完了 / クローズ済み）。
  ComponentStatus _status = ComponentStatus.uninitialized;

  /// トピック名をキー、サブスクリプション ID とハンドラーのマップを値とする購読管理テーブル。
  /// 構造: { topic: { subscriptionId: MessageHandler } }
  final _subs = <String, Map<String, MessageHandler>>{};

  /// コンストラクタ。name のデフォルト値は 'inmemory-pubsub'。
  InMemoryPubSub({this.name = 'inmemory-pubsub'});

  /// コンポーネントを初期化し、状態を ready に移行する。
  @override
  Future<void> init() async { _status = ComponentStatus.ready; }

  /// コンポーネントをクローズし、全サブスクリプションを削除して状態を closed に移行する。
  @override
  Future<void> close() async { _subs.clear(); _status = ComponentStatus.closed; }

  /// コンポーネントの現在の状態を返す。
  @override
  Future<ComponentStatus> status() async => _status;

  /// バックエンドが memory であることを示すメタデータを返す。
  @override
  Map<String, String> metadata() => {'backend': 'memory'};

  /// 指定トピックにデータを発行する。
  /// そのトピックに登録された全ハンドラーを並列に呼び出す。
  /// 購読者が存在しない場合は何もしない。
  @override
  Future<void> publish(String topic, Uint8List data, {Map<String, String>? metadata}) async {
    final handlers = _subs[topic];
    // 購読者がいない場合は処理をスキップする。
    if (handlers == null) return;
    // メッセージオブジェクトを生成してユニーク ID を付与する。
    final msg = Message(
      topic: topic,
      data: data,
      metadata: metadata ?? {},
      id: _generateId(),
    );
    // 全ハンドラーを並列実行し、全完了を待機する。
    await Future.wait(handlers.values.map((h) => h.handle(msg)));
  }

  /// 指定トピックへのサブスクリプションを登録し、サブスクリプション ID を返す。
  /// 返された ID は unsubscribe 時に使用する。
  @override
  Future<String> subscribe(String topic, MessageHandler handler) async {
    // トピックのエントリが存在しない場合は新規作成する。
    _subs.putIfAbsent(topic, () => {});
    final id = _generateId();
    _subs[topic]![id] = handler;
    return id;
  }

  /// 指定サブスクリプション ID の購読を解除する。
  /// 全トピックを走査して該当する ID のエントリを削除する。
  @override
  Future<void> unsubscribe(String subscriptionId) async {
    for (final handlers in _subs.values) {
      handlers.remove(subscriptionId);
    }
  }
}
