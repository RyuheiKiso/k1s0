// k1s0 tier3 SPA ロール要求コンポーネント
// 指定したロールを持つユーザーにのみ子要素を表示する（クライアント側 UI ガード）
// NOTE: セキュリティの実体は tier1/tier2 サーバー側の RBAC で保証する。UI ガードは表示制御のみ。
// WCAG 2.1 AA: PermissionGate と同様に代替テキストを提供する

// React をインポートする
import React from "react";

// ロールの型（アプリケーション定義のロール名）
export type Role = string;

// RequireRole に渡すプロパティ型
export interface RequireRoleProps {
  // 必要なロール（いずれかを持っている場合に表示する）
  readonly roles: readonly Role[];
  // ユーザーが現在持っているロール一覧
  readonly userRoles: readonly Role[];
  // ロールを持つ場合に表示するコンテンツ
  readonly children: React.ReactNode;
  // ロールがない場合に表示するフォールバック（省略時は何も表示しない）
  readonly fallback?: React.ReactNode;
}

// RequireRole コンポーネント本体
export function RequireRole({
  roles,
  userRoles,
  children,
  fallback = null,
}: RequireRoleProps): React.JSX.Element | null {
  // ユーザーが必要なロールのいずれかを持っているか確認する（OR 条件）
  const hasRole = roles.some((role) => userRoles.includes(role));

  // ロールを持っている場合は子要素を表示する
  if (hasRole) {
    // 子要素をそのまま描画する
    return <>{children}</>;
  }

  // ロールがない場合はフォールバックを表示する（デフォルトは null）
  if (fallback !== null) {
    // フォールバックを描画する
    return <>{fallback}</>;
  }

  // ロールなし・フォールバックなしの場合は何も表示しない
  return null;
}
