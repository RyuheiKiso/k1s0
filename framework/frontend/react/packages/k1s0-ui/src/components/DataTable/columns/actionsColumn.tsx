/**
 * アクションカラムヘルパー
 */

import React from 'react';
import IconButton from '@mui/material/IconButton';
import Tooltip from '@mui/material/Tooltip';
import Box from '@mui/material/Box';
import EditIcon from '@mui/icons-material/Edit';
import DeleteIcon from '@mui/icons-material/Delete';
import type { GridRenderCellParams } from '@mui/x-data-grid';
import type { K1s0Column, ActionsColumnOptions, ActionColumnAction } from '../types.js';

/**
 * アクションカラム（編集/削除ボタン）を作成する
 *
 * @param options - アクションカラムのオプション
 * @returns K1s0Column
 *
 * @example
 * ```tsx
 * const columns = [
 *   // ... 他のカラム
 *   actionsColumn<User>({
 *     onEdit: (row) => navigate(`/users/${row.id}/edit`),
 *     onDelete: (row) => handleDelete(row.id),
 *   }),
 * ];
 * ```
 *
 * @example
 * ```tsx
 * // カスタムアクション
 * actionsColumn<User>({
 *   actions: [
 *     {
 *       name: 'view',
 *       label: '詳細',
 *       icon: <VisibilityIcon />,
 *       onClick: (row) => navigate(`/users/${row.id}`),
 *     },
 *     {
 *       name: 'edit',
 *       label: '編集',
 *       icon: <EditIcon />,
 *       onClick: (row) => navigate(`/users/${row.id}/edit`),
 *       hidden: (row) => !row.canEdit,
 *     },
 *   ],
 * })
 * ```
 */
export function actionsColumn<T extends { id: string | number }>(
  options: ActionsColumnOptions<T>
): K1s0Column<T> {
  const {
    onEdit,
    onDelete,
    actions = [],
    headerName = '',
    width = 120,
  } = options;

  // デフォルトアクションを構築
  const defaultActions: ActionColumnAction<T>[] = [];

  if (onEdit) {
    defaultActions.push({
      name: 'edit',
      label: '編集',
      icon: <EditIcon fontSize="small" />,
      onClick: onEdit,
      color: 'primary',
    });
  }

  if (onDelete) {
    defaultActions.push({
      name: 'delete',
      label: '削除',
      icon: <DeleteIcon fontSize="small" />,
      onClick: onDelete,
      color: 'error',
    });
  }

  const allActions = [...defaultActions, ...actions];

  return {
    field: 'actions' as keyof T & string,
    headerName,
    width,
    sortable: false,
    filterable: false,
    disableColumnMenu: true,
    exportable: false,
    renderCell: (params: GridRenderCellParams) => {
      const row = params.row as T;

      const visibleActions = allActions.filter(
        (action) => !action.hidden?.(row)
      );

      return (
        <Box
          sx={{
            display: 'flex',
            gap: 0.5,
            alignItems: 'center',
            justifyContent: 'center',
            width: '100%',
          }}
        >
          {visibleActions.map((action) => {
            const isDisabled = action.disabled?.(row) ?? false;

            const button = (
              <IconButton
                key={action.name}
                size="small"
                color={action.color ?? 'inherit'}
                onClick={(e) => {
                  e.stopPropagation();
                  action.onClick(row);
                }}
                disabled={isDisabled}
                aria-label={action.label ?? action.name}
              >
                {action.icon}
              </IconButton>
            );

            if (action.label) {
              return (
                <Tooltip key={action.name} title={action.label}>
                  <span>{button}</span>
                </Tooltip>
              );
            }

            return button;
          })}
        </Box>
      );
    },
  };
}
