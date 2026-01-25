# @k1s0/api-client

React アプリケーション向けの統一 API クライアントライブラリ。

## 概要

生成 SDK（OpenAPI/gRPC-web 等）を直接使用させず、認証/タイムアウト/リトライ/計測を含む共通クライアントを提供します。

### 主な機能

- **統一 transport**: fetch ベースの共通クライアント
- **認証付与**: トークン取得/refresh の自動化
- **タイムアウト/リトライ**: 既定値の強制（原則 retry 0）
- **エラー統一**: ProblemDetails 形式 + `error_code`
- **標準 UI**: エラー表示/リトライ導線/`trace_id` 表示
- **OTel 計測**: フロント側のトレーシング対応

## インストール

```bash
pnpm add @k1s0/api-client
```

### Peer Dependencies

```bash
pnpm add react react-dom @mui/material @opentelemetry/api
```

## 使用方法

### 基本セットアップ

```tsx
import {
  AuthProvider,
  ApiClientProvider,
  ApiErrorBoundary,
} from '@k1s0/api-client';

function App() {
  return (
    <AuthProvider
      config={{
        // トークンリフレッシュ関数（オプション）
        refreshToken: async (refreshToken) => {
          const response = await fetch('/api/auth/refresh', {
            method: 'POST',
            body: JSON.stringify({ refreshToken }),
          });
          if (!response.ok) return null;
          return response.json();
        },
        // 認証エラー時のコールバック
        onAuthError: () => {
          window.location.href = '/login';
        },
      }}
    >
      <ApiClientProvider
        config={{
          baseUrl: '/api',
          timeout: 30000,
        }}
      >
        <ApiErrorBoundary>
          <YourApp />
        </ApiErrorBoundary>
      </ApiClientProvider>
    </AuthProvider>
  );
}
```

### ログイン処理

```tsx
import { useAuth } from '@k1s0/api-client';

function LoginPage() {
  const { login } = useAuth();

  const handleLogin = async (credentials) => {
    const response = await fetch('/api/auth/login', {
      method: 'POST',
      body: JSON.stringify(credentials),
    });
    const tokens = await response.json();

    // トークンを保存（自動的に認証状態になる）
    login({
      accessToken: tokens.access_token,
      refreshToken: tokens.refresh_token,
      expiresAt: Date.now() + tokens.expires_in * 1000,
    });
  };

  return <LoginForm onSubmit={handleLogin} />;
}
```

### API リクエスト

```tsx
import { useApiClient, useApiRequest, AsyncContent } from '@k1s0/api-client';

// 直接クライアントを使用
function DirectUsage() {
  const client = useApiClient();

  const fetchData = async () => {
    const response = await client.get<User[]>('/users');
    return response.data;
  };
}

// フック経由で使用
function HookUsage() {
  const { state, execute, isLoading } = useApiRequest<User[]>('/users');

  return (
    <div>
      <button onClick={execute} disabled={isLoading}>
        データ取得
      </button>
      <AsyncContent state={state} onRetry={execute}>
        {(users) => (
          <ul>
            {users.map((user) => (
              <li key={user.id}>{user.name}</li>
            ))}
          </ul>
        )}
      </AsyncContent>
    </div>
  );
}
```

### ミューテーション

```tsx
import { useApiClient, useMutation } from '@k1s0/api-client';

function CreateUserForm() {
  const client = useApiClient();

  const { mutate, isLoading, error } = useMutation<User, CreateUserInput>(
    async (input) => {
      const response = await client.post<User>('/users', input);
      return response.data;
    }
  );

  const handleSubmit = async (data: CreateUserInput) => {
    try {
      const user = await mutate(data);
      console.log('Created:', user);
    } catch (err) {
      // エラーは error 変数で参照可能
    }
  };

  return (
    <form onSubmit={handleSubmit}>
      {error && <ErrorDisplay error={error} />}
      {/* フォームフィールド */}
    </form>
  );
}
```

### エラー表示

```tsx
import { ErrorDisplay, InlineError, ApiError } from '@k1s0/api-client';

function ErrorExample({ error }: { error: ApiError }) {
  return (
    <div>
      {/* 標準エラー表示 */}
      <ErrorDisplay
        error={error}
        onRetry={() => {/* リトライ処理 */}}
        showDetails={process.env.NODE_ENV === 'development'}
      />

      {/* フィールド単位のエラー */}
      <TextField
        name="email"
        helperText={<InlineError error={error} field="email" />}
      />
    </div>
  );
}
```

## エラーの種類

| kind | 説明 | リトライ |
|------|------|----------|
| `validation` | 入力不備（400） | 不可 |
| `authentication` | 認証エラー（401） | 不可 |
| `authorization` | 認可エラー（403） | 不可 |
| `not_found` | リソース不存在（404） | 不可 |
| `conflict` | 競合（409） | 不可 |
| `rate_limit` | レート制限（429） | 不可 |
| `dependency` | 依存先障害（502/503） | 可 |
| `temporary` | 一時障害 | 可 |
| `timeout` | タイムアウト | 可 |
| `network` | ネットワークエラー | 可 |

## ProblemDetails 形式

バックエンドは RFC 7807 + k1s0 拡張形式でエラーを返す必要があります：

```json
{
  "type": "about:blank",
  "title": "Bad Request",
  "status": 400,
  "detail": "メールアドレスの形式が正しくありません",
  "error_code": "INVALID_EMAIL",
  "trace_id": "abc123...",
  "errors": [
    {
      "field": "email",
      "message": "有効なメールアドレスを入力してください",
      "code": "FORMAT"
    }
  ]
}
```

## OTel 計測

```tsx
import * as otelApi from '@opentelemetry/api';
import { defaultTelemetry } from '@k1s0/api-client';

// OTel Tracer を設定
const tracer = otelApi.trace.getTracer('frontend');
defaultTelemetry.setTracer(tracer, otelApi);

// テレメトリーイベントのリスニング（OTel無しでも動作）
defaultTelemetry.addListener((event) => {
  console.log(`${event.type}: ${event.telemetry.method} ${event.telemetry.url}`);
  if (event.type === 'request_end') {
    const duration = event.telemetry.endTime! - event.telemetry.startTime;
    console.log(`Duration: ${duration.toFixed(2)}ms`);
  }
});
```

## 設計原則

1. **retry 原則 0**: リトライはデフォルト無効。必要な場合のみ opt-in
2. **timeout 必須**: すべてのリクエストにタイムアウトを設定
3. **エラー統一**: 画面ごとの流儀を許さず、同じ形式で返す
4. **trace_id 必須**: すべてのエラーで調査の入口を提供
5. **認証自動化**: トークン付与/リフレッシュを自動化

## 完了条件

- [x] 生成 SDK と共通 transport（fetch）を繋ぐラッパ
- [x] 認証付与（token 取得/refresh の共通化）
- [x] timeout/retry の既定（原則 retry 0）
- [x] 失敗時のエラー表現統一（problem details + `error_code`）
- [x] problem details の標準 UI（エラー表示/リトライ導線/`trace_id` 表示）
- [x] フロント側の計測（OTel）
