import React from "react";
import {
  AppBar,
  Toolbar,
  IconButton,
  Typography,
  Box,
  Avatar,
  Menu,
  MenuItem,
  Divider,
  useTheme,
} from "@mui/material";
import MenuIcon from "@mui/icons-material/Menu";
import AccountCircleIcon from "@mui/icons-material/AccountCircle";

/**
 * ユーザー情報
 */
export interface UserInfo {
  /** ユーザー名 */
  name: string;
  /** メールアドレス */
  email?: string;
  /** アバター画像URL */
  avatarUrl?: string;
}

/**
 * ヘッダーメニュー項目
 */
export interface HeaderMenuItem {
  /** メニュー項目ID */
  id: string;
  /** 表示ラベル */
  label: string;
  /** クリック時のコールバック */
  onClick: () => void;
  /** 区切り線の前に表示するか */
  dividerBefore?: boolean;
}

/**
 * Header プロパティ
 */
export interface HeaderProps {
  /** アプリケーションタイトル */
  title?: string;
  /** ロゴ要素 */
  logo?: React.ReactNode;
  /** サイドバー幅（位置調整用） */
  sidebarWidth?: number;
  /** メニューボタン表示 */
  showMenuButton?: boolean;
  /** メニューボタンクリック時のコールバック */
  onMenuClick?: () => void;
  /** ユーザー情報 */
  user?: UserInfo;
  /** ユーザーメニュー項目 */
  userMenuItems?: HeaderMenuItem[];
  /** ツールバー右側に表示する要素 */
  actions?: React.ReactNode;
  /** AppBarに適用する追加スタイル */
  sx?: object;
}

/**
 * アプリケーションヘッダーコンポーネント
 */
export function Header({
  title,
  logo,
  sidebarWidth = 0,
  showMenuButton = true,
  onMenuClick,
  user,
  userMenuItems = [],
  actions,
  sx,
}: HeaderProps): React.ReactElement {
  const theme = useTheme();
  const [anchorEl, setAnchorEl] = React.useState<null | HTMLElement>(null);
  const menuOpen = Boolean(anchorEl);

  const handleMenuOpen = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget);
  };

  const handleMenuClose = () => {
    setAnchorEl(null);
  };

  const handleMenuItemClick = (item: HeaderMenuItem) => {
    item.onClick();
    handleMenuClose();
  };

  return (
    <AppBar
      position="fixed"
      sx={{
        width: `calc(100% - ${sidebarWidth}px)`,
        ml: `${sidebarWidth}px`,
        transition: theme.transitions.create(["width", "margin"], {
          easing: theme.transitions.easing.sharp,
          duration: theme.transitions.duration.leavingScreen,
        }),
        ...sx,
      }}
    >
      <Toolbar>
        {showMenuButton && (
          <IconButton
            color="inherit"
            aria-label="toggle sidebar"
            edge="start"
            onClick={onMenuClick}
            sx={{ mr: 2 }}
          >
            <MenuIcon />
          </IconButton>
        )}

        {logo && <Box sx={{ mr: 2, display: "flex", alignItems: "center" }}>{logo}</Box>}

        {title && (
          <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
            {title}
          </Typography>
        )}

        {!title && <Box sx={{ flexGrow: 1 }} />}

        {actions && <Box sx={{ mr: 2 }}>{actions}</Box>}

        {user && (
          <>
            <IconButton
              onClick={handleMenuOpen}
              size="small"
              aria-controls={menuOpen ? "user-menu" : undefined}
              aria-haspopup="true"
              aria-expanded={menuOpen ? "true" : undefined}
            >
              {user.avatarUrl ? (
                <Avatar src={user.avatarUrl} alt={user.name} sx={{ width: 32, height: 32 }} />
              ) : (
                <AccountCircleIcon sx={{ width: 32, height: 32, color: "inherit" }} />
              )}
            </IconButton>
            <Menu
              id="user-menu"
              anchorEl={anchorEl}
              open={menuOpen}
              onClose={handleMenuClose}
              anchorOrigin={{ vertical: "bottom", horizontal: "right" }}
              transformOrigin={{ vertical: "top", horizontal: "right" }}
            >
              <Box sx={{ px: 2, py: 1 }}>
                <Typography variant="subtitle2">{user.name}</Typography>
                {user.email && (
                  <Typography variant="body2" color="text.secondary">
                    {user.email}
                  </Typography>
                )}
              </Box>
              {userMenuItems.length > 0 && <Divider />}
              {userMenuItems.map((item) => (
                <React.Fragment key={item.id}>
                  {item.dividerBefore && <Divider />}
                  <MenuItem onClick={() => handleMenuItemClick(item)}>{item.label}</MenuItem>
                </React.Fragment>
              ))}
            </Menu>
          </>
        )}
      </Toolbar>
    </AppBar>
  );
}
