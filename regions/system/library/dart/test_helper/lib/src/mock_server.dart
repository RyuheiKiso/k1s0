/// モックルート定義。
class MockRoute {
  final String method;
  final String path;
  final int status;
  final String body;

  const MockRoute({
    required this.method,
    required this.path,
    required this.status,
    required this.body,
  });
}

/// モックサーバー (インメモリ)。
class MockServer {
  final List<MockRoute> _routes;
  final List<({String method, String path})> _requests = [];

  MockServer(this._routes);

  /// 登録済みルートからレスポンスを取得する。
  ({int status, String body})? handle(String method, String path) {
    _requests.add((method: method, path: path));
    for (final route in _routes) {
      if (route.method == method && route.path == path) {
        return (status: route.status, body: route.body);
      }
    }
    return null;
  }

  /// 記録されたリクエスト数を返す。
  int get requestCount => _requests.length;

  /// 記録されたリクエストを返す。
  List<({String method, String path})> get recordedRequests =>
      List.unmodifiable(_requests);
}

/// モックサーバービルダー。
class MockServerBuilder {
  final String _serverType;
  final List<MockRoute> _routes = [];

  MockServerBuilder._(this._serverType);

  /// Notification サーバーモックを構築する。
  static MockServerBuilder notificationServer() =>
      MockServerBuilder._('notification');

  /// Ratelimit サーバーモックを構築する。
  static MockServerBuilder ratelimitServer() =>
      MockServerBuilder._('ratelimit');

  /// Tenant サーバーモックを構築する。
  static MockServerBuilder tenantServer() => MockServerBuilder._('tenant');

  /// サーバータイプを返す。
  String get serverType => _serverType;

  /// ヘルスチェック用の成功レスポンスを追加する。
  MockServerBuilder withHealthOk() {
    _routes.add(const MockRoute(
      method: 'GET',
      path: '/health',
      status: 200,
      body: '{"status":"ok"}',
    ));
    return this;
  }

  /// 成功レスポンスルートを追加する。
  MockServerBuilder withSuccessResponse(String path, String body) {
    _routes.add(MockRoute(
      method: 'POST',
      path: path,
      status: 200,
      body: body,
    ));
    return this;
  }

  /// エラーレスポンスルートを追加する。
  MockServerBuilder withErrorResponse(String path, int status) {
    _routes.add(MockRoute(
      method: 'POST',
      path: path,
      status: status,
      body: '{"error":"mock error"}',
    ));
    return this;
  }

  /// モックサーバーを構築する。
  MockServer build() => MockServer(_routes);
}
