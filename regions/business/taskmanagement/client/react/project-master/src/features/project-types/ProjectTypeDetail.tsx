import { useProjectType } from '../../hooks/useProjectTypes';
import { StatusDefinitionList } from '../status-definitions/StatusDefinitionList';

// プロジェクトタイプ詳細コンポーネントのProps
interface ProjectTypeDetailProps {
  projectTypeId: string;
}

// プロジェクトタイプ詳細コンポーネント: 詳細情報とステータス定義一覧を表示
export function ProjectTypeDetail({ projectTypeId }: ProjectTypeDetailProps) {
  const { data: projectType, isLoading, error } = useProjectType(projectTypeId);

  // ローディング中の表示
  if (isLoading) return <div>読み込み中...</div>;

  // エラー発生時の表示
  if (error) return <div>エラーが発生しました: {(error as Error).message}</div>;

  // プロジェクトタイプが見つからない場合
  if (!projectType) return <div>プロジェクトタイプが見つかりません。</div>;

  return (
    <div>
      {/* ナビゲーションリンク */}
      <p>
        <a href="/project-types">← プロジェクトタイプ一覧に戻る</a>
      </p>

      {/* プロジェクトタイプ詳細情報 */}
      <h1>{projectType.display_name}</h1>
      <dl style={dlStyle}>
        <div style={dlRowStyle}>
          <dt style={dtStyle}>コード</dt>
          <dd style={ddStyle}>{projectType.code}</dd>
        </div>
        <div style={dlRowStyle}>
          <dt style={dtStyle}>説明</dt>
          <dd style={ddStyle}>{projectType.description ?? '-'}</dd>
        </div>
        <div style={dlRowStyle}>
          <dt style={dtStyle}>デフォルトワークフロー</dt>
          <dd style={ddStyle}>{projectType.default_workflow ?? '-'}</dd>
        </div>
        <div style={dlRowStyle}>
          <dt style={dtStyle}>状態</dt>
          <dd style={ddStyle}>{projectType.is_active ? '有効' : '無効'}</dd>
        </div>
        <div style={dlRowStyle}>
          <dt style={dtStyle}>並び順</dt>
          <dd style={ddStyle}>{projectType.sort_order}</dd>
        </div>
        <div style={dlRowStyle}>
          <dt style={dtStyle}>作成者</dt>
          <dd style={ddStyle}>{projectType.created_by}</dd>
        </div>
        <div style={dlRowStyle}>
          <dt style={dtStyle}>作成日時</dt>
          <dd style={ddStyle}>{new Date(projectType.created_at).toLocaleString('ja-JP')}</dd>
        </div>
        <div style={dlRowStyle}>
          <dt style={dtStyle}>更新日時</dt>
          <dd style={ddStyle}>{new Date(projectType.updated_at).toLocaleString('ja-JP')}</dd>
        </div>
      </dl>

      {/* プロジェクトタイプに属するステータス定義一覧 */}
      <div style={{ marginTop: '32px' }}>
        <StatusDefinitionList projectTypeId={projectTypeId} />
      </div>
    </div>
  );
}

// 定義リストのスタイル
const dlStyle: React.CSSProperties = {
  border: '1px solid #eee',
  borderRadius: '4px',
  padding: '16px',
};

// 定義リスト行のスタイル
const dlRowStyle: React.CSSProperties = {
  display: 'flex',
  gap: '16px',
  padding: '8px 0',
  borderBottom: '1px solid #f5f5f5',
};

// 定義リスト用語のスタイル
const dtStyle: React.CSSProperties = {
  fontWeight: 'bold',
  width: '160px',
  flexShrink: 0,
};

// 定義リスト値のスタイル
const ddStyle: React.CSSProperties = {
  margin: 0,
};
