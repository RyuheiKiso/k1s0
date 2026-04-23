# 01. Harbor / Trivy / push 設計

本ファイルは構想設計 `02_構想設計/04_CICDと配信/00_CICDパイプライン.md` で確定した 7 段ステージの後段 3 段（scan → push → GitOps 更新）のうち、scan と push を担う Harbor + Trivy + cosign 連携の物理配置を確定する。30 章原則 4（IMP-CI-POL-004: Harbor 門番の Trivy Critical 拒否）と原則 5（IMP-CI-POL-005: cosign keyless 署名）の具体化であり、Harbor プロジェクト構成・Trivy スキャナ運用・CVE 閾値・allowlist 管理・cosign 署名経路を 1 本のパイプラインとして束ねる。

![Harbor / Trivy / cosign パイプライン](img/Harbor_Trivy_cosignパイプライン.svg)

Harbor を「サプライチェーン境界の唯一の門番」として機能させるには、push 側（CI）とスキャン側（Harbor 内蔵 Trivy）と署名側（cosign + Fulcio + Rekor）の 3 層が同じ単一真実源を向く構成が必要である。本節は 3 層の物理配置、robot アカウント権限、quarantine プロジェクトへの隔離動線、allowlist 例外の時限付き承認までを確定する。

Kyverno 側（ADR-CICD-003）の admission verify は `70_リリース設計/` が所管するため、本節は「Harbor に到達するまで」に責務を限定し、CD 側の署名検証はリンクで参照するに留める。

## Harbor インスタンスの物理配置

Harbor 本体は `infra/data/harbor/` に Helm chart 化した形で配置し、バックエンドストレージは Longhorn PV（ReadWriteMany）で揃える。Harbor DB は CloudNativePG（ADR-DATA-001）の専用クラスタ `harbor-db` を使い、Harbor 付属の内部 DB は無効化する（IMP-CI-HAR-040）。

- エンドポイント: `harbor.k1s0.internal`（内部向け単一 FQDN、Envoy Gateway 経由で TLS 終端）
- 公開レジストリは Phase 0 では立てず、外部配布は Phase 1b 以降で再設計
- ストレージバックエンド: Longhorn（3 replica）、容量は Phase 0 で 500GB、Phase 1a で 2TB に拡張
- 認証: Keycloak OIDC（人間は SSO）+ robot アカウント（CI のみ）
- ログ: Harbor → OTel Collector（60 章）→ Loki、監査ログは append-only ストレージに 7 年保管

エンドポイントを単一 FQDN に固定する理由は、Kyverno の `image-source` ポリシー（構想設計 MVP-1b 採用）が `harbor.k1s0.internal/*` 以外のイメージを admission で拒否するためである。別 FQDN を同居させると、その一貫性が崩れて下流の admission 検証が機能しなくなる。

## 5 Harbor project と RBAC

Harbor プロジェクトは tier 境界と一致させ、以下の 5 プロジェクトで運用する（IMP-CI-HAR-041）。プロジェクト粒度は tier 分離の可視性とロール管理のバランスで決める。

- `tier1`: tier1 公開 11 API のコンテナイメージ（t1-state / t1-secret / t1-workflow / t1-decision / t1-audit / t1-pii）
- `tier2`: tier2 ドメインサービスのイメージ（C# / Go の両系）
- `tier3`: tier3 Web BFF / Native ラッパのイメージ
- `infra`: observability / mesh / data / security 系自前ビルドイメージ
- `sdk`: SDK の test harness コンテナ（契約テスト実行用）

`quarantine` は project ではなく「隔離用の別 project」として IMP-CI-HAR-042 で後述する。project 単位の RBAC は次の通り分離する。

- 人間: Keycloak のグループ `k1s0-tier1-dev` / `k1s0-tier2-dev` / ... が自 tier に `Developer`、他 tier に `Guest`
- CI: robot アカウント `robot$<project>-ci` が push / scan 実行権限を持つ（他プロジェクトへの到達不可）
- SRE: グループ `k1s0-sre` が全 project に `Maintainer`（緊急時の削除・復旧権限）
- Security: グループ `k1s0-security` が全 project に `Auditor` + `ProjectAdmin`（allowlist 管理）

robot アカウントは CI 専用とし、有効期限は 90 日、ARC の Kubernetes Secret を 60 日周期で自動ローテする（cert-manager + custom controller による発行、IMP-CI-HAR-043）。

## Trivy スキャン運用と CVE 閾値

Harbor 内蔵の Trivy は push 完了 webhook でスキャンをトリガし、結果を Harbor DB に保存する。スキャン結果は Harbor UI と REST API（`/api/v2.0/projects/<p>/repositories/<r>/artifacts/<d>/scan`）の両方で取得できる。本節の閾値運用は 30 章原則 4 を以下の形で物理化する（IMP-CI-HAR-044）。

- CVSS 9.0+（Critical）: push 後自動で `quarantine/` project に tag 付きで隔離し、元の project からは削除
- CVSS 7.0-8.9（High）: push は成功、ただし pull 時に WARN ヘッダ返却、72 時間以内の解消を要求
- CVSS 4.0-6.9（Medium）: push 成功、週次レポートで可視化
- CVSS 3.9 以下: 情報のみ、backlog へ

自動隔離の実装は Harbor Webhook → Argo Events（Phase 2 で正式化、Phase 0 は KEDA ScaledJob で代替）→ `harbor-quarantine-job` で `harbor/robot$quarantine-ci` が `POST /artifacts` で隔離先にコピー後、元を削除する。quarantine に隔離されたイメージは Kyverno 側で pull 不可とし、Security チームの作業場としてのみ利用する。

Trivy DB の更新は Harbor pod 内で日次 cron（2:00 JST）。エアギャップ環境では Phase 1a から Trivy DB オフラインミラー（`infra/data/trivy-mirror/`）を立て、Harbor が外部に出ない運用に移行する（IMP-CI-HAR-045）。

## allowlist 例外の時限付き承認

CVE Critical の即時解消が技術的に不可能（base image 側の未パッチ等）な場合に備え、allowlist 例外フローを定義する。例外は `deploy/security/trivy-allowlist.yaml` に YAML で記述し、Security（D）承認 + 30 日時限付を必須化する（IMP-CI-HAR-046）。

```yaml
# deploy/security/trivy-allowlist.yaml の断片
- cve: CVE-2025-12345
  image_prefix: harbor.k1s0.internal/tier1/t1-audit
  reason: base image openjdk:21-alpine の未パッチ、upstream 対応待ち
  approved_by: k1s0-security-lead
  approved_at: 2026-04-23
  expires_at: 2026-05-23  # 30 日時限
  tracking_issue: https://github.com/k1s0/k1s0/issues/2341
```

allowlist の PR は Security グループの承認 + CODEOWNERS で `Security-lead` の承認を併せて要求する。`expires_at` 超過は GitHub Actions の日次 cron（`_reusable-allowlist-expiry.yml`）が検出し、該当 PR の自動発行で Security に再判定を促す。再承認が期限内に完了しない場合、allowlist から自動削除され、対象イメージは quarantine に再隔離される。

## cosign keyless 署名と Rekor 記録

Harbor push 成功後、同一 reusable workflow 内で cosign による署名を実行する（30 章原則 5）。鍵ベース署名は使わず、GitHub Actions OIDC → Fulcio → Rekor の keyless 経路に統一する（IMP-CI-HAR-047）。

- 署名対象: Harbor に push したすべてのコンテナイメージ（manifest digest に対する署名）と SBOM 添付
- 署名コマンド: `cosign sign --yes --oidc-issuer=https://token.actions.githubusercontent.com harbor.k1s0.internal/<project>/<repo>@<digest>`
- 透明性ログ: Rekor（public instance）に記録し、URL を PR 本文へ自動コメント
- SBOM: `syft` で生成し、`cosign attest --predicate sbom.spdx.json` で添付
- 検証: Kyverno `ClusterImagePolicy` が Admission 時に cosign verify（70 章所管）

Rekor public instance への依存はエアギャップ運用で問題になるため、Phase 1b で Rekor private instance（`infra/security/rekor/`）の立ち上げを計画に入れる。Phase 0 / 1a は public に記録しつつ、記録 URL を社内エビデンスとして `ops/runbooks/daily/rekor-mirror/` で日次スナップショット取得する。

## Phase 展開と DR 対応

Harbor は k1s0 のサプライチェーン単一点障害になるため、DR / HA 対応を Phase 別に段階導入する（IMP-CI-HAR-048）。

- Phase 0: 単一クラスタ、Longhorn 3 replica、DB は CloudNativePG 3 replica（HA）
- Phase 1a: Harbor replication を DR サイトの Harbor へ自動（`harbor.k1s0-dr.internal`）、RPO 目標 1 時間
- Phase 1b: in-line scanner を sigstore/policy-controller に差し替え検討、SBOM 参照の統一化
- Phase 2: 公開レジストリ（`harbor.k1s0.com`）への公開範囲検討、外部配布の is-it-safe 判定を自動化

DR サイト Harbor への replication は rule として「CVE Critical が含まれないイメージ」のみを対象とし、quarantine へ隔離されたイメージは replicate しない。この条件付け自体も allowlist と連動させ、例外承認されたイメージは replication 対象に戻す運用とする（30 章原則 1 の「CI は Harbor push までで完結」との整合）。

## 観測性との接続

Harbor と Trivy の稼働自体を観測対象として扱う。Harbor の可用性は tier1 公開 API の SLI と同列に計測し、イメージ push の成功率は 60 章 `40_SLO_SLI定義/` で SLI として管理する（IMP-CI-HAR-049）。

- Harbor API 可用性: `sli_harbor_api_success_ratio` を OTel Collector 経由で Mimir に記録、SLO 99.9%
- Trivy スキャン実行 SLI: push から scan 完了までの p99 時間、目標 5 分以内
- 隔離動作 SLI: quarantine 隔離ジョブの成功率、SLO 99.99%（失敗は Kyverno 側 admission で捕捉するが、隔離失敗は即時 Sev2）
- cosign 署名成功率: `sli_cosign_sign_success_ratio`、SLO 99.95%、Rekor 不達は警告

Harbor / Trivy が停止すると CI の push 段が詰まり、結果として Phase 0 の deploy が止まる。障害時は `ops/runbooks/incidents/tier1/harbor-down.md` を Sev1 として発動し、Harbor 暫定 bypass（署名済イメージのみ GitOps 更新を許可する縮退運用）を判断する。この縮退運用の承認は Security（D）+ SRE（B）の共同で、1 営業日以内のタイムボックスを必須化する。

## 手動 push の禁止と例外

人間が手で `docker push` / `crane push` を Harbor に対して実行することを禁止し、すべての push は CI 経由に限定する（IMP-CI-HAR-050）。これは原則 1（CI の責務は Harbor push まで）の反対側の保証であり、「Harbor にイメージがあることと、そのイメージが CI を通過したこと」を同値にする。

- Harbor 側の project RBAC で人間アカウント（Keycloak SSO）には `push` 権限を付与しない
- push 権限を持つのは robot$<project>-ci のみ
- 緊急時の手動 push は Security チーム承認で一時的に人間アカウントへ push 権限を付与、作業完了後に revoke（手順は `ops/runbooks/incidents/harbor-emergency-push.md`）
- 一時付与の記録は Audit ログ（tier1 `t1-audit` Pod）に 7 年保管、四半期レビューで棚卸し

この規律が崩れると、「誰かが手で push した経路不明のイメージ」が Harbor に滞留し、サプライチェーン追跡が不能になる。Phase 0 から厳守し、例外を作らない。

## Retention とガベコレ

Harbor に push され続けるイメージは、適切な retention を設けないと指数的に PV 容量を消費する。Phase 0 の 500GB が 6 ヶ月で満杯になる試算があり、次の retention policy を Phase 0 から有効化する（IMP-CI-HAR-051）。

- `main` ブランチの最新 30 tag: 永久保持
- `main` の 30 tag より古いもの: 90 日保持後削除
- `release/*` タグ: 3 年保持（顧客配布対象のため長期）
- feature branch のイメージ: 14 日保持後削除
- cosign 署名 artifact（`*.sig`）と SBOM: 対応イメージと同じ期間保持
- quarantine project のイメージ: 1 年保持（監査証跡として）、1 年超は Security 承認で削除

retention は Harbor の `GC`（garbage collection）を週次（日曜 3:00 JST）で実行し、`untagged manifest` と `dangling blob` を回収する。GC 実行中は push が 30 秒ほどブロックされる可能性があるため、GC 時間帯は PagerDuty の maintenance window で alerting を抑制する。

## 対応 IMP-CI ID

- IMP-CI-HAR-040: Harbor 物理配置と CloudNativePG バックエンド
- IMP-CI-HAR-041: 5 Harbor project（tier1 / tier2 / tier3 / infra / sdk）と RBAC 分離
- IMP-CI-HAR-042: quarantine プロジェクトへの自動隔離
- IMP-CI-HAR-043: robot アカウントの 60 日自動ローテ
- IMP-CI-HAR-044: CVSS 連動の 4 段階閾値運用
- IMP-CI-HAR-045: Trivy DB の日次更新とオフラインミラー（Phase 1a）
- IMP-CI-HAR-046: allowlist 例外の 30 日時限 + Security 承認
- IMP-CI-HAR-047: cosign keyless 署名と Rekor 記録
- IMP-CI-HAR-048: Harbor DR replication と Phase 展開
- IMP-CI-HAR-049: Harbor / Trivy / cosign の SLI 計測と SLO 定義
- IMP-CI-HAR-050: 手動 push 禁止と緊急時の一時付与手順
- IMP-CI-HAR-051: Retention / GC policy とスナップショット管理

## 対応 ADR / DS-SW-COMP / NFR

- ADR-CICD-003（Kyverno による admission）/ ADR-DATA-001（CloudNativePG）/ ADR-SEC-001（OpenBao + Keycloak OIDC）
- DS-SW-COMP-135（配信系）
- NFR-H-INT-001（Cosign 署名）/ NFR-H-INT-002（SBOM 添付）/ NFR-E-MON-001（特権監査）/ NFR-E-NW-004（イメージソース制限）
- 構想設計 7 段ステージ `02_構想設計/04_CICDと配信/00_CICDパイプライン.md` の scan / push 段を本節で物理化
- 関連節: `80_サプライチェーン設計/`（cosign / SBOM 本体）/ `70_リリース設計/`（Kyverno verify）
