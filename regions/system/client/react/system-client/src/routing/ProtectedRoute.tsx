import type { ReactNode } from 'react';
import { useAuth } from '../auth/useAuth';

/**
 * L-004 監査対応: ロール条件の型定義。
 * - `required`: AND 条件。列挙したロールを全て保持している場合のみアクセスを許可する。
 * - `any`: OR 条件。列挙したロールのうち一つでも保持していればアクセスを許可する（既存の requiredRoles と同等の動作）。
 * 両方を指定した場合は AND 条件と OR 条件の両方を満たす必要がある（積集合）。
 */
export interface RolesCondition {
  required?: string[];
  any?: string[];
}

interface ProtectedRouteProps {
  children: ReactNode;
  fallback: ReactNode;
  /**
   * L-004 監査対応: AND/OR 混合条件をサポートするロール指定。
   * 後方互換性のため string[] 形式（OR 条件）も引き続き受け付ける。
   * - string[]: OR 条件（いずれか一つのロールを保持していればアクセス可）
   * - RolesCondition: { required: AND 条件, any: OR 条件 } を個別に指定可能
   */
  roles?: string[] | RolesCondition;
  /**
   * @deprecated roles プロパティを使用してください。後方互換性のために残してあります。
   * roles が未指定の場合のみ有効です。
   */
  requiredRoles?: string[];
}

/**
 * L-004 監査対応: ロール条件を評価するヘルパー関数。
 * userRoles: ユーザーが保持するロール一覧
 * condition: string[]（OR 条件）または RolesCondition（AND/OR 複合条件）
 */
function evaluateRoleCondition(
  userRoles: string[] | undefined,
  condition: string[] | RolesCondition | undefined,
): boolean {
  // 条件が指定されていない場合は制限なし（常に true）
  if (condition == null) {
    return true;
  }

  // string[] 形式は後方互換 OR 条件として扱う
  if (Array.isArray(condition)) {
    if (condition.length === 0) {
      return true;
    }
    return condition.some((role) => userRoles?.includes(role) === true);
  }

  // RolesCondition 形式: required（AND）と any（OR）を個別評価し積集合を取る
  const { required, any: anyRoles } = condition;

  // AND 条件: required に列挙した全ロールを保持している必要がある
  const passesRequired =
    required == null ||
    required.length === 0 ||
    required.every((role) => userRoles?.includes(role) === true);

  // OR 条件: any に列挙したロールのうち一つでも保持していればよい
  const passesAny =
    anyRoles == null ||
    anyRoles.length === 0 ||
    anyRoles.some((role) => userRoles?.includes(role) === true);

  return passesRequired && passesAny;
}

export function ProtectedRoute({ children, fallback, roles, requiredRoles }: ProtectedRouteProps) {
  // M-009 監査対応: 認証確認中に fallback が一瞬フラッシュする問題を修正する
  // loading 中は null を返してフラッシュを防止する
  const { isAuthenticated, loading, user } = useAuth();

  // セッション確認が完了するまでは何も描画しない（fallback フラッシュ防止）
  if (loading) return null;

  // L-004 監査対応: roles プロパティを優先し、未指定の場合は後方互換のため requiredRoles を使用する
  const effectiveCondition: string[] | RolesCondition | undefined = roles ?? requiredRoles;

  const hasRequiredRole = evaluateRoleCondition(user?.roles, effectiveCondition);

  return isAuthenticated && hasRequiredRole ? <>{children}</> : <>{fallback}</>;
}
