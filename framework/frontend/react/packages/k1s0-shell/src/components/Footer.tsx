import React from "react";
import { Box, Typography, Link, useTheme } from "@mui/material";

/**
 * フッターリンク
 */
export interface FooterLink {
  /** リンクID */
  id: string;
  /** 表示ラベル */
  label: string;
  /** リンク先URL */
  href: string;
  /** 新しいタブで開くか */
  external?: boolean;
}

/**
 * Footer プロパティ
 */
export interface FooterProps {
  /** コピーライトテキスト */
  copyright?: string;
  /** フッターリンク */
  links?: FooterLink[];
  /** 追加要素（左側） */
  leftContent?: React.ReactNode;
  /** 追加要素（右側） */
  rightContent?: React.ReactNode;
  /** サイドバー幅（位置調整用） */
  sidebarWidth?: number;
  /** Boxに適用する追加スタイル */
  sx?: object;
}

/**
 * アプリケーションフッターコンポーネント
 */
export function Footer({
  copyright,
  links = [],
  leftContent,
  rightContent,
  sidebarWidth = 0,
  sx,
}: FooterProps): React.ReactElement {
  const theme = useTheme();
  const currentYear = new Date().getFullYear();

  const defaultCopyright = `${currentYear} k1s0. All rights reserved.`;

  return (
    <Box
      component="footer"
      sx={{
        py: 2,
        px: 3,
        mt: "auto",
        ml: `${sidebarWidth}px`,
        backgroundColor: theme.palette.background.paper,
        borderTop: `1px solid ${theme.palette.divider}`,
        transition: theme.transitions.create("margin", {
          easing: theme.transitions.easing.sharp,
          duration: theme.transitions.duration.leavingScreen,
        }),
        display: "flex",
        flexDirection: { xs: "column", sm: "row" },
        alignItems: { xs: "center", sm: "center" },
        justifyContent: "space-between",
        gap: 2,
        ...sx,
      }}
    >
      {/* Left section */}
      <Box sx={{ display: "flex", alignItems: "center", gap: 2 }}>
        {leftContent}
        <Typography variant="body2" color="text.secondary">
          {copyright ?? defaultCopyright}
        </Typography>
      </Box>

      {/* Center section - Links */}
      {links.length > 0 && (
        <Box sx={{ display: "flex", alignItems: "center", gap: 2 }}>
          {links.map((link, index) => (
            <React.Fragment key={link.id}>
              {index > 0 && (
                <Typography variant="body2" color="text.secondary">
                  |
                </Typography>
              )}
              <Link
                href={link.href}
                target={link.external ? "_blank" : undefined}
                rel={link.external ? "noopener noreferrer" : undefined}
                color="text.secondary"
                underline="hover"
                variant="body2"
              >
                {link.label}
              </Link>
            </React.Fragment>
          ))}
        </Box>
      )}

      {/* Right section */}
      {rightContent && (
        <Box sx={{ display: "flex", alignItems: "center", gap: 2 }}>
          {rightContent}
        </Box>
      )}
    </Box>
  );
}
