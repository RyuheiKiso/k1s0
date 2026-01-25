/**
 * ナビゲーション設定の型定義
 *
 * config/{env}.yaml の ui.navigation セクションに対応
 */

/** 権限・フラグによる表示/遷移条件 */
export interface RequiresCondition {
  /** 必要な権限（すべて満たす必要がある） */
  permissions?: string[];
  /** 必要な feature flag（すべて満たす必要がある） */
  flags?: string[];
}

/** ルート定義 */
export interface RouteConfig {
  /** URL パス */
  path: string;
  /** リダイレクト先（redirect_to が指定された場合、screen_id は不要） */
  redirect_to?: string;
  /** 画面ID（コード側で登録される Screen と対応） */
  screen_id?: string;
  /** 画面タイトル */
  title?: string;
  /** 表示/遷移条件 */
  requires?: RequiresCondition;
}

/** メニュー項目定義 */
export interface MenuItemConfig {
  /** 表示ラベル */
  label: string;
  /** 遷移先パス */
  to: string;
  /** アイコン名 */
  icon?: string;
  /** 表示条件 */
  requires?: RequiresCondition;
}

/** メニューグループ定義 */
export interface MenuGroupConfig {
  /** メニューグループID */
  id: string;
  /** グループラベル */
  label: string;
  /** メニュー項目 */
  items: MenuItemConfig[];
}

/** フロー遷移定義 */
export interface FlowTransitionConfig {
  /** 遷移元ノードID */
  from: string;
  /** イベント名（next, back, submit 等） */
  event: string;
  /** 遷移先ノードID */
  to: string;
  /** 遷移条件 */
  when?: {
    /** 必要なフォームキー */
    required_form_keys?: string[];
    /** 必要な feature flag */
    flags?: string[];
  };
}

/** フローノード定義 */
export interface FlowNodeConfig {
  /** ノードID（遷移定義の参照名） */
  node_id: string;
  /** 画面ID */
  screen_id: string;
}

/** フロー定義 */
export interface FlowConfig {
  /** フローID */
  id: string;
  /** フロータイトル */
  title: string;
  /** 開始画面 */
  start: {
    screen_id: string;
  };
  /** フロー実行に必要な条件 */
  requires?: RequiresCondition;
  /** フローを構成するノード */
  nodes: FlowNodeConfig[];
  /** 許可遷移 */
  transitions: FlowTransitionConfig[];
  /** エラー時の遷移 */
  on_error?: {
    redirect_to: string;
  };
}

/** ナビゲーション設定全体 */
export interface NavigationConfig {
  /** 設定スキーマバージョン */
  version: number;
  /** ルート定義 */
  routes: RouteConfig[];
  /** メニュー定義 */
  menu: MenuGroupConfig[];
  /** フロー定義 */
  flows?: FlowConfig[];
}

/** 画面レジストリに登録される画面情報 */
export interface ScreenDefinition {
  /** 画面ID */
  id: string;
  /** React コンポーネント */
  component: React.ComponentType;
  /** 画面メタ情報（オプション） */
  meta?: Record<string, unknown>;
}

/** 権限・フラグのコンテキスト */
export interface AuthContext {
  /** ユーザーが持つ権限 */
  permissions: string[];
  /** 有効な feature flag */
  flags: string[];
}

/** ナビゲーションコンテキストの値 */
export interface NavigationContextValue {
  /** ナビゲーション設定 */
  config: NavigationConfig;
  /** 画面レジストリ */
  screens: Map<string, ScreenDefinition>;
  /** 権限・フラグコンテキスト */
  auth: AuthContext;
  /** 条件を満たすか判定 */
  checkRequires: (requires?: RequiresCondition) => boolean;
  /** 設定が有効か */
  isValid: boolean;
  /** バリデーションエラー */
  errors: string[];
}
