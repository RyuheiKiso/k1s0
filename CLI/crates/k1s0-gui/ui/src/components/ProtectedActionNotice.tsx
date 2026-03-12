export default function ProtectedActionNotice({ loading }: { loading: boolean }) {
  return (
    <p className="mt-5 rounded-2xl border border-amber-400/25 bg-amber-400/10 px-4 py-3 text-sm text-amber-100">
      {loading
        ? 'セキュアなオペレーターセッションを確認しています。'
        : '保護されたアクションを実行する前に認証ページでサインインしてください。'}
    </p>
  );
}
