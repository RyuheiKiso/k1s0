import { useState, useEffect, useCallback, useMemo } from "react";
import { useTheme, useMediaQuery } from "@mui/material";

/**
 * ブレークポイント
 */
export type Breakpoint = "xs" | "sm" | "md" | "lg" | "xl";

/**
 * レイアウト状態
 */
export interface LayoutState {
  /** 現在のブレークポイント */
  breakpoint: Breakpoint;
  /** モバイルビューかどうか */
  isMobile: boolean;
  /** タブレットビューかどうか */
  isTablet: boolean;
  /** デスクトップビューかどうか */
  isDesktop: boolean;
  /** サイドバーが開いているかどうか */
  sidebarOpen: boolean;
  /** サイドバーの幅 */
  sidebarWidth: number;
  /** サイドバーが折りたたまれているかどうか */
  sidebarCollapsed: boolean;
}

/**
 * レイアウトアクション
 */
export interface LayoutActions {
  /** サイドバーを開く */
  openSidebar: () => void;
  /** サイドバーを閉じる */
  closeSidebar: () => void;
  /** サイドバーの開閉をトグル */
  toggleSidebar: () => void;
  /** サイドバーを折りたたむ */
  collapseSidebar: () => void;
  /** サイドバーを展開する */
  expandSidebar: () => void;
  /** サイドバーの折りたたみをトグル */
  toggleSidebarCollapse: () => void;
}

/**
 * レスポンシブレイアウト設定
 */
export interface ResponsiveLayoutOptions {
  /** デフォルトでサイドバーを開くか */
  defaultSidebarOpen?: boolean;
  /** サイドバーの通常幅 */
  sidebarWidth?: number;
  /** サイドバーの折りたたみ幅 */
  collapsedSidebarWidth?: number;
  /** モバイルでサイドバーを自動で閉じるか */
  autoCloseMobileOnNavigate?: boolean;
}

const DEFAULT_SIDEBAR_WIDTH = 280;
const COLLAPSED_SIDEBAR_WIDTH = 72;

/**
 * レスポンシブレイアウトを管理するフック
 */
export function useResponsiveLayout(
  options: ResponsiveLayoutOptions = {}
): LayoutState & LayoutActions {
  const {
    defaultSidebarOpen = true,
    sidebarWidth: configuredSidebarWidth = DEFAULT_SIDEBAR_WIDTH,
    collapsedSidebarWidth = COLLAPSED_SIDEBAR_WIDTH,
  } = options;

  const theme = useTheme();

  // Media queries
  const isXs = useMediaQuery(theme.breakpoints.only("xs"));
  const isSm = useMediaQuery(theme.breakpoints.only("sm"));
  const isMd = useMediaQuery(theme.breakpoints.only("md"));
  const isLg = useMediaQuery(theme.breakpoints.only("lg"));
  const isUpMd = useMediaQuery(theme.breakpoints.up("md"));

  // Determine current breakpoint
  const breakpoint: Breakpoint = useMemo(() => {
    if (isXs) return "xs";
    if (isSm) return "sm";
    if (isMd) return "md";
    if (isLg) return "lg";
    return "xl";
  }, [isXs, isSm, isMd, isLg]);

  // Device type
  const isMobile = isXs || isSm;
  const isTablet = isMd;
  const isDesktop = isLg || !isXs && !isSm && !isMd && !isLg;

  // Sidebar state
  const [sidebarOpen, setSidebarOpen] = useState(() => {
    // Mobile starts closed, desktop starts based on default
    return isUpMd ? defaultSidebarOpen : false;
  });

  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);

  // Auto-close sidebar on mobile
  useEffect(() => {
    if (isMobile && sidebarOpen) {
      setSidebarOpen(false);
    }
    if (!isMobile && !sidebarOpen && defaultSidebarOpen) {
      setSidebarOpen(true);
    }
  }, [isMobile, defaultSidebarOpen]);

  // Calculate effective sidebar width
  const sidebarWidth = useMemo(() => {
    if (!sidebarOpen) return 0;
    if (sidebarCollapsed) return collapsedSidebarWidth;
    return configuredSidebarWidth;
  }, [sidebarOpen, sidebarCollapsed, configuredSidebarWidth, collapsedSidebarWidth]);

  // Actions
  const openSidebar = useCallback(() => setSidebarOpen(true), []);
  const closeSidebar = useCallback(() => setSidebarOpen(false), []);
  const toggleSidebar = useCallback(() => setSidebarOpen((prev) => !prev), []);
  const collapseSidebar = useCallback(() => setSidebarCollapsed(true), []);
  const expandSidebar = useCallback(() => setSidebarCollapsed(false), []);
  const toggleSidebarCollapse = useCallback(() => setSidebarCollapsed((prev) => !prev), []);

  return {
    // State
    breakpoint,
    isMobile,
    isTablet,
    isDesktop,
    sidebarOpen,
    sidebarWidth,
    sidebarCollapsed,
    // Actions
    openSidebar,
    closeSidebar,
    toggleSidebar,
    collapseSidebar,
    expandSidebar,
    toggleSidebarCollapse,
  };
}
