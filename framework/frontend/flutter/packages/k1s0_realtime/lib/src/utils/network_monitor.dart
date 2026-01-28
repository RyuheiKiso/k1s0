import 'dart:async';

import 'package:connectivity_plus/connectivity_plus.dart';

/// ネットワーク状態監視クラス
class NetworkMonitor {
  final Connectivity _connectivity;
  final StreamController<bool> _controller = StreamController<bool>.broadcast();
  StreamSubscription<List<ConnectivityResult>>? _subscription;
  bool _isOnline = true;

  NetworkMonitor({Connectivity? connectivity})
      : _connectivity = connectivity ?? Connectivity();

  /// 現在のオンライン状態
  bool get isOnline => _isOnline;

  /// オンライン状態のストリーム
  Stream<bool> get onlineStream => _controller.stream;

  /// 監視を開始する
  Future<void> start() async {
    final results = await _connectivity.checkConnectivity();
    _isOnline = !results.contains(ConnectivityResult.none);

    _subscription = _connectivity.onConnectivityChanged.listen((results) {
      final online = !results.contains(ConnectivityResult.none);
      if (online != _isOnline) {
        _isOnline = online;
        _controller.add(online);
      }
    });
  }

  /// 監視を停止する
  Future<void> stop() async {
    await _subscription?.cancel();
    _subscription = null;
  }

  /// リソースを解放する
  Future<void> dispose() async {
    await stop();
    await _controller.close();
  }
}
