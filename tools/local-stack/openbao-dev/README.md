# `tools/local-stack/openbao-dev/` — OpenBao dev server スタンドアロン起動

[ADR-SEC-002](../../../docs/02_構想設計/adr/ADR-SEC-002-openbao.md) で採用した OpenBao を、kind を立てずに **手元の docker daemon 上に dev mode で起動** するための補助スタック。

## ファイル

```
openbao-dev/
├── README.md
├── docker-compose.yml   # openbao-dev (server) + openbao-init (初期 secret 投入)
└── up.sh                # 起動 + .devcontainer/.env.local への root token 書き出し
```

## 利用

```bash
./tools/local-stack/openbao-dev/up.sh
```

実行後、`.devcontainer/.env.local` に dev token が記録される（`.gitignore` 対象）。

```
BAO_ADDR=http://localhost:8200
BAO_TOKEN=dev-root-token
```

## 動作

- `openbao-dev` コンテナ: `bao server -dev` モード起動。Port 8200 を host に bind。
- `openbao-init` コンテナ: `secret/` KV v2 マウントを有効化し、初期 secret を投入:
  - `secret/k1s0/dev/db-password`
  - `secret/k1s0/dev/argocd-admin`

## 本番再現の OpenBao との関係

本ファイルは **kind を起動しない場合の代替経路**。`tools/local-stack/up.sh --role <role>` を使うと kind 上の `manifests/80-openbao/` 配下の Helm chart で OpenBao が起動する。両者は独立しており、同時起動するとポート 8200 が衝突するため、どちらか一方を使う。

## 注意

- `dev-root-token` は固定文字列。**ローカル開発専用**で、本番には決して使わない。
- `restart` 時に secret は失われる（dev mode は揮発）。永続が必要になった時点で本番 manifest 経由（kind/k8s）に切替える。
