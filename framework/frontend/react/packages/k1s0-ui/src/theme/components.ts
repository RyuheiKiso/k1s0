import type { Components, Theme } from '@mui/material/styles';

/**
 * k1s0 共通コンポーネントスタイルオーバーライド
 *
 * MUI コンポーネントのデフォルトスタイルを統一
 */
export const components: Components<Omit<Theme, 'components'>> = {
  // ボタン
  MuiButton: {
    defaultProps: {
      disableElevation: true, // フラットなデザイン
    },
    styleOverrides: {
      root: {
        borderRadius: 8,
        textTransform: 'none',
        fontWeight: 500,
      },
      sizeSmall: {
        padding: '6px 16px',
      },
      sizeMedium: {
        padding: '8px 20px',
      },
      sizeLarge: {
        padding: '12px 24px',
      },
    },
  },

  // テキストフィールド
  MuiTextField: {
    defaultProps: {
      variant: 'outlined',
      size: 'small',
    },
  },

  // アウトラインインプット
  MuiOutlinedInput: {
    styleOverrides: {
      root: {
        borderRadius: 8,
        '&:hover .MuiOutlinedInput-notchedOutline': {
          borderWidth: 2,
        },
        '&.Mui-focused .MuiOutlinedInput-notchedOutline': {
          borderWidth: 2,
        },
      },
    },
  },

  // カード
  MuiCard: {
    defaultProps: {
      elevation: 0,
    },
    styleOverrides: {
      root: {
        borderRadius: 12,
        border: '1px solid',
        borderColor: 'rgba(0, 0, 0, 0.12)',
      },
    },
  },

  // ペーパー
  MuiPaper: {
    styleOverrides: {
      root: {
        borderRadius: 8,
      },
      elevation1: {
        boxShadow: '0px 1px 3px rgba(0, 0, 0, 0.08)',
      },
    },
  },

  // アラート
  MuiAlert: {
    styleOverrides: {
      root: {
        borderRadius: 8,
      },
      standardSuccess: {
        backgroundColor: 'rgba(46, 125, 50, 0.08)',
      },
      standardError: {
        backgroundColor: 'rgba(211, 47, 47, 0.08)',
      },
      standardWarning: {
        backgroundColor: 'rgba(237, 108, 2, 0.08)',
      },
      standardInfo: {
        backgroundColor: 'rgba(2, 136, 209, 0.08)',
      },
    },
  },

  // チップ
  MuiChip: {
    styleOverrides: {
      root: {
        borderRadius: 6,
      },
    },
  },

  // ダイアログ
  MuiDialog: {
    styleOverrides: {
      paper: {
        borderRadius: 12,
      },
    },
  },

  // ダイアログタイトル
  MuiDialogTitle: {
    styleOverrides: {
      root: {
        fontSize: '1.25rem',
        fontWeight: 500,
      },
    },
  },

  // ダイアログアクション
  MuiDialogActions: {
    styleOverrides: {
      root: {
        padding: '16px 24px',
      },
    },
  },

  // テーブル
  MuiTableHead: {
    styleOverrides: {
      root: {
        backgroundColor: 'rgba(0, 0, 0, 0.02)',
      },
    },
  },

  MuiTableCell: {
    styleOverrides: {
      head: {
        fontWeight: 600,
      },
    },
  },

  // タブ
  MuiTab: {
    styleOverrides: {
      root: {
        textTransform: 'none',
        fontWeight: 500,
        minWidth: 120,
      },
    },
  },

  // ツールチップ
  MuiTooltip: {
    styleOverrides: {
      tooltip: {
        borderRadius: 4,
        fontSize: '0.75rem',
      },
    },
  },

  // スナックバー
  MuiSnackbar: {
    defaultProps: {
      anchorOrigin: {
        vertical: 'bottom',
        horizontal: 'right',
      },
    },
  },

  // スケルトン
  MuiSkeleton: {
    styleOverrides: {
      root: {
        borderRadius: 4,
      },
      rounded: {
        borderRadius: 8,
      },
    },
  },

  // リンク
  MuiLink: {
    styleOverrides: {
      root: {
        textDecoration: 'none',
        '&:hover': {
          textDecoration: 'underline',
        },
      },
    },
  },

  // ブレッドクラム
  MuiBreadcrumbs: {
    styleOverrides: {
      root: {
        fontSize: '0.875rem',
      },
    },
  },
};

export type K1s0Components = typeof components;
