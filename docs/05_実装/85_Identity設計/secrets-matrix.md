# Secrets マトリクス（k1s0 全 secret 種別 × 環境）

本ファイルは k1s0 で扱う **すべての secret** の保管場所・参照経路・ローテーション周期を環境別（dev / CI / 採用側 prod）に集約する正典である。`02_secret_management` 章配下の他ファイルが本マトリクスを参照する。

## 関連設計

- [ADR-SEC-001（Keycloak）](../../02_構想設計/adr/ADR-SEC-001-keycloak.md)
- [ADR-SEC-002（OpenBao）](../../02_構想設計/adr/ADR-SEC-002-openbao.md)
- [ADR-SEC-003（SPIRE / SPIFFE）](../../02_構想設計/adr/ADR-SEC-003-spiffe-spire.md)
- plan/02_開発環境整備/08_secret_management.md
- [.env.example](../../../.env.example) — 開発者ローカル用雛形

## 設計原則

1. **本番 secret はリポジトリに置かない**。dev fixture と本番値を物理的に分離。
2. **OIDC keyless を最優先**。鍵管理を最小化（cosign / GHCR push）。
3. **OpenBao は採用側 prod の中枢**。dev は dev mode（root token 固定）、CI は GH Actions secrets、prod は OpenBao KV v2。
4. **gitleaks で誤コミットを止める**。本表で定義した keyword を `.gitleaks.toml` に追加。
5. **SOPS は環境別 YAML の暗号化用**。Argo CD / Helm values の secret セクションに限定使用。

## マトリクス

各 secret 種別ごとに「保管場所」「参照経路」「ローテーション周期」「漏洩時の影響」「責任者」を列挙する。

### 1. データベース接続情報

| 環境 | 保管場所 | 参照経路 | ローテーション | 漏洩時影響 |
|---|---|---|---|---|
| dev (k3d) | OpenBao dev mode (`secret/k1s0/dev/db/*`) | tier1 facade が `Secret.Get` API 経由で取得 | 不要（dev fixture） | 低（dev データのみ） |
| CI | encrypted secrets（`PG_TEST_PASSWORD` 等）+ Testcontainers の ephemeral instance | env 経由（test only） | 不要（ephemeral） | 低（test 限定） |
| 採用側 prod | OpenBao KV v2 (`secret/k1s0/prod/db/{tier}/{env}`) | tier1 facade Pod が ServiceAccount + SPIFFE 経由で OpenBao 認証 → Secret API | 90 日（自動ローテ可） | 高（プロダクションデータ） |

### 2. Kafka SASL credentials

| 環境 | 保管場所 | 参照経路 | ローテーション |
|---|---|---|---|
| dev | OpenBao dev mode (`secret/k1s0/dev/kafka/*`) | Strimzi KafkaUser CR が自動払出 | 不要 |
| CI | Strimzi の ephemeral cluster で都度生成 | KafkaUser CR | 不要 |
| 採用側 prod | OpenBao KV v2 + Strimzi KafkaUser CR | Strimzi が User Operator 経由で payload を Kubernetes Secret に | 30 日（KafkaUser の `passwordVolume.spec.passwordLength` で再生成） |

### 3. OIDC client secret（Keycloak）

| 環境 | 保管場所 | 参照経路 | ローテーション |
|---|---|---|---|
| dev | Keycloak realm-import json の固定値（`tools/local-stack/manifests/keycloak/realm-import.json` に dev 固定値） | 環境変数 | 不要 |
| CI | encrypted secrets（`KEYCLOAK_TEST_CLIENT_SECRET`） | env 経由 | 不要 |
| 採用側 prod | OpenBao KV v2 (`secret/k1s0/prod/oidc/clients/*`) + Keycloak Operator | tier1 が Workload Identity（SPIFFE）でアクセス | 180 日 |

### 4. TLS 証明書

| 環境 | 保管場所 | 参照経路 | ローテーション |
|---|---|---|---|
| dev | cert-manager + selfsigned issuer | k8s Secret（cert-manager 自動払出） | 90 日（cert-manager 自動更新、`renewBefore: 720h`） |
| CI | — | （TLS は ephemeral） | — |
| 採用側 prod | cert-manager + Let's Encrypt or 内部 CA | k8s Secret | 90 日（自動更新） |

### 5. cosign 署名鍵

| 環境 | 保管場所 | 参照経路 | ローテーション |
|---|---|---|---|
| dev | dev fixture（`tests/fixtures/signing-keys/dev/cosign.key`） | env 経由 | 不要 |
| CI | **OIDC keyless（Sigstore Fulcio）**、鍵保管なし | `cosign sign --yes`（GitHub OIDC） | 鍵不要（短命証明書を毎回発行） |
| 採用側 prod | **OIDC keyless（Sigstore）** | 同上 | 鍵不要 |

### 6. Container registry token

| 環境 | 保管場所 | 参照経路 | ローテーション |
|---|---|---|---|
| dev | — | local registry（kind 内蔵） | — |
| CI | `secrets.GITHUB_TOKEN`（自動発行、PAT 不要） | docker/login-action | job 内のみ有効（自動失効） |
| 採用側 prod | OpenBao KV v2 + Pod Service Account の image pull secret | imagePullSecrets | 90 日 |

### 7. flag 定義署名検証鍵（flagd / OpenFeature）

| 環境 | 保管場所 | 参照経路 | ローテーション |
|---|---|---|---|
| dev | dev fixture（cosign signed flag json） | flagd ロード時検証 | 不要 |
| CI | OIDC keyless（cosign） | flag 定義の cosign sign / verify | 鍵不要 |
| 採用側 prod | OIDC keyless | 同上 | 鍵不要 |

### 8. SOPS 暗号化対象 YAML

以下の secret セクションは SOPS で暗号化してから git に含める。`age` recipient は `.sops.yaml` で指定。

| 対象ファイル | 暗号化フィールド |
|---|---|
| `deploy/apps/projects/*.yaml` | `repoCreds[].password` 等 |
| `deploy/charts/*/values-prod.yaml` | `secrets:` セクション全体 |
| `deploy/opentofu/environments/*/secrets.auto.tfvars` | 全フィールド |

`age` 鍵の管理:

- 個人 OSS の dev: `~/.config/sops/age/keys.txt`（リポジトリ外）
- CI: encrypted secret（`SOPS_AGE_KEY`）
- 採用側 prod: 採用側組織が管理、k1s0 リポジトリには含めない

### 9. GH Actions secrets キー一覧（リリース時点 想定）

`.github/repo-settings.md` と整合する形で、現リリース時点で必要な secrets キー名のみを列挙する（実値は別経路）。

| キー | スコープ | 用途 | リリース時点必要性 |
|---|---|---|---|
| `RENOVATE_TOKEN` | repository | Renovate self-hosted 用 PAT | リリース時点 SHOULD（Mend Cloud 不採用） |
| `NUGET_API_KEY` | environment: release | NuGet 公開 | リリース時点 MUST |
| `NPM_TOKEN` | environment: release | npm 公開 | リリース時点 MUST |
| `CRATES_IO_TOKEN` | environment: release | crates.io 公開 | リリース時点 MUST |
| `SOPS_AGE_KEY` | environment: release | SOPS 復号鍵（CI で envoy values 暗号解除に使用） | リリース時点 SHOULD |

`COSIGN_*` / `GHCR_TOKEN` は OIDC で代替するため secret 不要。

## 漏洩時の対応

漏洩を検知した secret 種別ごとに、以下の手順を `ops/runbooks/secret-rotation.md` に展開する。

| 種別 | 即時対応 | 推奨 SLA |
|---|---|---|
| DB password | OpenBao で新値に rotate → tier1 を rolling restart | 1 時間 |
| Kafka SASL | KafkaUser の Secret 削除 → User Operator が再払出 | 1 時間 |
| OIDC client secret | Keycloak で client secret regenerate | 4 時間 |
| TLS 証明書 | cert-manager Certificate を delete → 再発行 | 即時 |
| cosign 鍵（dev fixture） | dev fixture を新世代に置換、過去 sign は invalid 化 | 24 時間 |
| GHCR PAT（誤発行時） | PAT を revoke、image pull secret を更新 | 即時 |

## 関連

- [.env.example](../../../.env.example) — 開発者ローカル `.env` の雛形
- [.gitleaks.toml](../../../.gitleaks.toml) — 漏洩検出 rule
- [ops/runbooks/secret-rotation.md](../../../ops/runbooks/secret-rotation.md) — rotation 手順書
- [`infra/security/openbao/policies/`](../../../infra/security/openbao/policies/) — OpenBao policy 雛形
