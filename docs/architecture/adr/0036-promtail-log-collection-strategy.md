# ADR-0036: Promtail ログ収集戦略（docker.sock 廃止）

## ステータス

承認済み（2026-03-25）

## コンテキスト

外部技術監査（SC-2/M-5）において、Docker Compose 環境の Promtail が以下の問題を抱えていることが指摘された。

1. **SC-2**: `/var/run/docker.sock` をマウントしている。Docker ソケットはホスト Docker デーモンへの完全なアクセス権を持ち、侵害時にはホスト全体の Docker 操作が可能になる。
2. **M-5**: `promtail-config.yaml` が `/var/log/containers/*.log` をスクレイプ対象に設定しているが、このパスは Linux ネイティブ（Kubernetes ノード）に存在するものであり Docker Desktop on Windows では利用不可。設定ファイルとボリュームマウントにも不整合があった（config: ファイルパス / volume: docker.sock）。

## 決定

Docker Compose 環境での Promtail ログ収集方式を以下に変更する。

- **廃止**: `/var/run/docker.sock:/var/run/docker.sock:ro` マウント
- **採用**: `/var/lib/docker/containers:/var/lib/docker/containers:ro` マウント + ファイルベース収集（`/var/lib/docker/containers/*/*.log`）

## 理由

1. **セキュリティ**: docker.sock を使用しないことでホスト Docker デーモンへのアクセス経路を排除できる。
2. **クロスプラットフォーム対応**: `/var/lib/docker/containers` は Docker Desktop on Windows の Linux VM 内に存在するため、Windows/Mac/Linux 全環境で動作する。
3. **設定一貫性**: promtail-config.yaml のスクレイプ対象とボリュームマウントが一致する。
4. **最小権限**: 読み取り専用でコンテナログのみへのアクセスに限定できる。

## 影響

**ポジティブな影響**:

- docker.sock 経由のホスト侵害リスクを解消
- Windows Docker Desktop 環境で Promtail が正常動作する（M-5 解消）
- promtail-config.yaml とボリューム設定の不整合を解消

**ネガティブな影響・トレードオフ**:

- docker_sd_configs による自動コンテナ検出が使えないため、コンテナ ID ベースのファイルパスからラベルを抽出する必要がある
- コンテナ名などの豊富なメタデータを自動付与するには追加の relabeling 設定が必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A: docker.sock 継続 | セキュリティ設定を強化して docker.sock を維持 | ソケット自体がフルアクセスを提供するため根本的な解決にならない |
| 案 B: Docker Loki ドライバー | `--log-driver=loki` でコンテナから直接 Loki へプッシュ | 全コンテナの docker-compose.yaml に logging 設定が必要で変更コストが高い |
| 案 C: Fluent Bit | Promtail の代わりに Fluent Bit を使用 | 既存の Promtail 設定資産を活かし最小変更で対応できる本案を優先 |

## 参考

- 外部技術監査報告書 SC-2: promtail の docker.sock マウント（ホスト全体への侵害リスク）
- 外部技術監査報告書 M-5: promtail が Windows/Docker Desktop 環境で動作不可
- [Promtail configuration](https://grafana.com/docs/loki/latest/send-data/promtail/configuration/)

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-25 | 初版作成（SC-2/M-5 監査対応） | - |
