# tools/e2e/

k1s0 e2e テストの cluster orchestration スクリプト群。`tests/e2e/` の Go test を走らせる前提となる cluster ライフサイクル（起動 / 状態確認 / 削除）を担う。

## 配置正典

- **設計**: [ADR-TEST-008](../../docs/02_構想設計/adr/ADR-TEST-008-e2e-owner-user-bisection.md)（e2e owner / user 二分構造）
- **実装規約**: [docs/05_実装/30_CI_CD設計/35_e2e_test_design/](../../docs/05_実装/30_CI_CD設計/35_e2e_test_design/)
- **責務分界**: [00_方針/01_owner_user_責務分界.md](../../docs/05_実装/30_CI_CD設計/35_e2e_test_design/00_方針/01_owner_user_責務分界.md)

## ディレクトリ構造

```text
tools/e2e/
├── README.md          # 本ファイル
├── owner/             # owner suite (48GB host 専用、CI 不可、release tag ゲートで代替保証)
│   ├── up.sh          # multipass × 5 + kubeadm 3CP HA + Cilium + Longhorn + MetalLB + フルスタック
│   ├── down.sh        # multipass delete × 5
│   └── check.sh       # 5 VM Running + kubeconfig context + 全 node Ready 確認
├── user/              # user suite (16GB host OK、PR + nightly CI で機械検証)
│   ├── up.sh          # kind + minimum stack (Dapr / tier1 facade / Keycloak / 1 backend)
│   ├── down.sh        # kind delete cluster
│   └── check.sh       # kind cluster + kubeconfig context + node Ready 確認
└── lib/               # owner / user 両 suite 共通 helper
    ├── common.sh      # ログ / repo root / artifact dir / sha256 / kubectl wait helper
    ├── multipass.sh   # multipass VM 操作（owner 専用、user は kind なので不要）
    ├── kubeadm.sh     # kubeadm cluster bootstrap（owner 専用）
    ├── cluster_components.sh  # Cilium / Longhorn / MetalLB install（owner 専用）
    └── artifact.sh    # cluster-info / dmesg 集約 + tar.zst 化 + sha256 計算
```

## tools/local-stack/ との責務分離

| layer | 配置 | 責務 |
|---|---|---|
| 構成 SoT | `tools/local-stack/install/` (helm values / manifests) | ADR-POL-002 正典、11 components の install 内容 |
| orchestration (e2e) | `tools/e2e/{owner,user}/` | cluster ライフサイクル / 起動順序 / artifact 集約 |
| orchestration (dev cluster) | `tools/local-stack/up.sh --role <profile>` | cone profile 別の dev cluster (10 役) |

`tools/local-stack/up.sh` の `--role` 引数空間は cone profile 専用で、e2e cluster orchestration は `tools/e2e/{owner,user}/` に物理分離する。

## 使い方

### owner suite

```bash
# 起動 (host OS の WSL2 native shell から、約 1 時間 30 分)
./tools/e2e/owner/up.sh

# 状態確認
./tools/e2e/owner/check.sh

# 全 8 部位実行 (Makefile target、約 60 分)
make e2e-owner-full

# 部位個別実行
make e2e-owner-platform
make e2e-owner-observability
# ...

# 削除
./tools/e2e/owner/down.sh
```

### user suite

```bash
# 起動 (devcontainer 内可、約 5 分)
./tools/e2e/user/up.sh

# smoke test (PR 5 分以内)
make e2e-user-smoke

# 全 test (nightly 30〜45 分)
make e2e-user-full

# 削除
./tools/e2e/user/down.sh
```

## 関連

- [tests/e2e/](../../tests/e2e/) — Go test 本体
- [src/sdk/{go,rust,dotnet,typescript}/test-fixtures/](../../src/sdk/) — 4 言語 SDK 同梱 fixtures (ADR-TEST-010)
- [tools/release/cut.sh](../release/cut.sh) — release tag ゲート (ADR-TEST-011)
- [tools/local-stack/](../local-stack/) — cluster 構成 SoT (ADR-POL-002)
