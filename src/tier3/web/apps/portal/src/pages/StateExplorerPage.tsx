// k1s0 State キーをブラウズする画面。
//
// 使い方: store / key を入力して BFF 経由で State.Get を呼び、結果を表示する。

import { useMemo, useState } from 'react';
import { Button, Card, Spinner } from '@k1s0/ui';
import { createApiClient } from '@k1s0/api-client';
import { loadConfig } from '@k1s0/config';

export function StateExplorerPage() {
  // ApiClient はコンポーネント初回 render 時に lazy 構築する。
  // 旧実装は module top-level で `loadConfig(import.meta.env)` を呼んでいたため
  // VITE_BFF_URL 未設定の vitest 環境で App をロードした瞬間に throw し、
  // 単純な smoke test も走らなかった（リリース時点 で React Context に移行する
  // までの暫定的な分離措置）。
  const apiClient = useMemo(() => {
    const config = loadConfig(import.meta.env);
    return createApiClient({ config });
  }, []);
  const [store, setStore] = useState('postgres');
  const [key, setKey] = useState('user/123');
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  // 取得ボタン押下時に BFF を叩く。
  const handleFetch = async () => {
    setLoading(true);
    setError(null);
    setResult(null);
    try {
      const value = await apiClient.stateGet(store, key);
      setResult(JSON.stringify(value, null, 2));
    } catch (e) {
      const message = e instanceof Error ? e.message : 'unknown error';
      setError(message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Card title="State Explorer">
      <div style={{ display: 'grid', gap: 8, marginBottom: 16 }}>
        <label>
          Store
          <input
            value={store}
            onChange={(e) => setStore(e.target.value)}
            style={{ marginLeft: 8, padding: 4, border: '1px solid #ccc', borderRadius: 4 }}
          />
        </label>
        <label>
          Key
          <input
            value={key}
            onChange={(e) => setKey(e.target.value)}
            style={{ marginLeft: 8, padding: 4, border: '1px solid #ccc', borderRadius: 4 }}
          />
        </label>
      </div>
      <Button onClick={handleFetch} disabled={loading}>
        {loading ? <Spinner size={16} /> : 'Fetch'}
      </Button>
      {error ? (
        <pre style={{ marginTop: 16, color: '#b00', whiteSpace: 'pre-wrap' }}>{error}</pre>
      ) : null}
      {result ? (
        <pre style={{ marginTop: 16, background: '#f4f4f4', padding: 8, borderRadius: 4 }}>
          {result}
        </pre>
      ) : null}
    </Card>
  );
}
