/// k1s0 Authentication Client Library
///
/// Provides JWT/OIDC token management, secure storage, authentication state
/// management with Riverpod, and GoRouter integration for protected routes.
library k1s0_auth;

export 'src/guard/auth_guard.dart';
export 'src/provider/auth_provider.dart';
export 'src/provider/auth_state.dart';
export 'src/storage/memory_token_storage.dart';
export 'src/storage/secure_token_storage.dart';
export 'src/storage/token_storage.dart';
export 'src/token/claims.dart';
export 'src/token/token_decoder.dart';
export 'src/token/token_manager.dart';
export 'src/token/token_pair.dart';
export 'src/types/auth_config.dart';
export 'src/types/auth_error.dart';
export 'src/types/auth_user.dart';
