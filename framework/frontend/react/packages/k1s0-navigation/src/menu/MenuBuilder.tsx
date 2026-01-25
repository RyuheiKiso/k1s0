/**
 * MenuBuilder - 設定からメニューを生成するコンポーネント
 */

import { useMemo, type ReactNode } from 'react';
import { useNavigation, isMenuItemActive } from '../router/useNavigation';
import type { MenuGroupConfig, MenuItemConfig } from '../schema/types';

/** メニュー項目のレンダリングプロパティ */
export interface MenuItemRenderProps {
  /** メニュー項目設定 */
  item: MenuItemConfig;
  /** アクティブ状態 */
  isActive: boolean;
  /** クリックハンドラ */
  onClick: () => void;
}

/** メニューグループのレンダリングプロパティ */
export interface MenuGroupRenderProps {
  /** メニューグループ設定 */
  group: MenuGroupConfig;
  /** 子要素（メニュー項目） */
  children: ReactNode;
}

/** MenuBuilder のプロパティ */
export interface MenuBuilderProps {
  /** メニュー項目のレンダリング関数 */
  renderItem: (props: MenuItemRenderProps) => ReactNode;
  /** メニューグループのレンダリング関数（オプション） */
  renderGroup?: (props: MenuGroupRenderProps) => ReactNode;
  /** グループIDでフィルタ（指定した場合、そのグループのみ表示） */
  groupId?: string;
}

/**
 * MenuBuilder コンポーネント
 *
 * 設定からメニューを生成し、カスタムレンダリングを可能にする。
 * 権限・フラグによる表示制御は自動的に適用される。
 */
export function MenuBuilder({
  renderItem,
  renderGroup,
  groupId,
}: MenuBuilderProps) {
  const { accessibleMenus, currentPath, navigateTo } = useNavigation();

  // グループIDでフィルタ
  const filteredMenus = useMemo(() => {
    if (!groupId) return accessibleMenus;
    return accessibleMenus.filter((g) => g.id === groupId);
  }, [accessibleMenus, groupId]);

  // デフォルトのグループレンダリング
  const defaultRenderGroup = ({ children }: MenuGroupRenderProps) => (
    <>{children}</>
  );

  const groupRenderer = renderGroup || defaultRenderGroup;

  return (
    <>
      {filteredMenus.map((group) => (
        <div key={group.id}>
          {groupRenderer({
            group,
            children: (
              <>
                {group.items.map((item) =>
                  renderItem({
                    item,
                    isActive: isMenuItemActive(item, currentPath),
                    onClick: () => navigateTo(item.to),
                  })
                )}
              </>
            ),
          })}
        </div>
      ))}
    </>
  );
}

/** 単一グループのメニュー項目リストを取得するフック */
export function useMenuItems(groupId?: string) {
  const { accessibleMenus, currentPath, navigateTo } = useNavigation();

  return useMemo(() => {
    const menus = groupId
      ? accessibleMenus.filter((g) => g.id === groupId)
      : accessibleMenus;

    return menus.flatMap((group) =>
      group.items.map((item) => ({
        ...item,
        groupId: group.id,
        groupLabel: group.label,
        isActive: isMenuItemActive(item, currentPath),
        onClick: () => navigateTo(item.to),
      }))
    );
  }, [accessibleMenus, groupId, currentPath, navigateTo]);
}
