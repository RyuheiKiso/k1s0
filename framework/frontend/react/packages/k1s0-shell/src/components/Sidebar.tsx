import React from "react";
import {
  Drawer,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Divider,
  Box,
  IconButton,
  Tooltip,
  useTheme,
} from "@mui/material";
import ChevronLeftIcon from "@mui/icons-material/ChevronLeft";
import ChevronRightIcon from "@mui/icons-material/ChevronRight";

/**
 * ナビゲーション項目
 */
export interface NavItem {
  /** 項目ID */
  id: string;
  /** 表示ラベル */
  label: string;
  /** アイコン */
  icon?: React.ReactNode;
  /** クリック時のコールバック */
  onClick?: () => void;
  /** リンク先パス */
  href?: string;
  /** 現在選択されているか */
  selected?: boolean;
  /** 無効化 */
  disabled?: boolean;
  /** 区切り線 */
  divider?: boolean;
  /** サブ項目 */
  children?: NavItem[];
}

/**
 * Sidebar プロパティ
 */
export interface SidebarProps {
  /** サイドバーが開いているか */
  open: boolean;
  /** サイドバーの幅 */
  width: number;
  /** 折りたたみ状態 */
  collapsed?: boolean;
  /** 折りたたみボタン表示 */
  showCollapseButton?: boolean;
  /** 折りたたみトグルコールバック */
  onCollapseToggle?: () => void;
  /** モバイルモードか（一時的なDrawer） */
  mobile?: boolean;
  /** 閉じるコールバック（モバイル用） */
  onClose?: () => void;
  /** ナビゲーション項目 */
  items: NavItem[];
  /** ヘッダー要素 */
  header?: React.ReactNode;
  /** フッター要素 */
  footer?: React.ReactNode;
  /** Drawerに適用する追加スタイル */
  sx?: object;
}

/**
 * ナビゲーション項目をレンダリング
 */
function NavItemRenderer({
  item,
  collapsed,
}: {
  item: NavItem;
  collapsed?: boolean;
}): React.ReactElement | null {
  if (item.divider) {
    return <Divider key={item.id} sx={{ my: 1 }} />;
  }

  const content = (
    <ListItem key={item.id} disablePadding sx={{ display: "block" }}>
      <ListItemButton
        onClick={item.onClick}
        href={item.href}
        selected={item.selected}
        disabled={item.disabled}
        sx={{
          minHeight: 48,
          justifyContent: collapsed ? "center" : "initial",
          px: 2.5,
        }}
      >
        {item.icon && (
          <ListItemIcon
            sx={{
              minWidth: 0,
              mr: collapsed ? 0 : 3,
              justifyContent: "center",
            }}
          >
            {item.icon}
          </ListItemIcon>
        )}
        {!collapsed && <ListItemText primary={item.label} />}
      </ListItemButton>
    </ListItem>
  );

  if (collapsed && item.label) {
    return (
      <Tooltip key={item.id} title={item.label} placement="right">
        {content}
      </Tooltip>
    );
  }

  return content;
}

/**
 * サイドバーコンポーネント
 */
export function Sidebar({
  open,
  width,
  collapsed = false,
  showCollapseButton = true,
  onCollapseToggle,
  mobile = false,
  onClose,
  items,
  header,
  footer,
  sx,
}: SidebarProps): React.ReactElement {
  const theme = useTheme();

  const drawerContent = (
    <Box
      sx={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        overflow: "hidden",
      }}
    >
      {/* Header */}
      {header && (
        <Box sx={{ p: 2, flexShrink: 0 }}>
          {header}
        </Box>
      )}

      {/* Collapse button */}
      {showCollapseButton && !mobile && (
        <Box sx={{ display: "flex", justifyContent: collapsed ? "center" : "flex-end", p: 1 }}>
          <IconButton onClick={onCollapseToggle} size="small">
            {collapsed ? <ChevronRightIcon /> : <ChevronLeftIcon />}
          </IconButton>
        </Box>
      )}

      <Divider />

      {/* Navigation items */}
      <Box sx={{ flexGrow: 1, overflow: "auto" }}>
        <List>
          {items.map((item) => (
            <NavItemRenderer key={item.id} item={item} collapsed={collapsed} />
          ))}
        </List>
      </Box>

      {/* Footer */}
      {footer && (
        <>
          <Divider />
          <Box sx={{ p: 2, flexShrink: 0 }}>
            {footer}
          </Box>
        </>
      )}
    </Box>
  );

  // Mobile drawer (temporary)
  if (mobile) {
    return (
      <Drawer
        variant="temporary"
        open={open}
        onClose={onClose}
        ModalProps={{ keepMounted: true }}
        sx={{
          "& .MuiDrawer-paper": {
            boxSizing: "border-box",
            width: width,
          },
          ...sx,
        }}
      >
        {drawerContent}
      </Drawer>
    );
  }

  // Desktop drawer (permanent)
  return (
    <Drawer
      variant="permanent"
      open={open}
      sx={{
        width: width,
        flexShrink: 0,
        "& .MuiDrawer-paper": {
          width: width,
          boxSizing: "border-box",
          transition: theme.transitions.create("width", {
            easing: theme.transitions.easing.sharp,
            duration: theme.transitions.duration.enteringScreen,
          }),
        },
        ...sx,
      }}
    >
      {drawerContent}
    </Drawer>
  );
}
