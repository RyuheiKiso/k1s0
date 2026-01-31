# k1s0_state (Flutter)

← [Flutter パッケージ一覧](./)

## 目的

Riverpod 状態管理ユーティリティを提供する。AsyncValue ヘルパー、状態永続化、グローバル状態管理を実現。

## モジュール構成

| モジュール | 内容 |
|-----------|------|
| `async/` | AsyncValue拡張、AsyncState、K1s0AsyncNotifier |
| `persistence/` | StateStorage、PreferencesStorage、HiveStorage、PersistedState |
| `global/` | AppState、UserPreferences、NavigationState、ConnectivityState |
| `utils/` | StateLogger、Debouncer、Throttler、StateSelector |
| `widgets/` | AsyncValueWidget、StateConsumer、StateScope |

## 使用例

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
