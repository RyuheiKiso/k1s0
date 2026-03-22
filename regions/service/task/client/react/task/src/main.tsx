import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './app/App';

// アプリケーションのエントリポイント: StrictModeでReactの潜在的問題を検出
createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>
);
