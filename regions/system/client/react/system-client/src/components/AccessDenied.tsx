// アクセス拒否コンポーネント: 権限不足によるアクセス禁止を明示的に表示する（M-27 監査対応）
// ProtectedRoute の fallback として使用し、ローディングスピナーの永続表示を防ぐ

interface AccessDeniedProps {
  // 表示するメッセージ（省略時はデフォルトメッセージを使用）
  message?: string;
}

// 権限不足時に表示するコンポーネント
// role="alert" により支援技術（スクリーンリーダー）が即座に内容を読み上げる
export function AccessDenied({ message }: AccessDeniedProps) {
  return (
    <div role="alert" aria-live="assertive">
      <h2>アクセスが拒否されました</h2>
      <p>{message ?? 'このページへのアクセス権限がありません。'}</p>
    </div>
  );
}
