# k1s0_state

State management utilities for k1s0 Flutter applications with Riverpod integration.

## Features

- AsyncValue helpers and extensions
- State persistence (SharedPreferences/Hive)
- Global app state management
- State logging and debugging
- Debounce and throttle utilities
- Convenient state consumer widgets

## Installation

Add to your `pubspec.yaml`:

```yaml
dependencies:
  k1s0_state:
    path: ../packages/k1s0_state
```

## Basic Usage

### App Setup

```dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:k1s0_state/k1s0_state.dart';

void main() {
  runApp(
    K1s0StateProvider(
      enableLogging: true, // Enable state logging in debug mode
      child: const MyApp(),
    ),
  );
}
```

### AsyncValue Extensions

```dart
final dataProvider = FutureProvider<List<Item>>((ref) async {
  return await fetchItems();
});

class ItemList extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final items = ref.watch(dataProvider);

    // Check states
    if (items.isInitialLoading) {
      return const LoadingWidget();
    }

    if (items.isRefreshing) {
      // Show data with refresh indicator
    }

    // Use when2 for custom refresh handling
    return items.when2(
      data: (data) => ListView.builder(...),
      loading: () => const LoadingWidget(),
      error: (error, stackTrace) => ErrorWidget(error),
      refreshing: (data) => RefreshingWidget(data),
    );

    // Or use getOrElse for default values
    final itemList = items.getOrElse([]);

    // Combine multiple AsyncValues
    final combined = items.combine(ref.watch(otherProvider));
  }
}
```

### AsyncValue Widget

```dart
// Simple usage
AsyncValueWidget<User>(
  value: ref.watch(userProvider),
  data: (user) => UserCard(user),
  onRetry: () => ref.invalidate(userProvider),
)

// Watch a provider directly
AsyncProviderWidget<List<Item>>(
  provider: itemsProvider,
  data: (items, ref) => ItemList(items),
  loading: () => const ShimmerLoading(),
  error: (error, stack) => ErrorState(error: error),
)

// Multiple AsyncValues
MultiAsyncValueWidget<User, Settings>(
  value1: ref.watch(userProvider),
  value2: ref.watch(settingsProvider),
  data: (user, settings) => ProfileScreen(user, settings),
)
```

### Global App State

```dart
// Access app state
class MyWidget extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final isInitialized = ref.watch(appInitializedProvider);
    final environment = ref.watch(environmentProvider);
    final isDarkMode = ref.watch(isDarkModeProvider);

    // Update app state
    ref.read(appStateProvider.notifier).setDarkMode(true);
    ref.read(appStateProvider.notifier).setFeatureFlag('newFeature', true);

    // Check feature flags
    final isFeatureEnabled = ref.watch(isFeatureEnabledProvider('newFeature'));
  }
}
```

### User Preferences

```dart
// Initialize with storage
void initializePreferences(WidgetRef ref, StateStorage storage) {
  ref.read(userPreferencesProvider.notifier).initialize(storage);
}

// Use preferences
class SettingsPage extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final prefs = ref.watch(userPreferencesProvider);
    final themeMode = ref.watch(themeModePreferenceProvider);

    return SwitchListTile(
      title: const Text('Notifications'),
      value: prefs.notificationsEnabled,
      onChanged: (value) {
        ref.read(userPreferencesProvider.notifier)
            .setNotificationsEnabled(value);
      },
    );
  }
}
```

### State Persistence

```dart
// Using SharedPreferences
final storage = await PreferencesStorage.create();
final value = await storage.read<String>('key');
await storage.write('key', 'value');

// Using Hive for complex data
await HiveStorage.initialize();
final hiveStorage = await HiveStorage.create(boxName: 'myBox');
await hiveStorage.write('user', userObject);

// Typed storage
final userStorage = TypedHiveStorage<User>(
  storage: hiveStorage,
  key: 'current_user',
);
final user = await userStorage.read();

// Persisted StateNotifier
class CounterNotifier extends PersistedStateNotifier<int> {
  CounterNotifier(StateStorage storage)
      : super(0, storage: storage, key: 'counter');

  @override
  Map<String, dynamic> toJson(int state) => {'value': state};

  @override
  int fromJson(Map<String, dynamic> json) => json['value'] as int;

  void increment() => state++;
}
```

### State Consumers

```dart
// Consumer with state change callback
StateConsumer<int>(
  provider: counterProvider,
  builder: (context, count, ref) => Text('Count: $count'),
  onStateChanged: (previous, current) {
    if (current > 10) {
      showSnackbar('Count exceeded 10!');
    }
  },
  listenWhen: (previous, current) => current != previous,
)

// Listener without rebuild
StateListener<AuthState>(
  provider: authStateProvider,
  onStateChanged: (context, previous, current) {
    if (current is Unauthenticated) {
      Navigator.pushReplacementNamed(context, '/login');
    }
  },
  child: const HomePage(),
)

// Selective consumer
SelectiveConsumer<AppState, bool>(
  provider: appStateProvider,
  selector: (state) => state.isDarkMode,
  builder: (context, isDark, ref) => Text(isDark ? 'Dark' : 'Light'),
)
```

### Debounce and Throttle

```dart
// Debounce search input
final debouncer = Debouncer(duration: Duration(milliseconds: 300));

TextField(
  onChanged: (value) {
    debouncer.run(() {
      ref.read(searchProvider.notifier).search(value);
    });
  },
)

// Throttle API calls
final throttler = Throttler(duration: Duration(seconds: 1));

void onScroll() {
  throttler.run(() {
    loadMoreItems();
  });
}

// Async debouncer
final asyncDebouncer = AsyncDebouncer<List<Item>>(
  duration: Duration(milliseconds: 500),
);

Future<void> search(String query) async {
  final results = await asyncDebouncer.run(() => api.search(query));
  // Use results
}
```

### State Logging

```dart
// Enable logging
K1s0StateProvider(
  enableLogging: true,
  child: MyApp(),
)

// Or custom logger
K1s0StateProvider(
  observers: [
    StateLogger(
      logUpdates: true,
      logDispose: true,
      logErrors: true,
    ),
  ],
  child: MyApp(),
)

// Filter specific providers
K1s0StateProvider(
  observers: [
    filteredStateLogger(['userProvider', 'authProvider']),
  ],
  child: MyApp(),
)
```

### State Selectors

```dart
// Create reusable selectors
final userNameSelector = StateSelector<UserState, String>(
  (state) => state.user?.name ?? 'Guest',
);

// Use in widgets
final name = ref.watch(userStateProvider.select(userNameSelector._selector));

// Or create derived provider
final userNameProvider = userNameSelector.createProvider(userStateProvider);
final name = ref.watch(userNameProvider);

// Memoized selector for expensive computations
final expensiveSelector = createMemoizedSelector<AppState, ComputedValue>(
  (state) => computeExpensiveValue(state),
);
```

### State Initialization

```dart
// Initialize state on app start
StateInitializer(
  initialize: (ref) async {
    // Load persisted state
    final storage = await PreferencesStorage.create();
    ref.read(userPreferencesProvider.notifier).initialize(storage);

    // Initialize other state
    await ref.read(authProvider.notifier).checkAuthStatus();

    // Mark app as initialized
    ref.read(appStateProvider.notifier).setInitialized();
  },
  loading: const SplashScreen(),
  child: const MainApp(),
)
```

### Ref Extensions

```dart
// Watch multiple providers at once
final (user, settings) = ref.watch2(userProvider, settingsProvider);
final (a, b, c) = ref.watch3(provider1, provider2, provider3);

// Read multiple providers
final (user, settings) = ref.read2(userProvider, settingsProvider);
```

## AsyncState (Alternative to AsyncValue)

```dart
// More explicit state handling
AsyncState<User> state = const AsyncState.initial();
state = const AsyncState.loading();
state = AsyncState.success(user);
state = AsyncState.failure(error, stackTrace: trace);

// Pattern matching
final widget = state.when(
  initial: () => const InitialWidget(),
  loading: (previousData) => LoadingWidget(previousData: previousData),
  success: (data) => SuccessWidget(data),
  failure: (error, stackTrace, previousData) => ErrorWidget(error),
);

// State transitions
state = state.toLoading(); // Preserves previous data
state = state.toFailure(error); // Preserves previous data
```

## Providers

| Provider | Type | Description |
|----------|------|-------------|
| `appStateProvider` | `AppState` | Global app state |
| `appInitializedProvider` | `bool` | App initialized flag |
| `appLoadingProvider` | `bool` | Global loading flag |
| `environmentProvider` | `String` | Current environment |
| `localeProvider` | `String` | Current locale |
| `isDarkModeProvider` | `bool` | Dark mode flag |
| `featureFlagsProvider` | `Map<String, bool>` | Feature flags |
| `isFeatureEnabledProvider` | `bool` (family) | Check specific feature |
| `userPreferencesProvider` | `UserPreferences` | User preferences |
| `themeModePreferenceProvider` | `String` | Theme mode preference |
| `navigationStateProvider` | `NavigationState` | Navigation state |
| `currentPathProvider` | `String` | Current route path |
| `connectivityStateProvider` | `ConnectivityState` | Connectivity state |
| `isConnectedProvider` | `bool` | Internet connection flag |

## License

MIT
