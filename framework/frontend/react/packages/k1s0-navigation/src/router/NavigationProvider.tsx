/**
 * NavigationProvider - ナビゲーション設定を提供するコンテキスト
 */

import {
  createContext,
  useContext,
  useMemo,
  type ReactNode,
} from 'react';
import type {
  NavigationConfig,
  NavigationContextValue,
  ScreenDefinition,
  AuthContext,
  RequiresCondition,
} from '../schema/types';
import {
  validateNavigationConfig,
  validateConfigIntegrity,
} from '../schema/navigation.schema';

/** ナビゲーションコンテキスト */
const NavigationContext = createContext<NavigationContextValue | null>(null);

/** NavigationProvider のプロパティ */
export interface NavigationProviderProps {
  /** ナビゲーション設定 */
  config: NavigationConfig;
  /** 画面定義リスト */
  screens: ScreenDefinition[];
  /** 認証コンテキスト（権限・フラグ） */
  auth?: AuthContext;
  /** 子要素 */
  children: ReactNode;
  /** バリデーションエラー時のコールバック */
  onValidationError?: (errors: string[]) => void;
  /** 起動時にバリデーションエラーで例外を投げるか（デフォルト: true） */
  throwOnValidationError?: boolean;
}

/**
 * NavigationProvider コンポーネント
 *
 * ナビゲーション設定と画面レジストリを提供する。
 * 起動時にバリデーションを実行し、不正な設定は検知する。
 */
export function NavigationProvider({
  config,
  screens,
  auth = { permissions: [], flags: [] },
  children,
  onValidationError,
  throwOnValidationError = true,
}: NavigationProviderProps) {
  const value = useMemo<NavigationContextValue>(() => {
    // 画面レジストリを作成
    const screenMap = new Map<string, ScreenDefinition>();
    for (const screen of screens) {
      screenMap.set(screen.id, screen);
    }

    // スキーマバリデーション
    const schemaResult = validateNavigationConfig(config);
    if (!schemaResult.success) {
      onValidationError?.(schemaResult.errors);
      if (throwOnValidationError) {
        throw new Error(
          `ナビゲーション設定が不正です:\n${schemaResult.errors.join('\n')}`
        );
      }
      return {
        config,
        screens: screenMap,
        auth,
        checkRequires: () => false,
        isValid: false,
        errors: schemaResult.errors,
      };
    }

    // 整合性チェック
    const registeredScreenIds = new Set(screens.map((s) => s.id));
    const integrityResult = validateConfigIntegrity(config, registeredScreenIds);
    if (!integrityResult.success) {
      onValidationError?.(integrityResult.errors);
      if (throwOnValidationError) {
        throw new Error(
          `ナビゲーション設定の整合性エラー:\n${integrityResult.errors.join('\n')}`
        );
      }
      return {
        config,
        screens: screenMap,
        auth,
        checkRequires: () => false,
        isValid: false,
        errors: integrityResult.errors,
      };
    }

    // 条件チェック関数
    const checkRequires = (requires?: RequiresCondition): boolean => {
      if (!requires) return true;

      // 権限チェック
      if (requires.permissions && requires.permissions.length > 0) {
        const hasAllPermissions = requires.permissions.every((p) =>
          auth.permissions.includes(p)
        );
        if (!hasAllPermissions) return false;
      }

      // フラグチェック
      if (requires.flags && requires.flags.length > 0) {
        const hasAllFlags = requires.flags.every((f) => auth.flags.includes(f));
        if (!hasAllFlags) return false;
      }

      return true;
    };

    return {
      config,
      screens: screenMap,
      auth,
      checkRequires,
      isValid: true,
      errors: [],
    };
  }, [config, screens, auth, onValidationError, throwOnValidationError]);

  return (
    <NavigationContext.Provider value={value}>
      {children}
    </NavigationContext.Provider>
  );
}

/**
 * ナビゲーションコンテキストを取得するフック
 */
export function useNavigationContext(): NavigationContextValue {
  const context = useContext(NavigationContext);
  if (!context) {
    throw new Error(
      'useNavigationContext は NavigationProvider 内で使用してください'
    );
  }
  return context;
}
