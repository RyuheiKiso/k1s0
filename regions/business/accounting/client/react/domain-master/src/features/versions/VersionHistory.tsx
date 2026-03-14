import { useVersions } from '../../hooks/useDomainMaster';

// バージョン履歴コンポーネントのProps
interface VersionHistoryProps {
  categoryCode: string;
  itemCode: string;
}

// バージョン履歴コンポーネント: before_data/after_dataの差分をJSON形式で表示
export function VersionHistory({ categoryCode, itemCode }: VersionHistoryProps) {
  const { data: versions, isLoading, error } = useVersions(categoryCode, itemCode);

  if (isLoading) return <div>読み込み中...</div>;
  if (error) return <div>エラーが発生しました: {(error as Error).message}</div>;

  return (
    <div>
      <h1>バージョン履歴 - {itemCode}</h1>

      {/* ナビゲーションリンク */}
      <p>
        <a href={`/categories/${categoryCode}/items`}>← アイテム一覧に戻る</a>
      </p>

      {/* バージョンが存在しない場合 */}
      {versions?.length === 0 && <p>バージョン履歴がありません。</p>}

      {/* バージョン履歴の一覧表示 */}
      {versions?.map((version) => (
        <div
          key={version.id}
          style={{
            border: '1px solid #ddd',
            borderRadius: '4px',
            padding: '16px',
            marginBottom: '12px',
          }}
        >
          {/* バージョンヘッダー: バージョン番号と変更メタ情報 */}
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              marginBottom: '8px',
            }}
          >
            <strong>バージョン {version.version_number}</strong>
            <span style={{ color: '#666' }}>
              {new Date(version.created_at).toLocaleString('ja-JP')}
            </span>
          </div>

          {/* 変更者と変更理由 */}
          <div style={{ marginBottom: '8px', fontSize: '0.9em', color: '#555' }}>
            <span>変更者: {version.changed_by}</span>
            {version.change_reason && (
              <span style={{ marginLeft: '16px' }}>理由: {version.change_reason}</span>
            )}
          </div>

          {/* before_data/after_data の差分表示 */}
          <div style={{ display: 'flex', gap: '16px' }}>
            {/* 変更前データ */}
            <div style={{ flex: 1 }}>
              <h4 style={{ margin: '0 0 4px 0' }}>変更前</h4>
              <pre style={preStyle}>
                {version.before_data
                  ? JSON.stringify(version.before_data, null, 2)
                  : '(新規作成)'}
              </pre>
            </div>
            {/* 変更後データ */}
            <div style={{ flex: 1 }}>
              <h4 style={{ margin: '0 0 4px 0' }}>変更後</h4>
              <pre style={preStyle}>{JSON.stringify(version.after_data, null, 2)}</pre>
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}

// JSONデータ表示用のpreスタイル
const preStyle: React.CSSProperties = {
  background: '#f5f5f5',
  padding: '8px',
  borderRadius: '4px',
  overflow: 'auto',
  fontSize: '0.85em',
  maxHeight: '300px',
};
