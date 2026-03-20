/* クリムゾン警告スタイル — ペルソナ3風の警告表示 */
export default function ProtectedActionNotice({ loading }: { loading: boolean }) {
  return (
    <p className="mt-5 border border-red-400/25 bg-red-400/10 px-4 py-3 text-sm text-red-100">
      {loading
        ? 'セキュアなオペレーターセッションを確認しています。'
        : '保護されたアクションを実行する前に認証ページでサインインしてください。'}
    </p>
  );
}
