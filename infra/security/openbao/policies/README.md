# OpenBao policy 雛形

本ディレクトリは k1s0 の OpenBao（Vault 互換、MPL-2.0）に適用する **role 別 policy 雛形** を保持する。採用側組織が prod 環境に展開する際の出発点。

## 配置

```text
infra/security/openbao/policies/
├── README.md           # 本ファイル
├── tier1-facade.hcl    # tier1 facade（Go + Rust core）
├── tier2-service.hcl   # tier2 ドメインサービス（C# / Go）
├── tier3-bff.hcl       # tier3 BFF（Web / Native の中継）
└── ci-runner.hcl       # CI runner（リリース時点+ self-hosted ARC 用）
```

## 設計原則（最小権限）

各 role 独自の secret path のみ read 可能。他 tier の secret には明示 deny。`transit` への直接アクセスは tier1 Crypto API 経由のみ許可（tier1-facade のみ encrypt/decrypt 可能）。

| role | 自 tier read | 他 tier | transit | sys |
|---|---|---|---|---|
| `tier1-facade` | ✅ | ❌ deny | ✅（envelope encryption 用） | ❌ deny |
| `tier2-service` | ✅ | ❌ deny | ❌ deny | ❌ deny |
| `tier3-bff` | ✅ | ❌ deny | ❌ deny | ❌ deny |
| `ci-runner` | secret/k1s0/ci/\* のみ | ❌ deny（prod 含む） | ❌ deny | ❌ deny |

## テンプレート変数（重要: Vault Templated Policies の正規構文ではない）

本ディレクトリの `.hcl` 内に登場する `{{environment}}` / `{{tenant_id}}` は **Vault Templated Policies の正規構文ではない** ことに注意する。

- **本テンプレでの意図**: 採用側が `envsubst` / `sed` で実値に展開し、その結果を `bao policy write` に渡す前段テンプレ。
- **Vault Templated Policies の正規構文**: `{{identity.entity.metadata.tenant_id}}` のように entity metadata 経由で動的解決する形式。

### 採用側での展開手順（envsubst 経由）

```bash
ENV=prod TENANT=acme envsubst < tier1-facade.hcl > /tmp/tier1-facade.expanded.hcl
bao policy write tier1-facade /tmp/tier1-facade.expanded.hcl
```

### 動的解決に切替えたい場合

`{{tenant_id}}` を `{{identity.entity.metadata.tenant_id}}` に書き換え、Vault の Identity Engine で entity metadata に tenant_id を設定する。テナントごとに policy を生成し直す必要がなくなる。詳細は Vault 公式の Templated Policies / Identity Engine ドキュメントを参照。

## 適用方法（採用側 prod 想定）

```bash
# OpenBao 認証
export VAULT_ADDR=https://openbao.your-org.example.com
bao login -method=oidc

# policy 登録
for f in infra/security/openbao/policies/*.hcl; do
    name="$(basename "$f" .hcl)"
    bao policy write "$name" "$f"
done

# Kubernetes auth method の role 紐付け（例: tier1-facade）
bao write auth/kubernetes/role/tier1-facade \
    bound_service_account_names=tier1-facade \
    bound_service_account_namespaces=k1s0-tier1 \
    policies=tier1-facade \
    ttl=1h
```

## dev mode との関係

dev mode（`tools/local-stack/openbao-dev/`）は root token 固定で policy を強制しない。本 policy は **採用側 prod のみで発動**する。dev では tier1-facade が secret/k1s0/dev/\* を root token で読む運用。

## rotation との連動

policy 自体は変更頻度が低い（年次 review 程度）。policy 内の `secret/data/k1s0/{{environment}}/...` 配下の値の rotation は [`ops/runbooks/secret-rotation.md`](../../../../ops/runbooks/secret-rotation.md) を参照。

## 関連

- [docs/05_実装/85_Identity設計/secrets-matrix.md](../../../../docs/05_実装/85_Identity設計/secrets-matrix.md)
- [ADR-SEC-002（OpenBao）](../../../../docs/02_構想設計/adr/ADR-SEC-002-openbao.md)
- [ops/runbooks/secret-rotation.md](../../../../ops/runbooks/secret-rotation.md)
