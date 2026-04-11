// k1s0 GUIアプリのルートコンポーネント。全体のレイアウトとページ構成を管理する。

// ReactのコアAPIをインポートする
import React from "react";
// MUIのレイアウトコンポーネントをインポートする
import { CssBaseline, Container, Typography, Box } from "@mui/material";
// インストール確認パネルコンポーネントをインポートする
import InstallCheckPanel from "./components/InstallCheckPanel";

// アプリケーションのルートコンポーネントを定義する
const App: React.FC = () => {
  return (
    <>
      {/* MUIのグローバルCSSリセットを適用する */}
      <CssBaseline />
      {/* コンテンツ幅を制限するコンテナを設定する */}
      <Container maxWidth="md">
        {/* ページ上下のマージンを設定するボックスを配置する */}
        <Box sx={{ my: 4 }}>
          {/* アプリのタイトルを表示する */}
          <Typography variant="h4" component="h1" gutterBottom>
            k1s0 セットアップツール
          </Typography>
          {/* インストール確認パネルを表示する */}
          <InstallCheckPanel />
        </Box>
      </Container>
    </>
  );
};

// Appコンポーネントをデフォルトエクスポートする
export default App;
