# k1s0_auth

Authentication client for k1s0 Flutter applications with JWT/OIDC support.

## Features

- JWT token management and decoding
- Secure token storage with flutter_secure_storage
- Authentication state management with Riverpod
- Automatic token refresh
- GoRouter integration for protected routes
- Role and permission-based access control

## Installation

Add to your `pubspec.yaml`:

```yaml
dependencies:
  k1s0_auth:
    path: ../packages/k1s0_auth
```

## Basic Usage

### Setup Provider

```dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:k1s0_auth/k1s0_auth.dart';

void main() {
  runApp(
    ProviderScope(
      overrides: [
        // Optionally override the token refresher
        tokenRefresherProvider.overrideWithValue(
          (refreshToken) async {
            // Call your API to refresh the token
            final response = await api.post('/auth/refresh', {
              'refresh_token': refreshToken,
            });
            return TokenPair.fromJson(response.data);
          },
        ),
      ],
      child: const MyApp(),
    ),
  );
}
```

### Login

```dart
class LoginPage extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return ElevatedButton(
      onPressed: () async {
        // Get tokens from your auth API
        final tokens = TokenPair(
          accessToken: 'jwt.token.here',
          refreshToken: 'refresh.token.here',
        );

        // Login
        await ref.read(authProvider.notifier).login(tokens);
      },
      child: const Text('Login'),
    );
  }
}
```

### Check Authentication State

```dart
class HomePage extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final authState = ref.watch(authProvider);
    final user = ref.watch(currentUserProvider);
    final isAuthenticated = ref.watch(isAuthenticatedProvider);

    if (authState.isLoading) {
      return const CircularProgressIndicator();
    }

    if (!isAuthenticated) {
      return const LoginPage();
    }

    return Text('Welcome, ${user?.id}');
  }
}
```

### Logout

```dart
ElevatedButton(
  onPressed: () async {
    await ref.read(authProvider.notifier).logout();
  },
  child: const Text('Logout'),
)
```

### GoRouter Integration

```dart
import 'package:go_router/go_router.dart';
import 'package:k1s0_auth/k1s0_auth.dart';

class AppRouter extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final router = GoRouter(
      redirect: createAuthRedirect(
        ref,
        loginPath: '/login',
        homePath: '/home',
      ),
      routes: [
        GoRoute(
          path: '/login',
          builder: (context, state) => const LoginPage(),
        ),
        GoRoute(
          path: '/home',
          builder: (context, state) => const HomePage(),
        ),
        GoRoute(
          path: '/admin',
          redirect: createAuthRedirect(
            ref,
            loginPath: '/login',
            homePath: '/home',
            roles: ['admin'],
          ),
          builder: (context, state) => const AdminPage(),
        ),
      ],
    );

    return MaterialApp.router(
      routerConfig: router,
    );
  }
}
```

### Protected Widgets

```dart
// Show content only when authenticated
RequireAuth(
  loading: const CircularProgressIndicator(),
  fallback: const Text('Please login'),
  child: const UserProfile(),
)

// Show content only for specific roles
RequireRole(
  roles: ['admin', 'moderator'],
  fallback: const Text('Access denied'),
  child: const AdminPanel(),
)

// Show content only for specific permissions
RequirePermission(
  permissions: ['users:read'],
  requireAll: true,
  fallback: const Text('No permission'),
  child: const UserList(),
)
```

### Check Roles and Permissions

```dart
// Using providers
final isAdmin = ref.watch(hasRoleProvider('admin'));
final canEdit = ref.watch(hasPermissionProvider('posts:write'));
final isManager = ref.watch(hasAnyRoleProvider(['admin', 'manager']));

// Using user object
final user = ref.watch(currentUserProvider);
if (user?.hasRole('admin') ?? false) {
  // Show admin content
}
```

### Custom Token Storage

```dart
// Use in-memory storage (for testing)
ProviderScope(
  overrides: [
    tokenStorageProvider.overrideWithValue(MemoryTokenStorage()),
  ],
  child: const MyApp(),
)

// Use custom secure storage
ProviderScope(
  overrides: [
    tokenStorageProvider.overrideWithValue(
      SecureTokenStorage(keyPrefix: 'my_app'),
    ),
  ],
  child: const MyApp(),
)
```

## Token Decoding

```dart
// Decode a JWT token
final claims = TokenDecoder.decode(token);
print('User ID: ${claims.sub}');
print('Roles: ${claims.roles}');
print('Expires: ${claims.expirationTime}');
print('Is expired: ${claims.isExpired}');

// Safe decoding
final claims = TokenDecoder.tryDecode(token);
if (claims != null) {
  // Use claims
}

// Check expiration
if (TokenDecoder.isExpired(token)) {
  // Token is expired
}
```

## API Reference

### Providers

| Provider | Type | Description |
|----------|------|-------------|
| `authProvider` | `AuthState` | Main auth state |
| `currentUserProvider` | `AuthUser?` | Current user or null |
| `isAuthenticatedProvider` | `bool` | Whether authenticated |
| `hasRoleProvider(role)` | `bool` | Check if user has role |
| `hasAnyRoleProvider(roles)` | `bool` | Check if user has any role |
| `hasPermissionProvider(perm)` | `bool` | Check if user has permission |
| `hasAnyPermissionProvider(perms)` | `bool` | Check if user has any permission |

### AuthState

| Property | Type | Description |
|----------|------|-------------|
| `status` | `AuthStatus` | Current auth status |
| `user` | `AuthUser?` | Authenticated user |
| `error` | `AuthError?` | Error information |
| `isLoading` | `bool` | Whether loading |
| `isAuthenticated` | `bool` | Whether authenticated |
| `isInitialized` | `bool` | Whether initial check complete |

### AuthUser

| Property | Type | Description |
|----------|------|-------------|
| `id` | `String` | User ID |
| `roles` | `List<String>` | User roles |
| `permissions` | `List<String>` | User permissions |
| `tenantId` | `String?` | Tenant ID |
| `claims` | `Claims` | Full JWT claims |

### Claims

| Property | Type | Description |
|----------|------|-------------|
| `sub` | `String` | Subject (user ID) |
| `iss` | `String` | Issuer |
| `aud` | `List<String>?` | Audience |
| `exp` | `int` | Expiration timestamp |
| `iat` | `int` | Issued at timestamp |
| `roles` | `List<String>` | User roles |
| `permissions` | `List<String>` | User permissions |
| `tenantId` | `String?` | Tenant ID |

### AuthNotifier Methods

| Method | Description |
|--------|-------------|
| `initialize()` | Initialize from stored tokens |
| `login(tokens)` | Login with token pair |
| `logout()` | Logout and clear tokens |
| `refresh()` | Manually refresh token |
| `getAccessToken()` | Get current access token |

## License

MIT
