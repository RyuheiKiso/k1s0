# 40. 運用ライフサイクル（Runbook 集）

本ディレクトリは k1s0 の**運用フェーズで参照される Runbook 群**を保持する。Runbook は単一開発者期から運用拡大期まで通して同じものを使うことを前提とし、開発者本人がオンコール対応する想定で書く。書式は [`docs/00_format/document_standards.md`](../00_format/document_standards.md) §タイプ C（5 段構成: 検出 / 初動 / 復旧 / 根本原因調査 / 事後処理）に従う。ただし「定常運用手順（起動・停止・バックアップなど）」は incident ではないため、5 段構成の "トラブルシュート" セクションを章末に同梱する形で記述する。

## 配置

```
40_運用ライフサイクル/
├── README.md                              # 本ファイル（索引）
├── 01_ローカル本番再現スタック.md          # kind / Dapr Local の起動・停止・トラブルシュート
└── 02_WSL2_distributionバックアップ.md     # wsl --export / --import を使った退避経路
```

リリース時点では上記 2 件のみで、運用拡大期に応じて以下を順次追加する予定:

- `03_本番デプロイ.md` — Argo CD への昇格・ロールバック手順
- `04_インシデント対応.md` — Severity 別エスカレーション・通報経路
- `05_OpenBao_unseal.md` — OpenBao 本番の sealed 復旧手順
- `06_postmortem/` — ポストモーテム記録（`docs-postmortem` Skill 経由で起票）

## Runbook が「個人運用」段階から書かれる理由

k1s0 はリリース時点では単一開発者期にあるが、Runbook を後付けで書く運用は**インシデント発生時に間に合わない**。ローカル kind が起動しなくなった瞬間に「どこを直せばいいのか」を頭の中で再構築するのは、再現性のないトラブルシュートを生み、原因究明が遅れる。Runbook を**本番運用の前**に書く一次目的は、「次に同じ症状が出た時、思考プロセスを再開しなくて済む」ことにある。

二次目的は **オンコールの引継ぎコスト削減**で、運用拡大期に SRE を採用した時点で本ディレクトリがそのまま onboarding 教材になる。リリース時点 で発見済みのトラブルパターンが Runbook に固着していれば、新規参画者は最初の 1 週間で頻出 5 パターンに到達できる。

## 書式の絶対原則

Runbook は読者がアラート発生時に**焦って読む**ことを前提にする。文章の流暢さより、コマンドが**そのまま貼って動く**ことを優先する。具体的には次の 4 点:

- **コマンドはコピペ可能**: 環境変数・パスは具体値で示し、`<your-cluster>` のようなプレースホルダを最小化する
- **判定閾値を明示**: 「pod が遅い」ではなく「`kubectl get pod` で `STATUS=Pending` が 5 分以上継続」のように観測可能な基準で書く
- **5 段構成を徹底**: 検出シグナル → 初動コマンド → 復旧コマンド → 原因調査 → 事後（postmortem 起票・予防策）
- **依存 Runbook を明示**: 「OpenBao の unseal は別 Runbook を参照」のようにクロスリンクで重複記述を避ける

## アラートからの到達経路

将来 Prometheus / Grafana が接続された段階で、各 Runbook には `runbook_url` メタデータを付与する。Alertmanager から `runbook_url=https://github.com/<org>/k1s0/blob/main/docs/40_運用ライフサイクル/01_ローカル本番再現スタック.md#kind-cluster-が-not-ready` のような深いリンクで直接該当節に飛べる構造にする。深リンクのアンカー（`#kind-cluster-が-not-ready`）は本ディレクトリの md 内見出しに 1 対 1 対応させる。

## 関連

- 文書標準（タイプ C）: [`docs/00_format/document_standards.md`](../00_format/document_standards.md)
- 開発環境設計: [`docs/05_実装/50_開発者体験設計/05_ローカル環境基盤/01_WindowsWSL2環境構成.md`](../05_実装/50_開発者体験設計/05_ローカル環境基盤/01_WindowsWSL2環境構成.md)
- Dev Container 10 役: [`docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md`](../05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md)
- 本番再現スタック: [`tools/local-stack/README.md`](../../tools/local-stack/README.md)
