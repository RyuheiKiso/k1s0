/**
 * FlagGuard - Feature Flag による表示制御
 */

import type { ReactNode } from 'react';
import { useNavigationContext } from '../router/NavigationProvider';

/** FlagGuard のプロパティ */
export interface FlagGuardProps {
  /** 必要なフラグ（すべて満たす必要がある） */
  flags: string[];
  /** 子要素 */
  children: ReactNode;
  /** フラグがない場合のフォールバック */
  fallback?: ReactNode;
}

/**
 * FlagGuard コンポーネント
 *
 * 指定した feature flag がすべて有効な場合のみ子要素を表示する。
 */
export function FlagGuard({
  flags,
  children,
  fallback = null,
}: FlagGuardProps) {
  const { auth } = useNavigationContext();

  const hasAllFlags = flags.every((f) => auth.flags.includes(f));

  if (!hasAllFlags) {
    return <>{fallback}</>;
  }

  return <>{children}</>;
}

/** フラグが有効か判定するフック */
export function useHasFlag(flag: string): boolean {
  const { auth } = useNavigationContext();
  return auth.flags.includes(flag);
}

/** 複数フラグがすべて有効か判定するフック */
export function useHasAllFlags(flags: string[]): boolean {
  const { auth } = useNavigationContext();
  return flags.every((f) => auth.flags.includes(f));
}

/** 複数フラグのいずれかが有効か判定するフック */
export function useHasAnyFlag(flags: string[]): boolean {
  const { auth } = useNavigationContext();
  return flags.some((f) => auth.flags.includes(f));
}
