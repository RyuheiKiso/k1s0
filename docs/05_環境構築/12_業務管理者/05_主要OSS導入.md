---
id: env.overview.business_admin_oss_install
axis: overview
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.overview.business_admin_repository_acquisition
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# 主要 OSS 導入

## 一文方針

- 業務管理者が使う Backstage software catalog（ブラウザ操作）/ tier2 admin API CLI（`k1s0-admin`）/ 監査検索 UI の 3 ツールのアクセス設定を行い、それぞれが正常に動作することを確認する。

## 1. Backstage software catalog（ブラウザ操作）

Backstage は staging / production の両方にデプロイされている。初回アクセス時に SSO 認証を行う。

```
手順:
1. Chrome / Edge を開く
2. https://backstage.staging.example.internal にアクセスする
3. 組織 SSO（SAML / OIDC）でログインする
4. [Catalog] タブが表示されることを確認する
5. [Plugins] > [決定表エディタ] が表示されることを確認する
```

Backstage プラグイン（決定表エディタ）の確認:

```
[Plugins] > [Decision Table Editor] を開き、
テナントの決定表一覧が表示されることを確認する
```

## 2. tier2 admin API CLI（k1s0-admin）のインストール

WSL2 上でのインストール手順。

```bash
# 内部リポジトリからダウンロード（URL は組織設定に依存）
curl -L https://example.internal/k1s0-admin/latest/linux-amd64/k1s0-admin \
  -o /tmp/k1s0-admin
chmod +x /tmp/k1s0-admin
sudo mv /tmp/k1s0-admin /usr/local/bin/k1s0-admin

# 設定ファイルの配置
mkdir -p ~/.config/k1s0-admin
cat > ~/.config/k1s0-admin/config.yaml << 'CONFIG'
api_url: https://api.staging.example.internal
token_env: ADMIN_TOKEN
CONFIG

k1s0-admin --version
```

`k1s0-admin` が利用できない場合は `curl` + `jq` で代替する。

```bash
curl --version
jq --version
```

## 3. 監査検索 UI へのアクセス設定

監査検索 UI は別の URL で提供される。初回アクセス時に SSO 認証を行う。

```
手順:
1. https://audit.staging.example.internal にアクセスする
2. 組織 SSO でログインする
3. [検索] 画面が表示されることを確認する
4. 日付範囲を設定して検索を実行し、結果が返ることを確認する
```

## 検収コマンド

ブラウザ確認は目視。CLI 確認（WSL2 利用時）:

```bash
k1s0-admin --version 2>/dev/null || echo "curl+jq 代替モード"
curl --version | head -1
jq --version
```

## 関連参照

- [04_リポジトリ取得手順](04_リポジトリ取得手順.md)
- [06_開発エディタIDE設定](06_開発エディタIDE設定.md)
- [07_テスト検証環境](07_テスト検証環境.md)
