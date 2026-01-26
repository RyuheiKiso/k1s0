import React from "react";
import { Box, CssBaseline, Toolbar } from "@mui/material";
import { Header, HeaderProps, UserInfo, HeaderMenuItem } from "./Header.js";
import { Sidebar, SidebarProps, NavItem } from "./Sidebar.js";
import { Footer, FooterProps, FooterLink } from "./Footer.js";
import { useResponsiveLayout, ResponsiveLayoutOptions } from "../hooks/useResponsiveLayout.js";

/**
 * AppShell プロパティ
 */
export interface AppShellProps {
  /** 子要素（メインコンテンツ） */
  children: React.ReactNode;

  // Header props
  /** アプリケーションタイトル */
  title?: string;
  /** ロゴ要素 */
  logo?: React.ReactNode;
  /** ユーザー情報 */
  user?: UserInfo;
  /** ユーザーメニュー項目 */
  userMenuItems?: HeaderMenuItem[];
  /** ヘッダーアクション */
  headerActions?: React.ReactNode;
  /** カスタムヘッダープロパティ */
  headerProps?: Partial<HeaderProps>;

  // Sidebar props
  /** ナビゲーション項目 */
  navItems?: NavItem[];
  /** サイドバーヘッダー */
  sidebarHeader?: React.ReactNode;
  /** サイドバーフッター */
  sidebarFooter?: React.ReactNode;
  /** カスタムサイドバープロパティ */
  sidebarProps?: Partial<SidebarProps>;

  // Footer props
  /** コピーライト */
  copyright?: string;
  /** フッターリンク */
  footerLinks?: FooterLink[];
  /** フッター左側コンテンツ */
  footerLeftContent?: React.ReactNode;
  /** フッター右側コンテンツ */
  footerRightContent?: React.ReactNode;
  /** フッターを表示するか */
  showFooter?: boolean;
  /** カスタムフッタープロパティ */
  footerProps?: Partial<FooterProps>;

  // Layout options
  /** レスポンシブレイアウトオプション */
  layoutOptions?: ResponsiveLayoutOptions;

  /** コンテンツエリアのスタイル */
  contentSx?: object;
}

/**
 * アプリケーションシェルコンポーネント
 *
 * ヘッダー、サイドバー、フッターを統合したレイアウトを提供する
 */
export function AppShell({
  children,
  // Header
  title,
  logo,
  user,
  userMenuItems,
  headerActions,
  headerProps,
  // Sidebar
  navItems = [],
  sidebarHeader,
  sidebarFooter,
  sidebarProps,
  // Footer
  copyright,
  footerLinks,
  footerLeftContent,
  footerRightContent,
  showFooter = true,
  footerProps,
  // Layout
  layoutOptions,
  contentSx,
}: AppShellProps): React.ReactElement {
  const layout = useResponsiveLayout(layoutOptions);

  const hasSidebar = navItems.length > 0;
  const effectiveSidebarWidth = hasSidebar ? layout.sidebarWidth : 0;

  return (
    <Box sx={{ display: "flex", minHeight: "100vh" }}>
      <CssBaseline />

      {/* Header */}
      <Header
        title={title}
        logo={logo}
        sidebarWidth={effectiveSidebarWidth}
        showMenuButton={hasSidebar}
        onMenuClick={layout.toggleSidebar}
        user={user}
        userMenuItems={userMenuItems}
        actions={headerActions}
        {...headerProps}
      />

      {/* Sidebar */}
      {hasSidebar && (
        <Sidebar
          open={layout.sidebarOpen}
          width={layout.sidebarWidth}
          collapsed={layout.sidebarCollapsed}
          showCollapseButton={layout.isDesktop}
          onCollapseToggle={layout.toggleSidebarCollapse}
          mobile={layout.isMobile}
          onClose={layout.closeSidebar}
          items={navItems}
          header={sidebarHeader}
          footer={sidebarFooter}
          {...sidebarProps}
        />
      )}

      {/* Main content area */}
      <Box
        component="main"
        sx={{
          flexGrow: 1,
          display: "flex",
          flexDirection: "column",
          minHeight: "100vh",
          ml: layout.isMobile ? 0 : `${effectiveSidebarWidth}px`,
          transition: (theme) =>
            theme.transitions.create("margin", {
              easing: theme.transitions.easing.sharp,
              duration: theme.transitions.duration.leavingScreen,
            }),
        }}
      >
        {/* Toolbar spacer */}
        <Toolbar />

        {/* Content */}
        <Box
          sx={{
            flexGrow: 1,
            p: 3,
            ...contentSx,
          }}
        >
          {children}
        </Box>

        {/* Footer */}
        {showFooter && (
          <Footer
            copyright={copyright}
            links={footerLinks}
            leftContent={footerLeftContent}
            rightContent={footerRightContent}
            sidebarWidth={0} // Already handled by main's margin
            {...footerProps}
          />
        )}
      </Box>
    </Box>
  );
}
