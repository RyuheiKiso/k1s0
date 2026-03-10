export default function ProtectedActionNotice({ loading }: { loading: boolean }) {
  return (
    <p className="mt-5 rounded-2xl border border-amber-400/25 bg-amber-400/10 px-4 py-3 text-sm text-amber-100">
      {loading
        ? 'Checking the secure operator session before enabling protected actions.'
        : 'Sign in on the Authentication page before running protected actions.'}
    </p>
  );
}
