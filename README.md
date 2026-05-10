# k1s0

業界 pack 並立 + 19 軸同型構造 + 5 階層論 + L1+ 単一深耕 + 移行コミットメントの哲学に基づき、業務エンジニアが業務不変条件を踏み抜く経路を構造で塞ぎ、ジュニア級 tier3 担当でも事故が起きにくい業務プラットフォーム。

## 1.0.0 出荷スコープ
- 業界 pack: 製造業のみ（業界並立構造は day-1 から有効）
- アプリケーション形態: Web SPA / デスクトップ exe / レガシー .NET Framework 4.6.2+ の 3 形態
- 19 軸全てが完成、`release_gate.lock.yaml` の全 cell green、cosign signed tag が物理 prerequisite

## ドキュメント入口
- [docs/](docs/) — 全フェーズドキュメント
- [docs/01_企画](docs/01_企画/README.md) — 背景 / 価値 / 競合差別化 / 法務 / ターゲット / OSS 公開 / 業界 pack 戦略 / 開発体制 / 用語集
- [docs/02_要件定義](docs/02_要件定義/README.md) — スコープ / 機能要件 / 非機能要件 / 技術選定 / 開発体制 / 制約と前提
- [docs/03_概要設計](docs/03_概要設計/README.md) — アーキ概観 / 軸別設計方針 × 10 軸 / クロスカッティング設計
- [docs/04_詳細設計](docs/04_詳細設計/README.md) — 適合仕様 / 強制機構 / クロス適合仕様 / 運用 UI 開発者体験 / lock.yaml 体系
- [docs/00_format](docs/00_format/) — テンプレート / 規約 / frontmatter schema
- [docs/90_knowledge](docs/90_knowledge/) — 技術学習用 reference

## 至高路線
本企画は CLAUDE.md ポリシー「運用コスト度外視 / 1.0.0 で完璧 / 段階的 release 禁止 / 機能削減なし」を全軸で物理 enforce する至高路線を採る。技術判断で悩んだ際は至高を目指す判断を優先する。

## License
[LICENSE](LICENSE) を参照
