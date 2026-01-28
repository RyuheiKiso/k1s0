# Flutter パッケージ一覧

```
framework/frontend/flutter/packages/
├── k1s0_navigation/     # 設定駆動ナビゲーション（NEW）
├── k1s0_config/         # YAML設定管理
├── k1s0_http/           # API通信クライアント
├── k1s0_auth/           # 認証クライアント
├── k1s0_observability/  # OTel/ログ
├── k1s0_ui/             # Design System
└── k1s0_state/          # 状態管理
```

## 実装状況

| パッケージ | 状態 | 説明 |
|-----------|:----:|------|
| k1s0_navigation | ✅ | 設定駆動ナビゲーション、go_router統合、ルートガード |
| k1s0_config | ✅ | YAML設定管理、Zodスキーマバリデーション、環境マージ |
| k1s0_http | ✅ | Dioベース通信クライアント、OTel計測、ProblemDetails対応 |
| k1s0_auth | ✅ | JWT/OIDC認証、SecureStorage、トークン自動更新 |
| k1s0_observability | ✅ | 構造化ログ、分散トレース、メトリクス収集 |
| k1s0_ui | ✅ | Material 3 Design System、共通ウィジェット、テーマ |
| k1s0_state | ✅ | Riverpod状態管理、AsyncValueヘルパー、永続化 |

---

## k1s0_navigation (Flutter)

### 目的

go_router ベースの設定駆動型ナビゲーションを提供する。ルート設定、ルートガード、認証連携、Shell Routes を統合。

### 主要な型

#### RouteConfig / RouteEntry

```dart
/// ルート設定
class RouteConfig {
  final List<RouteEntry> routes;
  final String initialLocation;
  final bool debugLogDiagnostics;
  final bool routerNeglect;

  RouteConfig copyWith({...});
}

/// ルートエントリ
class RouteEntry {
  final String path;
  final String name;
  final Widget Function(BuildContext, GoRouterState) builder;
  final List<RouteEntry> children;
  final List<String> guards;

  /// GoRoute に変換
  GoRoute toGoRoute({required Map<String, RouteGuardCallback> guardCallbacks});
}

/// ルート設定ビルダー
class RouteConfigBuilder {
  RouteConfigBuilder addRoute(RouteEntry route);
  RouteConfigBuilder addRoutes(List<RouteEntry> routes);
  RouteConfigBuilder initialLocation(String location);
  RouteConfigBuilder enableDebugLogging();
  RouteConfig build();
}
```

#### Route Guards

```dart
/// ルートガード基底クラス
abstract class RouteGuard {
  final String name;

  /// ガードチェック（null を返すと通過、String を返すとリダイレクト）
  String? check(BuildContext context, GoRouterState state);
}

/// 関数ベースのルートガード
class FunctionalRouteGuard extends RouteGuard {
  FunctionalRouteGuard({
    required String name,
    required String? Function(BuildContext, GoRouterState) checkFn,
  });
}

/// 認証ガード
class AuthGuard extends RouteGuard {
  final bool Function(BuildContext) isAuthenticated;
  final String loginPath;
  final String? returnToParameter;
  final List<String> excludedPaths;

  AuthGuard({
    String name = 'auth',
    required this.isAuthenticated,
    this.loginPath = '/login',
    this.returnToParameter = 'returnTo',
    this.excludedPaths = const [],
  });
}

/// ロールベースガード
class RoleGuard extends RouteGuard {
  final List<String> Function(BuildContext) getRoles;
  final List<String> requiredRoles;
  final bool requireAll;
  final String fallbackPath;

  RoleGuard({
    String name = 'role',
    required this.getRoles,
    required this.requiredRoles,
    this.requireAll = false,
    this.fallbackPath = '/forbidden',
  });
}

/// 複合ガード（複数ガードを順番にチェック）
class CompositeRouteGuard extends RouteGuard {
  final List<RouteGuard> guards;

  CompositeRouteGuard({
    required String name,
    required this.guards,
  });
}

/// ガードレジストリ
class RouteGuardRegistry {
  void register(RouteGuard guard);
  void unregister(String name);
  RouteGuard? get(String name);
  bool contains(String name);
  void clear();
}
```

#### K1s0Router

```dart
/// k1s0 Router ラッパー
class K1s0Router {
  final RouteConfig config;
  final RouteGuardRegistry guardRegistry;
  final List<NavigatorObserver> observers;
  final Widget Function(BuildContext, GoRouterState)? errorBuilder;
  final Listenable? refreshListenable;

  K1s0Router({
    required this.config,
    RouteGuardRegistry? guardRegistry,
    this.observers = const [],
    this.errorBuilder,
    this.refreshListenable,
  });

  /// GoRouter インスタンスを作成
  GoRouter createRouter();
}
```

#### Shell Routes

```dart
/// Shell Route 設定
class K1s0ShellRoute {
  final String? name;
  final Widget Function(BuildContext, GoRouterState, Widget) builder;
  final List<RouteEntry> routes;
  final List<String> guards;
  final GlobalKey<NavigatorState>? navigatorKey;

  ShellRoute toShellRoute({required Map<String, RouteGuardCallback> guardCallbacks});
}

/// Stateful Shell Route（タブナビゲーション用）
class K1s0StatefulShellRoute {
  final List<K1s0ShellBranch> branches;
  final Widget Function(BuildContext, GoRouterState, StatefulNavigationShell) builder;
  final String? restorationScopeId;

  StatefulShellRoute toStatefulShellRoute({...});
}

class K1s0ShellBranch {
  final GlobalKey<NavigatorState>? navigatorKey;
  final String? restorationScopeId;
  final String? initialLocation;
  final List<RouteEntry> routes;
}
```

#### Riverpod Providers

```dart
/// ルート設定プロバイダー
final routeConfigProvider = Provider<RouteConfig>((ref) => throw UnimplementedError());

/// ガードレジストリプロバイダー
final routeGuardRegistryProvider = Provider<RouteGuardRegistry>((ref) {
  return RouteGuardRegistry();
});

/// GoRouter プロバイダー
final routerProvider = Provider<GoRouter>((ref) {
  final config = ref.watch(routeConfigProvider);
  final registry = ref.watch(routeGuardRegistryProvider);

  final k1s0Router = K1s0Router(
    config: config,
    guardRegistry: registry,
  );

  return k1s0Router.createRouter();
});

/// 認証状態通知
class SimpleAuthStateNotifier extends ChangeNotifier {
  bool _isAuthenticated;

  bool get isAuthenticated => _isAuthenticated;

  void login() { ... }
  void logout() { ... }
}

final authStateProvider = ChangeNotifierProvider<SimpleAuthStateNotifier>((ref) {
  return SimpleAuthStateNotifier();
});
```

### 使用例

```dart
// ルート設定
final config = RouteConfigBuilder()
    .addRoute(RouteEntry(
      path: '/',
      name: 'home',
      builder: (context, state) => const HomePage(),
    ))
    .addRoute(RouteEntry(
      path: '/login',
      name: 'login',
      builder: (context, state) => const LoginPage(),
    ))
    .addRoute(RouteEntry(
      path: '/dashboard',
      name: 'dashboard',
      builder: (context, state) => const DashboardPage(),
      guards: ['auth'],  // 認証ガードを適用
    ))
    .initialLocation('/')
    .enableDebugLogging()
    .build();

// ガード登録
final registry = RouteGuardRegistry();
registry.register(AuthGuard(
  isAuthenticated: (context) => ref.read(authStateProvider).isAuthenticated,
  loginPath: '/login',
  excludedPaths: ['/login', '/register'],
));

// Router 作成
final router = K1s0Router(
  config: config,
  guardRegistry: registry,
  refreshListenable: ref.read(authStateProvider),
).createRouter();

// MaterialApp で使用
MaterialApp.router(
  routerConfig: router,
)

// Shell Route（共通レイアウト）
final shellRoute = K1s0ShellRoute(
  builder: (context, state, child) => Scaffold(
    appBar: AppBar(title: Text('My App')),
    body: child,
  ),
  routes: [
    RouteEntry(path: '/home', name: 'home', builder: ...),
    RouteEntry(path: '/settings', name: 'settings', builder: ...),
  ],
);

// Stateful Shell Route（タブナビゲーション）
final statefulShell = K1s0StatefulShellRoute(
  branches: [
    K1s0ShellBranch(
      routes: [RouteEntry(path: '/home', name: 'home', builder: ...)],
    ),
    K1s0ShellBranch(
      routes: [RouteEntry(path: '/profile', name: 'profile', builder: ...)],
    ),
  ],
  builder: (context, state, shell) => Scaffold(
    body: shell,
    bottomNavigationBar: BottomNavigationBar(
      currentIndex: shell.currentIndex,
      onTap: shell.goBranch,
      items: [...],
    ),
  ),
);
```

---

## k1s0_config (Flutter)

### 目的

YAML 設定ファイルの読み込み、型付け、バリデーション、環境マージを提供する。

### 主要な型

```dart
@freezed
class AppConfig with _$AppConfig {
  const factory AppConfig({
    required ApiConfig api,
    required AuthConfig auth,
    required LoggingConfig logging,
    @Default({}) Map<String, bool> featureFlags,
  }) = _AppConfig;
}

class ConfigLoader {
  ConfigLoader({required String defaultPath, String? environment});
  Future<AppConfig> load();
}
```

### 使用例

```dart
final loader = ConfigLoader(
  defaultPath: 'assets/config/default.yaml',
  environment: 'production',
);
final config = await loader.load();

// Riverpod Provider経由でアクセス
ConfigScope(
  config: config,
  child: MyApp(),
)

// 子ウィジェットで使用
final config = ref.watch(configProvider);
```

---

## k1s0_http (Flutter)

### 目的

Dio ベースの HTTP クライアント。トレース伝播、エラーハンドリング、ProblemDetails 対応を提供。

### 主要な型

```dart
class K1s0HttpClient {
  K1s0HttpClient({required HttpClientConfig config});

  Future<K1s0Response<T>> get<T>(String path, {RequestOptions? options});
  Future<K1s0Response<T>> post<T>(String path, {dynamic data, RequestOptions? options});
  Future<K1s0Response<T>> put<T>(String path, {dynamic data, RequestOptions? options});
  Future<K1s0Response<T>> delete<T>(String path, {RequestOptions? options});
}

@freezed
class ProblemDetails with _$ProblemDetails {
  const factory ProblemDetails({
    required String type,
    required String title,
    required int status,
    String? detail,
    String? instance,
    String? errorCode,
    String? traceId,
  }) = _ProblemDetails;
}

class ApiError {
  final ApiErrorKind kind;
  final String message;
  final ProblemDetails? problemDetails;
}
```

### 使用例

```dart
final client = K1s0HttpClient(
  config: HttpClientConfig(
    baseUrl: 'https://api.example.com',
    timeout: Duration(seconds: 30),
  ),
);

try {
  final response = await client.get<User>('/users/123');
  print(response.data);
} on ApiError catch (e) {
  print('Error: ${e.message}');
  if (e.problemDetails != null) {
    print('Error Code: ${e.problemDetails!.errorCode}');
  }
}
```

---

## k1s0_auth (Flutter)

### 目的

JWT/OIDC 認証クライアント。トークン管理、認証状態管理、認証ガード、GoRouter 統合を提供。

### 主要な型

```dart
@freezed
class Claims with _$Claims {
  const factory Claims({
    required String sub,
    required String iss,
    String? aud,
    required int exp,
    required int iat,
    @Default([]) List<String> roles,
    @Default([]) List<String> permissions,
    String? tenantId,
  }) = _Claims;
}

@freezed
class AuthState with _$AuthState {
  const factory AuthState.initial() = AuthInitial;
  const factory AuthState.loading() = AuthLoading;
  const factory AuthState.authenticated(AuthUser user) = AuthAuthenticated;
  const factory AuthState.unauthenticated() = AuthUnauthenticated;
  const factory AuthState.error(AuthError error) = AuthError;
}

class AuthNotifier extends StateNotifier<AuthState> {
  Future<void> login(String accessToken, {String? refreshToken});
  Future<void> logout();
  Future<void> refreshTokens();
}
```

### 使用例

```dart
// AuthProvider で認証状態を管理
final authState = ref.watch(authProvider);

authState.when(
  initial: () => SplashScreen(),
  loading: () => LoadingScreen(),
  authenticated: (user) => HomePage(),
  unauthenticated: () => LoginPage(),
  error: (error) => ErrorPage(error),
);

// 認証ガード
AuthGuard(
  child: DashboardPage(),
  unauthenticatedBuilder: (context) => LoginPage(),
)

// ロールベースの認可
RequireRole(
  roles: ['admin'],
  child: AdminPanel(),
  fallback: AccessDenied(),
)

// GoRouter 統合
final router = GoRouter(
  redirect: authGuard(
    ref,
    redirectTo: '/login',
    allowedPaths: ['/login', '/register'],
  ),
  routes: [...],
);
```

---

## k1s0_observability (Flutter)

### 目的

フロントエンド向け観測性ライブラリ。構造化ログ、分散トレース、エラートラッキング、パフォーマンスメトリクスを提供。

### 必須フィールド（ログ）

バックエンド（k1s0-observability）と同じ必須フィールドをフロントエンドでも強制。

| フィールド | 説明 |
|-----------|------|
| `timestamp` | ISO 8601 形式のタイムスタンプ |
| `level` | ログレベル（debug/info/warn/error） |
| `message` | ログメッセージ |
| `service_name` | サービス名 |
| `env` | 環境名（dev/stg/prod） |
| `trace_id` | トレース ID（リクエスト相関用） |
| `span_id` | スパン ID |

### 主要な型

```dart
@freezed
class LogEntry with _$LogEntry {
  const factory LogEntry({
    required DateTime timestamp,
    required LogLevel level,
    required String message,
    required String serviceName,
    required String env,
    String? traceId,
    String? spanId,
    @Default({}) Map<String, dynamic> fields,
  }) = _LogEntry;
}

class Logger {
  void debug(String message, [Map<String, dynamic>? fields]);
  void info(String message, [Map<String, dynamic>? fields]);
  void warn(String message, [Map<String, dynamic>? fields]);
  void error(String message, [Object? error, StackTrace? stackTrace]);
}

class Tracer {
  Future<T> trace<T>(String name, Future<T> Function() fn);
  T traceSync<T>(String name, T Function() fn);
}
```

### 使用例

```dart
// Logger の使用
final logger = ref.read(loggerProvider);
logger.info('ユーザーがログインしました', {
  'userId': user.id,
  'loginMethod': 'oauth',
});

// Tracer の使用
final tracer = ref.read(tracerProvider);
final user = await tracer.trace('fetch-user-data', () async {
  return await api.getUser(userId);
});

// エラートラッキング
final errorTracker = ref.read(errorTrackerProvider);
try {
  await riskyOperation();
} catch (e, stackTrace) {
  errorTracker.capture(e, stackTrace);
}
```

---

## k1s0_ui (Flutter)

### 目的

k1s0 Design System を提供する。Material 3 ベースの統一されたテーマ、共通ウィジェット、フォームバリデーション、フィードバックコンポーネントを実現。

### モジュール構成

| モジュール | 内容 |
|-----------|------|
| `theme/` | K1s0Theme, K1s0Colors, K1s0Typography, K1s0Spacing, ThemeProvider |
| `widgets/` | K1s0PrimaryButton, K1s0SecondaryButton, K1s0Card, K1s0TextField |
| `form/` | K1s0Validators, K1s0FormContainer, K1s0FormSection, K1s0Form（スキーマ駆動） |
| `feedback/` | K1s0Snackbar, K1s0Dialog |
| `state/` | K1s0Loading, K1s0ErrorState, K1s0EmptyState |
| `data_table/` | K1s0DataTable、K1s0Column、ソート・ページネーション・選択機能 |

### 使用例

```dart
// テーマ設定
MaterialApp(
  theme: ref.watch(themeProvider).lightTheme,
  darkTheme: ref.watch(themeProvider).darkTheme,
  themeMode: ref.watch(themeProvider).themeMode,
)

// ボタン
K1s0PrimaryButton(
  onPressed: () {},
  loading: isSubmitting,
  child: Text('Submit'),
)

// テキストフィールド
K1s0TextField(
  controller: controller,
  label: 'Email',
  validator: K1s0Validators.combine([
    K1s0Validators.required,
    K1s0Validators.email,
  ]),
)

// フィードバック
K1s0Snackbar.success(context, 'Operation completed!');

final confirmed = await K1s0Dialog.confirm(
  context,
  title: 'Delete Item',
  message: 'Are you sure?',
  isDanger: true,
);

// 状態ウィジェット
K1s0Loading(message: 'Loading...')
K1s0ErrorState(message: 'Error occurred', onRetry: _retry)
K1s0EmptyState(title: 'No items', message: 'Add your first item')
```

### DataTable

高機能データテーブルコンポーネント。ソート、ページネーション、行選択、カスタムセルレンダリングをサポート。

#### 主要コンポーネント

| コンポーネント | 説明 |
|---------------|------|
| `K1s0DataTable<T>` | メインデータテーブルウィジェット |
| `K1s0Column<T>` | カラム定義 |
| `K1s0DataTableController` | 状態管理コントローラー |
| `K1s0SortModel` | ソート状態モデル |

#### カラムタイプ

| タイプ | 説明 |
|--------|------|
| `K1s0ColumnType.text` | テキスト表示（デフォルト） |
| `K1s0ColumnType.number` | 数値（右寄せ、カンマ区切り） |
| `K1s0ColumnType.date` | 日付フォーマット |
| `K1s0ColumnType.boolean` | チェックアイコン表示 |
| `K1s0ColumnType.chip` | Chip表示 |
| `K1s0ColumnType.actions` | アクションボタン |
| `K1s0ColumnType.custom` | カスタムレンダラー |

#### 使用例

```dart
K1s0DataTable<User>(
  rows: users,
  columns: [
    K1s0Column<User>(
      id: 'name',
      label: '氏名',
      sortable: true,
      valueGetter: (user) => user.name,
    ),
    K1s0Column<User>(
      id: 'role',
      label: '権限',
      type: K1s0ColumnType.chip,
      valueGetter: (user) => user.role,
    ),
    K1s0Column<User>(
      id: 'createdAt',
      label: '作成日',
      type: K1s0ColumnType.date,
      valueGetter: (user) => user.createdAt,
    ),
  ],
  getRowId: (user) => user.id,
  pagination: true,
  pageSize: 20,
  selectionMode: K1s0SelectionMode.multiple,
  onRowTap: (user) => _navigateToDetail(user),
);
```

### Form Generator

スキーマ駆動でフォームを自動生成。バリデーション、条件付き表示、グリッドレイアウトをサポート。

#### 主要コンポーネント

| コンポーネント | 説明 |
|---------------|------|
| `K1s0Form<T>` | メインフォームウィジェット |
| `K1s0FormSchema<T>` | フォームスキーマ定義 |
| `K1s0FormFieldSchema` | フィールド定義 |
| `K1s0FormController<T>` | フォーム状態管理 |

#### フィールドタイプ

| タイプ | 説明 |
|--------|------|
| `K1s0FieldType.text` | テキスト入力 |
| `K1s0FieldType.email` | メールアドレス入力 |
| `K1s0FieldType.password` | パスワード入力 |
| `K1s0FieldType.number` | 数値入力 |
| `K1s0FieldType.select` | ドロップダウン選択 |
| `K1s0FieldType.radio` | ラジオボタン |
| `K1s0FieldType.checkbox` | チェックボックス |
| `K1s0FieldType.switchField` | スイッチ |
| `K1s0FieldType.date` | 日付選択 |
| `K1s0FieldType.slider` | スライダー |

#### 使用例

```dart
final userSchema = K1s0FormSchema<UserInput>(
  fields: [
    K1s0FormFieldSchema(
      name: 'name',
      label: '氏名',
      required: true,
    ),
    K1s0FormFieldSchema(
      name: 'email',
      label: 'メールアドレス',
      type: K1s0FieldType.email,
      required: true,
    ),
    K1s0FormFieldSchema(
      name: 'role',
      label: '権限',
      type: K1s0FieldType.select,
      options: [
        K1s0FieldOption(label: '管理者', value: 'admin'),
        K1s0FieldOption(label: '一般', value: 'user'),
      ],
    ),
  ],
  fromMap: (map) => UserInput.fromMap(map),
  toMap: (user) => user.toMap(),
);

K1s0Form<UserInput>(
  schema: userSchema,
  onSubmit: (values) async {
    await createUser(values);
  },
  submitLabel: '作成',
  columns: 2,
);
```

---

## k1s0_state (Flutter)

### 目的

Riverpod 状態管理ユーティリティを提供する。AsyncValue ヘルパー、状態永続化、グローバル状態管理を実現。

### モジュール構成

| モジュール | 内容 |
|-----------|------|
| `async/` | AsyncValue拡張、AsyncState、K1s0AsyncNotifier |
| `persistence/` | StateStorage、PreferencesStorage、HiveStorage、PersistedState |
| `global/` | AppState、UserPreferences、NavigationState、ConnectivityState |
| `utils/` | StateLogger、Debouncer、Throttler、StateSelector |
| `widgets/` | AsyncValueWidget、StateConsumer、StateScope |

### 使用例

```dart
// AsyncValue 拡張
final items = ref.watch(itemsProvider);
items.when2(
  data: (data) => ListView(...),
  loading: () => LoadingWidget(),
  error: (e, s) => ErrorWidget(e),
  refreshing: (data) => RefreshingWidget(data),
);

// グローバル状態
ref.read(appStateProvider.notifier).setDarkMode(true);
final isDark = ref.watch(isDarkModeProvider);

// 状態永続化
final storage = await PreferencesStorage.create();
ref.read(userPreferencesProvider.notifier).initialize(storage);

// デバウンス
final debouncer = Debouncer(duration: Duration(milliseconds: 300));
debouncer.run(() => search(query));

// 状態ログ
K1s0StateProvider(
  enableLogging: true,
  child: MyApp(),
)
```
