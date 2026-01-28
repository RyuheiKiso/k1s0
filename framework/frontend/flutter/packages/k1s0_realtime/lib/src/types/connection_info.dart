import 'package:meta/meta.dart';
import 'connection_status.dart';

/// 接続情報
@immutable
class ConnectionInfo {
  /// 接続 ID
  final String id;

  /// 接続状態
  final ConnectionStatus status;

  /// 再接続試行回数
  final int reconnectAttempt;

  /// 接続日時
  final DateTime? connectedAt;

  /// 切断日時
  final DateTime? disconnectedAt;

  const ConnectionInfo({
    required this.id,
    required this.status,
    this.reconnectAttempt = 0,
    this.connectedAt,
    this.disconnectedAt,
  });

  ConnectionInfo copyWith({
    String? id,
    ConnectionStatus? status,
    int? reconnectAttempt,
    DateTime? connectedAt,
    DateTime? disconnectedAt,
  }) {
    return ConnectionInfo(
      id: id ?? this.id,
      status: status ?? this.status,
      reconnectAttempt: reconnectAttempt ?? this.reconnectAttempt,
      connectedAt: connectedAt ?? this.connectedAt,
      disconnectedAt: disconnectedAt ?? this.disconnectedAt,
    );
  }
}
