/**
 * FormGrid - フォームフィールドのグリッドレイアウト
 */

import React from 'react';
import Grid from '@mui/material/Grid';

interface FormGridProps {
  children: React.ReactNode;
  columns?: 1 | 2 | 3 | 4;
  spacing?: number;
}

export function FormGrid(props: FormGridProps): React.ReactElement {
  const { children, columns = 1, spacing = 2 } = props;

  // カラム数から Grid サイズを計算
  const gridSize = 12 / columns;

  return (
    <Grid container spacing={spacing}>
      {React.Children.map(children, (child, index) => {
        if (!React.isValidElement(child)) return null;

        // 子コンポーネントから gridColumn を取得（あれば）
        const childProps = child.props as { gridColumn?: number };
        const itemSize = childProps.gridColumn
          ? (12 / columns) * childProps.gridColumn
          : gridSize;

        return (
          <Grid key={index} size={{ xs: 12, md: Math.min(itemSize, 12) }}>
            {child}
          </Grid>
        );
      })}
    </Grid>
  );
}
