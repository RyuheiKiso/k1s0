# @k1s0/navigation

設定駆動ナビゲーションライブラリ。

`config/{env}.yaml` の `ui.navigation` セクションから routes/menu/flows を読み込み、React Router へ反映する。

## 概要

このパッケージは以下を提供する:

1. **スキーマ定義とバリデーション** - zod によるナビゲーション設定のバリデーション
2. **ConfigRouter** - 設定から React Router のルートを自動生成
3. **MenuBuilder** - 設定からメニューを自動生成（権限制御付き）
4. **FlowController** - ウィザード等のフロー遷移制御
5. **Guards** - 権限/feature flag による表示制御コンポーネント

## インストール

```bash
pnpm add @k1s0/navigation
```

## 基本的な使い方

### 1. 設定ファイル（config/default.yaml）

```yaml
ui:
  navigation:
    version: 1

    routes:
      - path: /
        redirect_to: /home

      - path: /home
        screen_id: home
        title: Home

      - path: /users
        screen_id: users.list
        title: Users
        requires:
          permissions: ["user:read"]

    menu:
      - id: primary
        label: Main
        items:
          - label: Home
            to: /home
            icon: home
          - label: Users
            to: /users
            icon: users
            requires:
              permissions: ["user:read"]
```

### 2. 画面コンポーネントの登録

```tsx
import type { ScreenDefinition } from '@k1s0/navigation';

const screens: ScreenDefinition[] = [
  { id: 'home', component: HomePage },
  { id: 'users.list', component: UsersListPage },
  { id: 'users.detail', component: UserDetailPage },
];
```

### 3. NavigationProvider の設定

```tsx
import { BrowserRouter } from 'react-router-dom';
import {
  NavigationProvider,
  ConfigRouter,
  type NavigationConfig,
} from '@k1s0/navigation';

function App() {
  // 設定をロード（実際は config-service や静的ファイルから取得）
  const config: NavigationConfig = {
    version: 1,
    routes: [...],
    menu: [...],
  };

  // ユーザーの権限とフラグ
  const auth = {
    permissions: ['user:read', 'user:write'],
    flags: ['new_dashboard'],
  };

  return (
    <BrowserRouter>
      <NavigationProvider
        config={config}
        screens={screens}
        auth={auth}
        throwOnValidationError={true}
      >
        <AppShell>
          <ConfigRouter />
        </AppShell>
      </NavigationProvider>
    </BrowserRouter>
  );
}
```

### 4. メニューの表示

```tsx
import { MenuBuilder, useMenuItems } from '@k1s0/navigation';

// カスタムレンダリング
function Sidebar() {
  return (
    <MenuBuilder
      groupId="primary"
      renderItem={({ item, isActive, onClick }) => (
        <ListItemButton selected={isActive} onClick={onClick}>
          <ListItemIcon>{getIcon(item.icon)}</ListItemIcon>
          <ListItemText primary={item.label} />
        </ListItemButton>
      )}
    />
  );
}

// または useMenuItems フックを使用
function SimpleSidebar() {
  const menuItems = useMenuItems('primary');

  return (
    <nav>
      {menuItems.map((item) => (
        <a
          key={item.to}
          href={item.to}
          className={item.isActive ? 'active' : ''}
          onClick={(e) => {
            e.preventDefault();
            item.onClick();
          }}
        >
          {item.label}
        </a>
      ))}
    </nav>
  );
}
```

### 5. フロー（ウィザード）の使用

```tsx
import {
  FlowProvider,
  FlowScreen,
  useFlowTransition,
  useFlowContext,
} from '@k1s0/navigation';

// フローのラッパー
function OnboardingFlow() {
  return (
    <FlowProvider
      flowId="user_onboarding"
      onComplete={(formData) => console.log('Completed:', formData)}
    >
      <FlowScreen />
      <FlowNavigation />
    </FlowProvider>
  );
}

// フローのナビゲーションボタン
function FlowNavigation() {
  const { next, back, submit, canNext, canBack, canSubmit, cancel } =
    useFlowTransition();
  const { currentIndex, totalNodes } = useFlowContext();

  return (
    <div>
      <span>Step {currentIndex + 1} / {totalNodes}</span>
      <button onClick={cancel}>キャンセル</button>
      {canBack && <button onClick={back}>戻る</button>}
      {canNext && <button onClick={next}>次へ</button>}
      {canSubmit && <button onClick={submit}>送信</button>}
    </div>
  );
}
```

### 6. 権限/フラグによる表示制御

```tsx
import {
  PermissionGuard,
  FlagGuard,
  useHasPermission,
} from '@k1s0/navigation';

function AdminSection() {
  return (
    <PermissionGuard
      permissions={['admin:access']}
      fallback={<p>アクセス権限がありません</p>}
    >
      <AdminDashboard />
    </PermissionGuard>
  );
}

function NewFeature() {
  return (
    <FlagGuard flags={['new_feature_enabled']}>
      <ExperimentalComponent />
    </FlagGuard>
  );
}

// フックを使用
function ConditionalButton() {
  const canEdit = useHasPermission('user:write');

  return canEdit ? <EditButton /> : null;
}
```

## API リファレンス

### NavigationProvider

ナビゲーション設定を提供するコンテキストプロバイダ。

| Prop | Type | Required | Description |
|------|------|----------|-------------|
| config | NavigationConfig | Yes | ナビゲーション設定 |
| screens | ScreenDefinition[] | Yes | 画面定義リスト |
| auth | AuthContext | No | 権限・フラグ情報 |
| throwOnValidationError | boolean | No | バリデーションエラー時に例外を投げるか（デフォルト: true） |
| onValidationError | (errors: string[]) => void | No | バリデーションエラー時のコールバック |

### ConfigRouter

設定から React Router のルートを自動生成するコンポーネント。

### MenuBuilder

設定からメニューを生成するコンポーネント。

| Prop | Type | Required | Description |
|------|------|----------|-------------|
| renderItem | (props: MenuItemRenderProps) => ReactNode | Yes | メニュー項目のレンダリング関数 |
| renderGroup | (props: MenuGroupRenderProps) => ReactNode | No | グループのレンダリング関数 |
| groupId | string | No | 表示するグループID |

### FlowProvider

フローの状態管理と遷移制御を提供するコンポーネント。

| Prop | Type | Required | Description |
|------|------|----------|-------------|
| flowId | string | Yes | フローID |
| initialFormData | FlowFormData | No | 初期フォームデータ |
| onComplete | (formData: FlowFormData) => void | No | 完了時コールバック |
| onCancel | () => void | No | キャンセル時コールバック |

## バリデーション

起動時に以下のバリデーションが実行される:

1. **スキーマバリデーション** - 設定の構造が正しいか
2. **整合性チェック** - screen_id が登録されているか、フローの遷移が有効か

バリデーションエラーは `throwOnValidationError: true`（デフォルト）の場合、例外として投げられる。
