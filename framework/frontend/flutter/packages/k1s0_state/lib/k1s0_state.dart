/// State management utilities for k1s0 Flutter applications.
///
/// This library provides Riverpod-based state management utilities,
/// including AsyncValue helpers, state persistence, and global state management.
library k1s0_state;

// AsyncValue helpers
export 'src/async/async_notifier_base.dart';
export 'src/async/async_state.dart';
export 'src/async/async_value_extensions.dart';

// Global state
export 'src/global/app_state.dart';
export 'src/global/app_state_provider.dart';

// Persistence
export 'src/persistence/hive_storage.dart';
export 'src/persistence/persisted_state.dart';
export 'src/persistence/preferences_storage.dart';
export 'src/persistence/state_storage.dart';

// Utilities
export 'src/utils/debouncer.dart';
export 'src/utils/state_logger.dart';
export 'src/utils/state_selector.dart';

// Widgets
export 'src/widgets/async_value_widget.dart';
export 'src/widgets/state_consumer.dart';
export 'src/widgets/state_scope.dart';
