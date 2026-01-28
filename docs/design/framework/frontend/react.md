# React パッケージ一覧

```
framework/frontend/react/packages/
├── @k1s0/navigation/     # 設定駆動ナビゲーション（実装済み）
├── @k1s0/config/         # YAML設定管理（実装済み）
├── @k1s0/api-client/     # API通信クライアント（実装済み）
├── @k1s0/ui/             # Design System（実装済み）
├── @k1s0/shell/          # AppShell（実装済み）
├── @k1s0/auth-client/    # 認証クライアント（実装済み）
├── @k1s0/observability/  # OTel/ログ（実装済み）
├── eslint-config-k1s0/   # ESLint設定（実装済み）
└── tsconfig-k1s0/        # TypeScript設定（実装済み）
```

## 実装状況

| パッケージ | 状態 | 説明 |
|-----------|:----:|------|
| @k1s0/navigation | ✅ | 設定駆動ナビゲーション、React Router統合、権限/feature flag制御 |
| @k1s0/config | ✅ | YAML設定読み込み、Zodスキーマバリデーション、環境マージ |
| @k1s0/api-client | ✅ | fetchベースAPI通信、OTel計測、ProblemDetailsエラー |
| @k1s0/ui | ✅ | Material-UI v5/v6 Design System、テーマ、フォーム、フィードバック |
| @k1s0/shell | ✅ | AppShell（Header/Sidebar/Footer）、レスポンシブ対応 |
| @k1s0/auth-client | ✅ | JWT/OIDCトークン管理、認証ガード、セッション管理 |
| @k1s0/observability | ✅ | OpenTelemetry統合、構造化ログ、Web Vitals計測 |
| @k1s0/eslint-config | ✅ | ESLint共通設定、TypeScript/React/a11yルール、Prettier連携、k1s0固有ルール（環境変数使用禁止） |
| @k1s0/tsconfig | ✅ | TypeScript共通設定、React/ライブラリ/Node.js/Strict用プリセット、厳格な型チェック |

---

## @k1s0/navigation

### 目的

`config/{env}.yaml` の設定からルート/メニュー/フローを自動構築し、権限・feature flag による表示制御を行う。

### 主要コンポーネント

| コンポーネント | 説明 |
|---------------|------|
| `ConfigRouter` | YAML設定からReact Routerルートを自動生成 |
| `NavigationProvider` | ナビゲーション状態のContext提供 |
| `MenuBuilder` | 設定からメニュー構造を構築 |
| `FlowController` | マルチステップフロー制御 |
| `PermissionGuard` | 権限ベースのルートガード |
| `FlagGuard` | feature flagベースのガード |

### 使用例

```tsx
import { ConfigRouter, NavigationProvider } from '@k1s0/navigation';
import { useConfig } from '@k1s0/config';

function App() {
  const config = useConfig();

  return (
    <NavigationProvider>
      <ConfigRouter config={config.ui.navigation} />
    </NavigationProvider>
  );
}
```

---

## @k1s0/config

### 目的

YAML設定ファイルの読み込み、型付け、バリデーションを提供する。

### 主要機能

| モジュール | 説明 |
|-----------|------|
| `schema` | Zodスキーマ定義（apiConfigSchema, authConfigSchema, appConfigSchema） |
| `loader` | ConfigLoader, loadConfigFromUrl, parseConfig |
| `merge` | deepMerge, mergeConfigs, mergeEnvironmentConfig |

### 使用例

```tsx
import { ConfigLoader, validateConfig } from '@k1s0/config';

const loader = new ConfigLoader({
  baseUrl: '/config',
  env: 'dev',
});

const config = await loader.load();
const validated = validateConfig(config);
```

---

## @k1s0/api-client

### 目的

API通信の標準化、OpenTelemetry計測、エラーハンドリングを提供する。

### 主要コンポーネント

| コンポーネント | 説明 |
|---------------|------|
| `ApiClient` | fetchベースのHTTPクライアント |
| `ApiClientProvider` | Context提供 |
| `TokenManager` | 認証トークン管理 |
| `OTelTracer` | OpenTelemetry計測 |
| `ErrorBoundary` | エラー境界コンポーネント |
| `useApiRequest` | API呼び出しフック |

### 使用例

```tsx
import { useApiRequest } from '@k1s0/api-client';

function UserList() {
  const { data, loading, error } = useApiRequest('/api/users');

  if (loading) return <Loading />;
  if (error) return <ErrorDisplay error={error} />;

  return <ul>{data.map(user => <li key={user.id}>{user.name}</li>)}</ul>;
}
```

---

## @k1s0/ui

### 目的

k1s0 Design/UX 標準コンポーネントライブラリを提供する。Material-UI v5/v6 をベースに統一されたデザインシステムを実現。

### モジュール構成

| モジュール | 内容 |
|-----------|------|
| `theme/` | K1s0ThemeProvider, createK1s0Theme, palette, typography, spacing |
| `form/` | FormContainer, FormField, validation, types |
| `feedback/` | Toast, ConfirmDialog, FeedbackProvider |
| `state/` | Loading, EmptyState |
| `data-table/` | K1s0DataTable（MUI DataGrid ベース）、カラムヘルパー、サーバーサイドページネーション |
| `form-generator/` | createFormFromSchema（Zod + react-hook-form + MUI）、フィールド自動生成 |

### 使用例

```tsx
import { K1s0ThemeProvider, FormContainer, FormField, Toast } from '@k1s0/ui';

function App() {
  return (
    <K1s0ThemeProvider>
      <FormContainer onSubmit={handleSubmit}>
        <FormField name="email" label="メールアドレス" required />
        <FormField name="password" label="パスワード" type="password" />
      </FormContainer>
      <Toast />
    </K1s0ThemeProvider>
  );
}
```

### DataTable（MUI DataGrid ベース）

高機能データテーブルコンポーネント。MUI DataGrid をラップし、日本語ローカライズ、カスタムツールバー、CSV エクスポート機能を提供。

#### 主要コンポーネント

| コンポーネント | 説明 |
|---------------|------|
| `K1s0DataTable` | メインデータテーブルコンポーネント |
| `K1s0DataTableToolbar` | 検索・エクスポート機能付きツールバー |
| `createColumns` | 型安全なカラム定義ヘルパー |
| `dateColumn` | 日付カラムヘルパー |
| `numberColumn` | 数値カラムヘルパー（通貨・パーセント対応） |
| `booleanColumn` | ブール値カラムヘルパー |
| `actionsColumn` | アクションボタンカラムヘルパー |
| `statusColumn` | ステータスChipカラムヘルパー |
| `useServerSidePagination` | サーバーサイドページネーションフック |

#### 使用例

```tsx
import { K1s0DataTable, createColumns, dateColumn, actionsColumn } from '@k1s0/ui';

interface User {
  id: string;
  name: string;
  email: string;
  role: 'admin' | 'user';
  createdAt: Date;
}

const columns = createColumns<User>([
  { field: 'name', headerName: '氏名', flex: 1, sortable: true },
  { field: 'email', headerName: 'メール', flex: 1 },
  {
    field: 'role',
    headerName: '権限',
    width: 120,
    type: 'singleSelect',
    valueOptions: [
      { value: 'admin', label: '管理者' },
      { value: 'user', label: '一般' },
    ],
  },
  dateColumn({ field: 'createdAt', headerName: '作成日' }),
  actionsColumn({
    onEdit: (row) => navigate(`/users/${row.id}/edit`),
    onDelete: (row) => handleDelete(row.id),
  }),
]);

function UserList() {
  const { data: users, isLoading } = useUsers();

  return (
    <K1s0DataTable
      rows={users ?? []}
      columns={columns}
      loading={isLoading}
      checkboxSelection
      pagination
      pageSize={20}
      toolbar
      exportOptions={{ csv: true }}
    />
  );
}
```

### Form Generator（Zod + react-hook-form + MUI）

Zod スキーマから MUI フォームを自動生成。

#### 主要機能

| 機能 | 説明 |
|-----|------|
| `createFormFromSchema` | Zod スキーマからフォームコンポーネントを生成 |
| `useFormGenerator` | フォーム状態管理フック |
| `useConditionalField` | 条件付きフィールド表示フック |

#### 対応フィールドタイプ

| Zod 型 | 生成されるMUIコンポーネント |
|--------|---------------------------|
| `z.string()` | TextField |
| `z.string().email()` | TextField (email) |
| `z.number()` | TextField (number) |
| `z.boolean()` | Checkbox / Switch |
| `z.enum()` | Select |
| `z.date()` | DatePicker |
| `z.array()` | ArrayField（動的追加・削除） |

#### 使用例

```tsx
import { createFormFromSchema } from '@k1s0/ui';
import { z } from 'zod';

const userSchema = z.object({
  name: z.string().min(1, '名前は必須です'),
  email: z.string().email('有効なメールアドレスを入力してください'),
  age: z.number().min(0).max(120).optional(),
  role: z.enum(['admin', 'user', 'guest']),
  notifications: z.boolean().default(true),
});

const UserForm = createFormFromSchema(userSchema, {
  labels: {
    name: '氏名',
    email: 'メールアドレス',
    age: '年齢',
    role: '権限',
    notifications: '通知を受け取る',
  },
  fieldConfig: {
    role: {
      component: 'Select',
      options: [
        { label: '管理者', value: 'admin' },
        { label: '一般ユーザー', value: 'user' },
        { label: 'ゲスト', value: 'guest' },
      ],
    },
    notifications: { component: 'Switch' },
  },
  columns: 2,
  submitLabel: '保存',
});

function CreateUserPage() {
  return (
    <UserForm
      defaultValues={{ role: 'user', notifications: true }}
      onSubmit={async (values) => {
        await createUser(values);
        navigate('/users');
      }}
    />
  );
}
```

---

## @k1s0/shell

### 目的

AppShell（Header/Sidebar/Footer）の標準レイアウトを提供する。

### 主要コンポーネント

| コンポーネント | 説明 |
|---------------|------|
| `AppShell` | メインレイアウトコンテナ |
| `Header` | ヘッダーコンポーネント |
| `Sidebar` | サイドバー（メニュー）コンポーネント |
| `Footer` | フッターコンポーネント |
| `useResponsiveLayout` | レスポンシブ対応フック |

### 使用例

```tsx
import { AppShell, Header, Sidebar, Footer } from '@k1s0/shell';
import { useConfig } from '@k1s0/config';

function Layout({ children }) {
  const config = useConfig();

  return (
    <AppShell>
      <Header title={config.app.name} />
      <Sidebar menuItems={config.ui.navigation.menus} />
      <main>{children}</main>
      <Footer />
    </AppShell>
  );
}
```

---

## @k1s0/auth-client

### 目的

JWT/OIDC 認証クライアントを提供する。トークン管理、認証状態管理、認証ガード、セッション管理を実現。

### モジュール構成

| モジュール | 内容 |
|-----------|------|
| `token/` | TokenManager, SessionTokenStorage, LocalTokenStorage, MemoryTokenStorage, decoder |
| `provider/` | AuthProvider, useAuth, useAuthState, useIsAuthenticated, useUser, usePermissions |
| `guard/` | AuthGuard, RequireAuth, RequireRole, RequirePermission |
| `session/` | SessionManager, useSession |

### 主要な型

```typescript
interface Claims {
  sub: string;           // ユーザーID
  iss: string;           // 発行者
  aud?: string | string[]; // 対象者
  exp: number;           // 有効期限
  iat: number;           // 発行日時
  roles?: string[];      // ロール
  permissions?: string[]; // パーミッション
  tenant_id?: string;    // テナントID
}

interface AuthState {
  status: 'idle' | 'loading' | 'authenticated' | 'unauthenticated' | 'error';
  user: AuthUser | null;
  error: AuthError | null;
}

interface AuthClientConfig {
  tokenStorage?: TokenStorage;
  refreshToken?: TokenRefresher;
  refreshThreshold?: number;  // デフォルト: 300秒（5分）
  autoRefresh?: boolean;
}
```

### 使用例

```tsx
import { AuthProvider, useAuth, RequireAuth } from '@k1s0/auth-client';

// アプリのルートで AuthProvider をラップ
function App() {
  const refreshToken = async (token: string) => {
    const response = await fetch('/api/auth/refresh', {
      method: 'POST',
      headers: { Authorization: `Bearer ${token}` },
    });
    return response.json();
  };

  return (
    <AuthProvider config={{ refreshToken, autoRefresh: true }}>
      <Router />
    </AuthProvider>
  );
}

// 認証が必要なページで RequireAuth を使用
function ProtectedPage() {
  return (
    <RequireAuth redirectTo="/login" navigate={navigate}>
      <Dashboard />
    </RequireAuth>
  );
}

// useAuth フックで認証状態を取得
function UserProfile() {
  const { user, logout, isAuthenticated } = useAuth();

  if (!isAuthenticated) return null;

  return (
    <div>
      <p>ようこそ、{user.name} さん</p>
      <button onClick={logout}>ログアウト</button>
    </div>
  );
}

// ロールベースの認可
function AdminPanel() {
  return (
    <RequireRole roles={['admin']} fallback={<AccessDenied />}>
      <AdminDashboard />
    </RequireRole>
  );
}
```

---

## @k1s0/observability

### 目的

フロントエンド向け観測性ライブラリを提供する。OpenTelemetry 統合、構造化ログ、エラートラッキング、パフォーマンス計測を実現。

### モジュール構成

| モジュール | 内容 |
|-----------|------|
| `tracing/` | TracingService, SpanBuilder |
| `logging/` | Logger, ConsoleLogSink, BufferedLogSink |
| `metrics/` | MetricsCollector, Web Vitals |
| `errors/` | ErrorTracker, グローバルエラーハンドリング |
| `provider/` | ObservabilityProvider, useTracing, useLogger, useMetrics, useErrorTracker |
| `utils/` | generateTraceId, generateSpanId, parseTraceparent |

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

```typescript
interface ObservabilityConfig {
  serviceName: string;
  env: string;
  version?: string;
  logLevel?: LogLevel;
  enableTracing?: boolean;
  enableMetrics?: boolean;
  enableErrorTracking?: boolean;
  traceExporter?: TraceExporter;
  logSink?: LogSink;
}

interface LogEntry {
  timestamp: string;
  level: LogLevel;
  message: string;
  service_name: string;
  env: string;
  trace_id?: string;
  span_id?: string;
  request_id?: string;
  [key: string]: unknown;
}

interface SpanInfo {
  traceId: string;
  spanId: string;
  parentSpanId?: string;
  name: string;
  startTime: number;
  endTime?: number;
  status: SpanStatus;
  attributes: Record<string, unknown>;
}
```

### 使用例

```tsx
import {
  ObservabilityProvider,
  useLogger,
  useTracing,
  useSpan,
} from '@k1s0/observability';

// アプリのルートで ObservabilityProvider をラップ
function App() {
  return (
    <ObservabilityProvider
      config={{
        serviceName: 'my-frontend',
        env: 'dev',
        enableTracing: true,
        enableErrorTracking: true,
      }}
    >
      <Router />
    </ObservabilityProvider>
  );
}

// useLogger フックでログ出力
function UserActions() {
  const logger = useLogger();

  const handleClick = () => {
    logger.info('ボタンがクリックされました', { buttonId: 'submit' });
  };

  return <button onClick={handleClick}>送信</button>;
}

// useSpan フックでスパン計測
function DataFetcher() {
  const { startSpan, endSpan } = useTracing();

  const fetchData = async () => {
    const span = startSpan('fetch-user-data');
    try {
      const data = await fetch('/api/users');
      endSpan(span.spanId, 'ok');
      return data;
    } catch (error) {
      endSpan(span.spanId, 'error', { error: error.message });
      throw error;
    }
  };

  return <button onClick={fetchData}>データ取得</button>;
}

// Web Vitals の自動収集
function PerformanceMonitor() {
  const metrics = useMetrics();

  useEffect(() => {
    const listener = (metric) => {
      console.log('Web Vital:', metric);
    };
    metrics.addListener(listener);
    return () => metrics.removeListener(listener);
  }, [metrics]);

  return null;
}
```
