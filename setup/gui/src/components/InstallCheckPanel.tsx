// インストール確認パネルコンポーネント。ツールのインストール状態を一覧表示する。

// ReactのコアAPIとuseStateフックをインポートする
import React, { useState } from "react";
// MUIのUIコンポーネントをインポートする
import {
  Box,
  Button,
  Typography,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableRow,
  Chip,
  CircularProgress,
} from "@mui/material";
// TauriのバックエンドコマンドをフロントエンドからInvokeするAPIをインポートする
import { invoke } from "@tauri-apps/api/core";

// インストール確認結果の型定義
interface CheckResult {
  // ツール名
  name: string;
  // インストールされているかどうかのフラグ
  installed: boolean;
  // バージョン文字列（未インストールの場合はnull）
  version: string | null;
}

// インストール確認パネルコンポーネントを定義する
const InstallCheckPanel: React.FC = () => {
  // 確認結果の状態を管理するステートを定義する
  const [results, setResults] = useState<CheckResult[]>([]);
  // ローディング状態を管理するステートを定義する
  const [loading, setLoading] = useState(false);

  // インストール確認ボタン押下時のハンドラを定義する
  const handleCheck = async () => {
    // ローディング状態を開始する
    setLoading(true);
    try {
      // TauriバックエンドのCheckコマンドを呼び出して結果を取得する
      const data = await invoke<CheckResult[]>("run_install_check");
      // 取得した結果をステートに保存する
      setResults(data);
    } catch (error) {
      // エラー発生時はコンソールにログを出力する
      console.error("インストール確認に失敗しました:", error);
    } finally {
      // 処理完了後はローディング状態を終了する
      setLoading(false);
    }
  };

  return (
    <Box>
      {/* セクションタイトルを表示する */}
      <Typography variant="h6" gutterBottom>
        インストール確認
      </Typography>
      {/* 確認実行ボタンを表示する */}
      <Button
        variant="contained"
        onClick={handleCheck}
        disabled={loading}
        sx={{ mb: 2 }}
      >
        {/* ローディング中はスピナーを表示し、待機中はボタンテキストを表示する */}
        {loading ? <CircularProgress size={20} /> : "確認を実行する"}
      </Button>
      {/* 結果が存在する場合のみ結果テーブルを表示する */}
      {results.length > 0 && (
        <Table>
          {/* テーブルヘッダーを表示する */}
          <TableHead>
            <TableRow>
              <TableCell>ツール</TableCell>
              <TableCell>状態</TableCell>
              <TableCell>バージョン</TableCell>
            </TableRow>
          </TableHead>
          {/* テーブルボディに各ツールの確認結果を表示する */}
          <TableBody>
            {results.map((r) => (
              <TableRow key={r.name}>
                {/* ツール名を表示する */}
                <TableCell>{r.name}</TableCell>
                {/* インストール状態をChipで色分け表示する */}
                <TableCell>
                  <Chip
                    label={r.installed ? "インストール済み" : "未インストール"}
                    color={r.installed ? "success" : "error"}
                    size="small"
                  />
                </TableCell>
                {/* バージョン情報を表示する（なければダッシュを表示する） */}
                <TableCell>{r.version ?? "—"}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}
    </Box>
  );
};

// InstallCheckPanelコンポーネントをデフォルトエクスポートする
export default InstallCheckPanel;
