import 'dart:async';

import 'package:http/http.dart' as http;
import 'package:meta/meta.dart';

typedef TokenProvider = Future<String> Function();
typedef CurrentVersionProvider = Future<String> Function();
typedef UrlLauncherCallback = Future<bool> Function(Uri uri);

@immutable
class AppUpdaterConfig {
  final String serverUrl;
  final String appId;
  final String? platform;
  final String? arch;
  final Duration? checkInterval;
  final String? iosStoreUrl;
  final String? androidStoreUrl;
  final Duration timeout;
  final TokenProvider? tokenProvider;
  final CurrentVersionProvider? currentVersionProvider;
  final http.Client? httpClient;
  final UrlLauncherCallback? urlLauncher;

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
