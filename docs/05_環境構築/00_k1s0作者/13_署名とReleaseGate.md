---
id: env.meta.author_signing_release_gate
axis: meta
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.meta.author_ci_local_reproducibility
  - env.meta.author_lock_yaml_generator
covered_by:
  defense_in_depth_layers: [B, E]
  proof_classes: []
---

# 署名と Release Gate

## 一文方針

- `release_gate.lock.yaml` の AND-gate が green になることが git tag 打鍵の物理前提条件であり、cosign による tag 署名が 1.0.0 release の唯一の認定手続きである。

## release_gate.lock.yaml AND-gate の概念

```yaml
release_gate:
  and_gate: green   # 全 98 cell が green になった場合のみ green
  cells:
    - name: meta.docs_lint_green
      status: green  # run_lint.py 全 green
    - name: tier1.transport_bidi
      status: red    # 未達
    ...（全 98 cell）
```

全 98 cell が `status: green` にならない限り `and_gate` は `red` のまま。`and_gate: green` が 1.0.0 git tag の物理前提（cosign sign-blob で署名する前にチェックする）。

## cosign のインストール

```bash
# cosign のインストール（最新版）
curl -LO https://github.com/sigstore/cosign/releases/latest/download/cosign-linux-amd64
chmod +x cosign-linux-amd64
sudo mv cosign-linux-amd64 /usr/local/bin/cosign
cosign version
```

## 鍵ペアの生成

```bash
cosign generate-key-pair
# cosign.key（秘密鍵）と cosign.pub（公開鍵）が生成される
# cosign.key は絶対に commit しない
```

`cosign.key` は `.gitignore` に追加すること（または別の安全なストレージに保管）。

## git tag への署名

```bash
git tag -s v1.0.0 -m "v1.0.0"
cosign sign-blob --key cosign.key <release artifact> --output-signature <artifact>.sig
```

tag 打鍵前に必ず `release_gate.lock.yaml` の `release_gate_status` を確認する。

```bash
python3 -c "
import yaml
with open('tools/lock_yaml_generator/samples/release_gate.lock.yaml') as f:
    data = yaml.safe_load(f)
gate = data.get('release_gate_status', 'unknown')
print('release_gate_status:', gate)
assert gate == 'green', 'release_gate_status is not green. Release blocked.'
"
```

## KEK shamir custodian との責務分離

| 責務 | 担当ロール |
|---|---|
| cosign 署名鍵の保管・使用 | k1s0 作者 |
| KEK shamir M-of-N ceremony | プラットフォーム運営者 |
| production cluster の break-glass | プラットフォーム運営者 |

k1s0 作者は production KEK を持たない。cosign 署名鍵と KEK は別の物理鍵として管理する。

## 検収コマンド

```bash
cosign version  # cosign がインストールされていること
ls cosign.pub 2>/dev/null && echo "公開鍵存在" || echo "未生成（generate-key-pair を実行）"
```

## 関連参照

- [08_lock_yaml生成器手順](08_lock_yaml生成器手順.md)
- [12_CI完全再現](12_CI完全再現.md)
