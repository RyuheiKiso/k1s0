import { useState } from 'react';
import { useBoardColumns } from '../../hooks/useBoardColumns';
import type { BoardColumn } from '../../types/board';
import { BoardColumnCard } from './BoardColumnCard';
import { WipLimitForm } from './WipLimitForm';
import styles from './BoardView.module.css';

// BoardViewのProps
interface BoardViewProps {
  projectId: string;
}

// Kanbanボード表示コンポーネント: カラム一覧をWIPゲージ付きで横並び表示
export function BoardView({ projectId }: BoardViewProps) {
  const { data: columns, isLoading, error } = useBoardColumns(projectId);

  // WIP制限編集対象のカラム（nullの場合はモーダル非表示）
  const [editingColumn, setEditingColumn] = useState<BoardColumn | null>(null);

  // ローディング中の表示
  if (isLoading) return <div>読み込み中...</div>;

  // エラー発生時の表示
  if (error) return <div role="alert">エラーが発生しました: {(error as Error).message}</div>;

  return (
    <main>
      <h1>Kanbanボード</h1>
      <p className={styles.projectLabel}>プロジェクトID: {projectId}</p>

      {/* カラムが空の場合のメッセージ */}
      {columns?.length === 0 && (
        <p>カラムが見つかりませんでした。</p>
      )}

      {/* カラム一覧: 横スクロール対応 */}
      <div className={styles.board} aria-label="Kanbanボード">
        {columns?.map((column) => (
          <BoardColumnCard
            key={`${column.project_id}-${column.status_code}`}
            column={column}
            onEditWipLimit={setEditingColumn}
          />
        ))}
      </div>

      {/* WIP制限編集モーダル: 対象カラムが選択されている場合のみ表示 */}
      {editingColumn && (
        <WipLimitForm
          column={editingColumn}
          onClose={() => setEditingColumn(null)}
        />
      )}
    </main>
  );
}
