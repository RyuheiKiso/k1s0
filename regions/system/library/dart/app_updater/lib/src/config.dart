import 'dart:async';

import 'package:http/http.dart' as http;
import 'package:meta/meta.dart';

/// アクセストークンを非同期で取得する関数の型。
/// 認証が必要なAPIリクエスト時に呼び出される。
typedef TokenProvider = Future<String> Function();

/// 現在のアプリバージョンを非同期で取得する関数の型。
/// デフォルトでは `package_info_plus` を使用するが、テスト時にモックと差し替えられる。
typedef CurrentVersionProvider = Future<String> Function();

/// URLを開く関数の型。
/// デフォルトでは `url_launcher` を使用するが、テスト時にモックと差し替えられる。
typedef UrlLauncherCallback = Future<bool> Function(Uri uri);

/// アプリアップデーターの設定。
/// すべての依存関係と動作パラメーターをここで一元管理する。
@immutable
class AppUpdaterConfig {
  /// レジストリサーバーのURL。
  /// バージョン情報取得のベースURLとして使用される。
  final String serverUrl;

  /// アプリID。
  /// レジストリサーバー上でアプリを一意に識別するための識別子。
  final String appId;

  /// 対象プラットフォーム。省略時は自動検出。
  /// 'ios', 'android', 'windows', 'macos', 'linux' などの文字列を指定する。
  final String? platform;

  /// 対象アーキテクチャ。省略時は自動検出。
  /// 'amd64', 'arm64' などの文字列を指定する。
  final String? arch;

  /// 定期チェックの間隔。`null` の場合は定期チェックを行わない。
  /// startPeriodicCheck で使用されるタイマーの周期を決定する。
  final Duration? checkInterval;

  /// iOS App StoreのURL。
  /// プラットフォームが 'ios' の場合に getStoreUrl / openStore で使用される。
  final String? iosStoreUrl;

  /// Google PlayストアのURL。
  /// プラットフォームが 'android' の場合に getStoreUrl / openStore で使用される。
  final String? androidStoreUrl;

  /// HTTPリクエストのタイムアウト。
  /// デフォルトは10秒。タイムアウト超過時は ConnectionError がスローされる。
  final Duration timeout;

  /// アクセストークンを取得するプロバイダー。
  /// 設定した場合、各APIリクエストの Authorization ヘッダーに Bearer トークンが付与される。
  final TokenProvider? tokenProvider;

  /// 現在のアプリバージョンを取得するプロバイダー。省略時は `package_info_plus` を使用。
  final CurrentVersionProvider? currentVersionProvider;

  /// HTTPクライアント。省略時はデフォルトのクライアントを使用。
  /// テスト時にモッククライアントを注入するために使用する。
  final http.Client? httpClient;

  /// URLを開く関数。省略時は `url_launcher` を使用。
  /// テスト時にモック関数を注入するために使用する。
  final UrlLauncherCallback? urlLauncher;

  /// [AppUpdaterConfig] を生成する。
  /// [serverUrl] と [appId] は必須。その他はすべて省略可能。
  const AppUpdaterConfig({
    required this.serverUrl,
    required this.appId,
    this.platform,
    this.arch,
    this.checkInterval,
    this.iosStoreUrl,
    this.androidStoreUrl,
    this.timeout = const Duration(seconds: 10),
    this.tokenProvider,
    this.currentVersionProvider,
    this.httpClient,
    this.urlLauncher,
  });
}
