# @k1s0/auth-client

React アプリケーション向けの認証クライアントライブラリ。

## 概要

JWT/OIDC トークン管理、認証状態管理、自動リフレッシュ、認証ガードを提供します。バックエンドの `k1s0-auth` crate と連携して動作することを想定しています。

### 主な機能

- **JWT トークン管理**: デコード、有効期限確認、自動リフレッシュ
- **認証状態管理**: AuthProvider、useAuth フック
- **認証ガード**: AuthGuard コンポーネント、RequireAuth/RequireRole/RequirePermission
- **セッション管理**: アイドルタイムアウト、アクティビティトラッキング
- **権限チェック**: ロール/パーミッションベースの認可

## インストール

```bash
pnpm add @k1s0/auth-client
```

### Peer Dependencies

```bash
pnpm add react react-dom react-router-dom
```

## 使用方法

### 基本セットアップ

```tsx
import { AuthProvider } from '@k1s0/auth-client';

function App() {
  return (
    <AuthProvider
      config={{
        // トークンリフレッシュ関数
        refreshToken: async (refreshToken) => {
          const response = await fetch('/api/auth/refresh', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ refresh_token: refreshToken }),
          });
          if (!response.ok) return null;
          const data = await response.json();
          return {
            accessToken: data.access_token,
            refreshToken: data.refresh_token,
            expiresAt: Date.now() + data.expires_in * 1000,
          };
        },
        // 認証エラー時のコールバック
        onAuthError: (error) => {
          console.error('Auth error:', error);
          window.location.href = '/login';
        },
        // ログアウト後のコールバック
        onLogout: () => {
          window.location.href = '/login';
        },
      }}
    >
      <YourApp />
    </AuthProvider>
  );
}
```

### ログイン処理

```tsx
import { useAuth } from '@k1s0/auth-client';

function LoginPage() {
  const { login } = useAuth();

  const handleLogin = async (credentials: { email: string; password: string }) => {
    const response = await fetch('/api/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(credentials),
    });

    if (!response.ok) {
      throw new Error('Login failed');
    }

    const data = await response.json();

    // トークンを保存（自動的に認証状態になる）
    login({
      accessToken: data.access_token,
      refreshToken: data.refresh_token,
      expiresAt: Date.now() + data.expires_in * 1000,
    });
  };

  return <LoginForm onSubmit={handleLogin} />;
}
```

### 認証状態の確認

```tsx
import { useAuth, useAuthState, useIsAuthenticated, useUser } from '@k1s0/auth-client';

function Header() {
  const { state, logout } = useAuth();
  const isAuthenticated = useIsAuthenticated();
  const user = useUser();

  if (state.isLoading) {
    return <div>Loading...</div>;
  }

  if (!isAuthenticated) {
    return <a href="/login">ログイン</a>;
  }

  return (
    <div>
      <span>ようこそ、{user?.id}さん</span>
      <button onClick={logout}>ログアウト</button>
    </div>
  );
}
```

### 認証ガード

```tsx
import { RequireAuth, RequireRole, AuthGuard } from '@k1s0/auth-client';
import { useNavigate } from 'react-router-dom';

// 認証が必要なページ
function ProtectedPage() {
  const navigate = useNavigate();

  return (
    <RequireAuth
      redirectTo="/login"
      navigate={navigate}
      loadingComponent={<div>Loading...</div>}
    >
      <div>認証済みユーザーのみ表示</div>
    </RequireAuth>
  );
}

// 管理者のみアクセス可能なページ
function AdminPage() {
  const navigate = useNavigate();

  return (
    <RequireRole
      role="admin"
      redirectTo="/login"
      forbiddenRedirectTo="/forbidden"
      navigate={navigate}
    >
      <div>管理者のみ表示</div>
    </RequireRole>
  );
}

// カスタム認可ロジック
function CustomAuthPage() {
  const navigate = useNavigate();

  return (
    <AuthGuard
      authorize={async (user) => {
        // カスタムの認可チェック
        return user.tenantId === 'allowed-tenant';
      }}
      redirectTo="/login"
      forbiddenRedirectTo="/forbidden"
      navigate={navigate}
    >
      <div>カスタム認可チェックをパスしたユーザーのみ表示</div>
    </AuthGuard>
  );
}
```

### HOC による保護

```tsx
import { withAuth, withRequireRole } from '@k1s0/auth-client';

// 認証が必要なコンポーネント
const ProtectedComponent = withAuth(MyComponent, {
  redirectTo: '/login',
});

// 管理者のみのコンポーネント
const AdminComponent = withRequireRole(MyComponent, ['admin'], {
  redirectTo: '/login',
  forbiddenRedirectTo: '/forbidden',
});
```

### 権限チェック

```tsx
import { usePermissions, useAuth } from '@k1s0/auth-client';

function ActionButtons() {
  const { hasRole, hasPermission, hasAnyRole, hasAllPermissions } = usePermissions();

  return (
    <div>
      {hasRole('admin') && <button>管理者メニュー</button>}
      {hasPermission('user.delete') && <button>ユーザー削除</button>}
      {hasAnyRole(['editor', 'admin']) && <button>編集</button>}
      {hasAllPermissions(['read', 'write']) && <button>読み書き</button>}
    </div>
  );
}
```

### セッション管理

```tsx
import { useSession } from '@k1s0/auth-client';

function SessionStatus() {
  const { session, isSessionValid, extendSession } = useSession({
    sessionDurationMs: 24 * 60 * 60 * 1000, // 24時間
    idleTimeoutMs: 30 * 60 * 1000, // 30分
    onIdleTimeout: () => {
      alert('セッションがタイムアウトしました');
    },
  });

  if (!session) {
    return null;
  }

  return (
    <div>
      <p>セッションID: {session.id}</p>
      <p>最終アクティブ: {new Date(session.lastActiveAt).toLocaleString()}</p>
      <button onClick={() => extendSession()}>セッションを延長</button>
    </div>
  );
}
```

### トークンストレージの選択

```tsx
import {
  AuthProvider,
  SessionTokenStorage,
  LocalTokenStorage,
  MemoryTokenStorage,
} from '@k1s0/auth-client';

// SessionStorage（デフォルト）- タブを閉じるとクリア
<AuthProvider config={{ storage: new SessionTokenStorage() }}>

// LocalStorage - 永続化（"Remember me" 用）
<AuthProvider config={{ storage: new LocalTokenStorage() }}>

// Memory - テスト用またはセキュリティ要件が高い場合
<AuthProvider config={{ storage: new MemoryTokenStorage() }}>
```

## API リファレンス

### AuthProvider

| Prop | Type | Description |
|------|------|-------------|
| `config.storage` | `TokenStorage` | トークンストレージ（デフォルト: SessionTokenStorage） |
| `config.refreshToken` | `(refreshToken: string) => Promise<TokenPair \| null>` | トークンリフレッシュ関数 |
| `config.refreshMarginMs` | `number` | リフレッシュマージン（デフォルト: 60000ms） |
| `config.autoRefresh` | `boolean` | 自動リフレッシュ（デフォルト: true） |
| `config.onAuthError` | `(error: AuthError) => void` | 認証エラーコールバック |
| `config.onLogout` | `() => void` | ログアウトコールバック |

### useAuth

```ts
interface AuthContextValue {
  state: AuthState;
  tokenManager: TokenManager;
  login: (tokens: TokenPair) => void;
  logout: () => void;
  refreshToken: () => Promise<boolean>;
  handleAuthError: (error?: AuthError) => void;
  hasRole: (role: string) => boolean;
  hasPermission: (permission: string) => boolean;
  hasAnyRole: (roles: string[]) => boolean;
  hasAllPermissions: (permissions: string[]) => boolean;
}
```

### AuthGuard

| Prop | Type | Description |
|------|------|-------------|
| `roles` | `string[]` | 必要なロール（いずれか） |
| `permissions` | `string[]` | 必要なパーミッション（すべて） |
| `redirectTo` | `string` | 未認証時のリダイレクト先 |
| `forbiddenRedirectTo` | `string` | 権限不足時のリダイレクト先 |
| `authorize` | `(user: AuthUser) => boolean \| Promise<boolean>` | カスタム認可関数 |
| `navigate` | `(to: string) => void` | リダイレクト関数 |
| `loadingComponent` | `ReactNode` | ローディング中の表示 |
| `unauthenticatedComponent` | `ReactNode` | 未認証時の表示 |
| `forbiddenComponent` | `ReactNode` | 権限不足時の表示 |

## JWT Claims 形式

バックエンドは以下の形式で Claims を含む JWT を発行する必要があります。

```json
{
  "sub": "user-123",
  "iss": "https://auth.example.com",
  "aud": "my-app",
  "exp": 1735689600,
  "iat": 1735686000,
  "roles": ["user", "admin"],
  "permissions": ["read", "write", "delete"],
  "tenant_id": "tenant-456"
}
```

## 設計原則

1. **トークン自動管理**: 有効期限の監視、自動リフレッシュ
2. **認可の分離**: 認証（Authentication）と認可（Authorization）を明確に分離
3. **セキュリティ**: デフォルトでSessionStorage使用、メモリストレージオプション
4. **柔軟性**: RBAC/ABAC/カスタム認可をサポート
5. **React Router 統合**: navigate 関数による統合
