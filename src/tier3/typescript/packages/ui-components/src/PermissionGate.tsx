// k1s0 tier3 SPA 権限ゲートコンポーネント
// 指定した権限を持つユーザーにのみ子要素を表示する（クライアント側 UI ガード）
// NOTE: セキュリティの実体は tier1/tier2 サーバー側で保証する。UI ガードは表示制御のみ。
// WCAG 2.1 AA: 権限なしの場合は意味ある代替テキストを提供する

// React をインポートする
import React from "react";

// 権限の型（アプリケーション定義の権限名）
export type Permission = string;

// PermissionGate に渡すプロパティ型
export interface PermissionGateProps {
  // 必要な権限名（すべて保持している場合に表示する）
  readonly requiredPermissions: readonly Permission[];
  // ユーザーが現在持っている権限一覧
  readonly userPermissions: readonly Permission[];
  // 権限を持つ場合に表示するコンテンツ
  readonly children: React.ReactNode;
  // 権限がない場合に表示するフォールバック（省略時は何も表示しない）
  readonly fallback?: React.ReactNode;
  // AND 条件（true）か OR 条件（false）か（省略時は AND = 全権限必要）
  readonly requireAll?: boolean;
}

// PermissionGate コンポーネント本体
export function PermissionGate({
  requiredPermissions,
  userPermissions,
  children,
  fallback = null,
  requireAll = true,
}: PermissionGateProps): React.JSX.Element | null {
  // 必要な権限を持っているかどうかを判定する
  const hasPermission = requireAll
    ? // AND 条件: 全ての必要権限を持っているか確認する
      requiredPermissions.every((perm) => userPermissions.includes(perm))
    : // OR 条件: いずれかの必要権限を持っているか確認する
      requiredPermissions.some((perm) => userPermissions.includes(perm));

  // 権限がある場合は子要素を表示する
  if (hasPermission) {
    // 子要素をそのまま描画する（フラグメントで包む）
    return <>{children}</>;
  }

  // 権限がない場合はフォールバックを表示する（デフォルトは null）
  if (fallback !== null) {
    // フォールバックを描画する
    return <>{fallback}</>;
  }

  // 権限なし・フォールバックなしの場合は何も表示しない
  return null;
}
