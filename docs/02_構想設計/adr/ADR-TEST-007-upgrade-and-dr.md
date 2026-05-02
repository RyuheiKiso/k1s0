# ADR-TEST-007: L9 upgrade を N-2→N→N+1 ローリング、L10 DR を Velero + minio + etcd PITR で構造化する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: 起案者 / 採用検討組織 / SRE / DR 担当（採用初期）

## コンテキスト

ADR-TEST-003 で L9 upgrade / L10 DR を独立層として切り出した。これらは本番運用で「いつか必ず起きる」が、単体テスト・smoke E2E では再現できず、本番で初めて顕在化すると影響が大きい層である。Kubernetes 自体のサポートサイクルは N / N-1 / N-2 の 3 世代維持で、年 3 回の minor リリースで N+1 が増えるたびに本番 cluster の upgrade が要る。upgrade 失敗は「control-plane が起動しない」「kubelet が新 version を拒否」「内部 API が deprecated で互換崩壊」など、平時は隠れているバグが一斉に表面化する種類のリスクである。

DR（Disaster Recovery）は「cluster 全体が壊れる」「etcd データが破損する」「region が落ちる」「PV が消える」「namespace が誤削除される」といった災害を想定し、事前に復旧経路を実走検証する層である。本番に乗ってから「Velero で取った backup が実は復元できなかった」と判明するのは最悪のシナリオで、これを防ぐにはローカルで etcd backup → restore → 整合性検証を毎リリースで回す必要がある。

ADR-INFRA-001 で本番が kubeadm + Cluster API、ADR-CNCF-001 で vanilla K8s + CNCF Conformance を維持と決定済。upgrade / DR の検証も本番と同じ kubeadm 経路で行う必要があり、これは ADR-TEST-004 で multipass kubeadm を L9 / L10 用クラスタとして決定した経緯と整合する。

ツール選定の選択肢:

- **L9 upgrade**: ① kubeadm 公式手順（`kubeadm upgrade plan` / `kubeadm upgrade apply`）② Cluster API の `KubeadmControlPlane` rolling upgrade ③ 手動 `kubectl drain` + `apt upgrade` + 再起動 ④ クラスタ削除 + 再構築
- **L10 backup/restore**: ① Velero（CNCF Incubating、VMware Tanzu 主導、PV / namespace / CRD backup）② Stash + Restic（AppsCode、incremental backup 強い）③ K8up（CNCF Sandbox、Restic ベース）④ 自前 `kubectl get -o yaml` + etcd snapshot
- **L10 etcd PITR**: ① kubeadm 標準の `etcdctl snapshot save` ② 自前の continuous backup（WAL log archive）③ etcd-backup-restore Operator（Gardener 製）

選定では以下を満たす必要がある:

- **本番 fidelity**: ADR-INFRA-001 / ADR-CNCF-001 と整合し、kubeadm 標準経路を逸脱しない
- **CNCF / 標準系列**: 10 年保守の継続性、採用組織のスキル流用性
- **multipass kubeadm 上で動く**: ADR-TEST-004 の L9 / L10 クラスタ実装と整合
- **release blocking** が成立する所要時間（1〜2 時間以内、`make qualify-release` の総時間予算に収まる）
- **integrity verification**: backup / upgrade 後の状態が「正しく復元 / 正しく upgrade されている」を機械的に判定できる

## 決定

### L9 upgrade

**multipass kubeadm cluster（ADR-TEST-004 の L9 環境）で N-2 → N → N+1 のローリング upgrade を `make qualify-release` で必須実行する。**

- **対象 K8s version**: ADR-TEST-005 matrix.yaml の `kubernetes_version` 軸と整合（Phase 0: N-1=1.30 / N=1.31、Phase 3: N-2=1.29 を追加）
- **upgrade 経路**: kubeadm 公式手順
  1. control-plane 1 台目を `kubeadm upgrade plan` → `kubeadm upgrade apply <version>`
  2. control-plane 2/3 台目を `kubeadm upgrade node`
  3. worker ノードを `kubectl drain` → `apt upgrade kubelet kubeadm` → `kubeadm upgrade node` → `kubectl uncordon`
- **assertion**:
  - upgrade 中に既存 Deployment の Pod が継続稼働している（rolling update 中の availability ≥ 99%）
  - control-plane API が常時応答する（`kubectl get nodes` が成功し続ける）
  - upgrade 完了後に `kubectl version` で server version が target に一致
  - upgrade 完了後に既存 CRD / Custom Resource が破壊されていない
  - upgrade 完了後に ADR-TEST-003 L4 standard E2E の代表 1 シナリオ（tenant-onboarding）が成功
- **シナリオ配置**: `tests/e2e/L9_upgrade/scenarios/<n>-to-<n+1>.go` で各 upgrade pair を独立 Go test として実装
- **所要時間**: 1 upgrade pair で約 30〜45 分、Phase 0 で 2 pairs（N-1 → N → N+1）合計 1〜2 時間

### L10 DR

**multipass kubeadm cluster + minio + Velero で DR drill を `make qualify-release` で必須実行する。**

- **DR drill シナリオ**: 4 種を最低セットとして `tests/e2e/L10_dr/scenarios/` に配置
  1. **`namespace-restore.go`**: 業務 namespace（Deployment / Service / ConfigMap / Secret / PVC を含む）を Velero で backup → 削除 → restore、整合性検証
  2. **`etcd-pitr.go`**: 任意時刻 T で etcd snapshot 取得 → T+10 分後に意図的破壊（namespace 削除 / Custom Resource 破壊）→ T snapshot から restore、整合性検証
  3. **`pv-data-restore.go`**: Longhorn PV（CNPG postgres）に書き込み → Velero で backup → PV 削除 → restore、データ整合性検証
  4. **`region-failover.go`**: 3 control-plane の 1 台を強制停止 → リーダー選出が完了 → 新リーダー側で書き込み → 停止ノードを復旧 → 整合性検証（Phase 0 で実装、Phase 3 で multi-region 模擬に拡張）
- **integrity verification**:
  - resource 数（Pod / Service / ConfigMap / Secret / PVC）が backup 取得時と一致
  - PV データの SHA256 hash が一致
  - etcd snapshot からの restore 後、`kubectl diff` で差分ゼロ（破壊された CR が完全復元）
- **artifact 保管**: Velero backup と etcd snapshot は minio（ローカル S3 互換、`tools/local-stack/up.sh --role qualify`）に保存。Phase 3 で実 S3 / GCS / Azure Blob への multi-region replication に拡張
- **所要時間**: 4 シナリオ合計 1〜2 時間

### release blocking

L9 + L10 合計 2〜4 時間が `make qualify-release` の総時間予算（半日〜1 日）に収まる。release tag 切る作業の連続マシン占有は許容（ADR-TEST-001 と整合）。

## 検討した選択肢

### 選択肢 A: kubeadm 公式 upgrade + Velero + minio + etcdctl PITR（採用）

- 概要: L9 は kubeadm 公式 `upgrade plan` / `upgrade apply` / `upgrade node`、L10 は Velero（CNCF Incubating）+ minio S3 互換 + etcdctl snapshot の組み合わせ
- メリット:
  - **本番 fidelity が 100%**: ADR-INFRA-001 / ADR-CNCF-001 と完全整合、本番手順をそのままローカルで実走できる
  - Velero は CNCF Incubating で VMware Tanzu の長期コミットがあり、resource + PV 両方の backup を 1 ツールで網羅
  - minio は ADR-DATA-003 で本番 S3 互換として採用済、ローカルでも同じ実装が使える
  - etcdctl は kubeadm が control-plane に内包する標準 binary、追加依存なし
  - 4 ツール（kubeadm / Velero / minio / etcdctl）すべて kubeadm cluster 上で素直に動き、追加 operator 不要
- デメリット:
  - Velero の incremental backup は full backup ベースで、Stash の Restic ベース incremental に劣る（ローカル DR drill では full backup で十分なため実害なし）
  - kubeadm upgrade は手順が多く、scripting が複雑（multipass VM 内で `apt upgrade` + `kubeadm upgrade` を順次実行する shell script を書く工数が要る）

### 選択肢 B: Cluster API 経由の rolling upgrade + Stash + Restic + 自前 etcd

- 概要: L9 は Cluster API の `KubeadmControlPlane` rolling upgrade、L10 は Stash + Restic + 自前 etcd backup script
- メリット:
  - Cluster API rolling upgrade は ADR-INFRA-001 で本番採用と決定済の経路
  - Stash の Restic ベース backup は incremental に強く、artifact 容量が小さい
- デメリット:
  - **multipass cluster 上で Cluster API を動かすのが複雑**: Cluster API は management cluster（CAPI を動かす別 K8s）が要るため、multipass で 2 つの cluster を立てる必要があり、ADR-TEST-004 の構成（multipass 1 cluster）と整合しない
  - Stash は CNCF プロジェクトではない（AppsCode 商用 Operator のコミュニティ版）。10 年保守の継続性で Velero（CNCF Incubating）に劣る
  - 自前 etcd backup script は kubeadm 標準と乖離、メンテ工数が増える

### 選択肢 C: 手動 upgrade + K8up + S3 + 自前 etcd

- 概要: L9 は `kubectl drain` + `apt upgrade` + 再起動の手作業手順、L10 は K8up（CNCF Sandbox）+ S3 + 自前 etcd
- メリット:
  - K8up は CNCF Sandbox 採択で、Restic ベース backup を CRD（Schedule / Backup）で宣言的に管理
  - S3 直接連携で minio 不要
- デメル:
  - **手動 upgrade は ADR-INFRA-001 の宣言的 cluster lifecycle 思想と矛盾**: `kubeadm upgrade` 経由の本番手順を逸脱し、手作業手順をローカルで取ると本番乖離が出る
  - K8up は CNCF Sandbox（Incubating より下位）で、Velero の成熟度に劣る。VMware Tanzu のような大手商用ベンダーのコミットが無く、長期保守継続性のリスク
  - S3 直接連携は Phase 0 でローカル minio に対し冗長（minio で十分）

### 選択肢 D: DR / upgrade 検証放棄

- 概要: L9 / L10 を実装せず、リリース時点では unit / integration / smoke / standard / conformance / chaos / scale だけで qualify を完結
- メリット:
  - 実装工数ゼロ
  - multipass cluster の起動時間が短縮（L5 conformance だけで済む）
- デメリット:
  - **本番 upgrade で初めて upgrade-blocking バグが顕在化**: K8s version 1.30 から 1.31 への移行で deprecated API / breaking change が事前検出されず、本番 down time の根本原因になる
  - **DR drill 未実施**: 本番で災害が起きてから「Velero backup が復元できない」と判明、RTO 4 時間（NFR-A-CONT-001）が達成不能
  - 採用検討者が「k1s0 は upgrade / DR を検証していない、エンタープライズで採用不可」と判定
  - ADR-TEST-003 で 11 層に分けた決定と矛盾、L9 / L10 が「未来への先送り」化

## 決定理由

選択肢 A（kubeadm 公式 + Velero + minio + etcdctl）を採用する根拠は以下。

- **本番 fidelity の最大化**: ADR-INFRA-001 / ADR-CNCF-001 と完全整合し、ローカル multipass cluster で本番と同じ手順を実走できる。Cluster API 経由（B）は management cluster が要りローカル構成が複雑化、手動 upgrade（C）は本番乖離。Velero + minio はそれぞれ CNCF Incubating + ADR-DATA-003 採用済で、本番と同じ tooling がローカルでも使える
- **CNCF / 標準系列の継続性**: Velero（CNCF Incubating）+ minio（CNCF Sandbox）+ etcdctl（kubeadm 内包）+ kubeadm 公式 upgrade は、すべて Kubernetes 公式 + CNCF 採択系列。Stash（B）は CNCF プロジェクト外で 10 年保守の継続性リスク、K8up（C）は CNCF Sandbox で成熟度不足
- **追加 operator 不要のシンプルさ**: 選択肢 A は multipass cluster の上に Velero helm chart を install するだけで動く。選択肢 B（Cluster API）は management cluster が要り、選択肢 C（K8up）は K8up Operator + S3 連携の追加設定が要る。個人 OSS の運用工数最小化に整合
- **assertion の機械化容易性**: Velero backup の `kubectl get` baseline と restore 後 `kubectl get` の diff が機械的に取れる。etcdctl snapshot は etcd-store の binary diff で integrity 検証可能。選択肢 B / C は CRD ベースの状態管理で diff 取得が複雑
- **Phase 移行への対応性**: 選択肢 A は Phase 3 で minio を実 S3 / GCS / Azure Blob に差し替えるだけで、Velero / etcdctl はそのまま使える。multi-region replication も Velero の `BackupStorageLocation` で実装可能。選択肢 C は K8up の S3 連携を再設定する追加工数
- **退路の確保**: Velero が将来の上流不具合で停滞した場合、CRD ベースの backup spec はそのまま K8up 等に machine-translation 可能（両者とも Restic ベースの spec が類似）。kubeadm upgrade 経路は K8s 本体に内包されているため、上流停滞リスクが事実上ない

## 影響

### ポジティブな影響

- L9 upgrade が本番 kubeadm 手順と 100% 一致するローカル検証となり、本番 upgrade 前に deprecated API / breaking change を事前検出できる
- L10 DR drill が毎 release tag で必須実行され、Velero backup の復元可能性が継続検証される。本番災害時の RTO 4 時間（NFR-A-CONT-001）が机上の数字でなく実走証跡で裏付けられる
- minio がローカル qualify と本番（ADR-DATA-003）の両方で使われ、artifact 保管の SoT が一貫する
- etcdctl PITR が kubeadm 内包の標準 binary で動き、追加依存ゼロでローカル DR drill が完結する
- Phase 3 で minio を実 S3 / GCS / Azure Blob に差し替える際、Velero の `BackupStorageLocation` 変更だけで済む（移行コスト最小）
- region-failover シナリオが Phase 0 で control-plane 強制停止 + リーダー選出として実装され、Phase 3 で multi-region 模擬に拡張される拡張余地が確保される

### ネガティブな影響 / リスク

- L9 upgrade は手順が多く、`tools/qualify/cluster/multipass-upgrade.sh` の shell script 実装に 5〜7 人日の初期工数を要する。kubeadm 公式手順を忠実に scripting する規律が要る
- L10 DR drill の所要時間が 1〜2 時間で、`make qualify-release` の総時間予算（半日〜1 日）の 1/4〜1/2 を消費する。release tag 切る作業の連続マシン占有が長くなる
- Velero の incremental backup は full backup ベースで、artifact 容量が大きい（PV データ込みで 1 backup 1〜10GB）。月次の backup 蓄積で minio storage が圧迫されるため、`tools/qualify/dr/cleanup.sh` で古い backup の自動削除が要る
- multipass VM 内で `apt upgrade kubelet kubeadm` を実行する際、Ubuntu archive のミラー応答時間が変動する。upgrade 全体時間が予測通り 30〜45 分に収まらないリスクがあり、release qualify 全体の time budget を不安定化させる
- region-failover シナリオは Phase 0 では「3 control-plane のうち 1 台を強制停止」という単純化に留まり、本物の multi-region failover（地理的に離れた cluster 間の failover）は Phase 3 で実装。Phase 0 のシナリオが本番 fidelity 不足というリスクは ADR で明示し、`docs/governance/QUALIFY-POLICY.md` で「Phase 0 / Phase 3 の DR 射程の差」を採用検討者向けに記述する
- etcd snapshot からの restore は手順が繊細で、protocol-buffers schema 不整合 / kubeadm version 不整合で失敗するケースがある。`ops/runbooks/RB-OPS-004-etcd-pitr-restore.md` で本番でも使える Runbook 化を Phase 0 で完成させる必要

### 移行・対応事項

- `tools/qualify/cluster/multipass-upgrade.sh` を新設し、kubeadm 公式 upgrade 手順（control-plane 1→2→3 → worker drain/upgrade/uncordon）を冪等 shell script として実装
- `tools/qualify/cluster/velero-install.sh` を新設し、multipass cluster に Velero helm chart + minio backend を install
- `tools/qualify/cluster/minio-up.sh` を新設し、qualify 専用 minio instance を `tools/local-stack/up.sh --role qualify` 経由で起動
- `tests/e2e/L9_upgrade/scenarios/n-1-to-n.go` / `n-to-n+1.go` を新設し、各 upgrade pair の Go test を実装
- `tests/e2e/L10_dr/scenarios/namespace-restore.go` / `etcd-pitr.go` / `pv-data-restore.go` / `region-failover.go` を新設、4 シナリオ最低セットを実装
- `tests/e2e/L9_upgrade/assertions/availability-assert.go` を新設し、upgrade 中の availability ≥ 99% を Prometheus クエリで判定
- `tests/e2e/L10_dr/assertions/integrity-check.go` を新設し、resource diff / PV SHA256 hash / etcd diff の機械的判定を実装
- `tools/qualify/dr/cleanup.sh` を新設し、minio 上の古い Velero backup（30 日超）を自動削除する
- `ops/runbooks/RB-OPS-004-etcd-pitr-restore.md` を新設し、本番でも使える etcd PITR 復旧手順を 8 セクション形式（ADR-OPS-001 準拠）で Runbook 化
- `ops/runbooks/RB-OPS-005-velero-backup-restore.md` を新設し、Velero backup-restore の本番手順を Runbook 化
- `docs/governance/QUALIFY-POLICY.md` に「L9 / L10 の Phase 0 / Phase 3 射程の差」を明記（Phase 0 = 単一 cluster 内 failover / Phase 3 = multi-region failover）
- ADR-DATA-003（minio）の「帰結」セクションに「qualify L10 DR でも minio が backup target として使われる」を追記する relate-back 作業

## 参考資料

- ADR-TEST-001（CI 留保 + qualify portable 設計）— release qualify の総時間予算
- ADR-TEST-003（テストピラミッド L0–L10）— L9 / L10 の責任分界
- ADR-TEST-004（kind + multipass 二層 E2E）— L9 / L10 が multipass kubeadm で動く根拠
- ADR-TEST-005（環境マトリクス）— L9 の K8s version 軸（N-2 / N-1 / N）
- ADR-INFRA-001（Cluster API + kubeadm）— L9 が kubeadm 公式手順に揃う根拠
- ADR-CNCF-001（CNCF Conformance）— L9 / L10 が vanilla K8s 経路を維持する根拠
- ADR-DATA-003（minio）— L10 backup storage の SoT
- ADR-OPS-001（Runbook 標準化）— RB-OPS-004 / RB-OPS-005 の形式根拠
- NFR-A-CONT-001（HA / RTO 4 時間）— L10 DR drill の RTO assertion
- NFR-A-DR-002（RPO / バックアップ）— L10 PV 復元 / etcd PITR の整合
- Velero: velero.io
- kubeadm upgrade: kubernetes.io/docs/tasks/administer-cluster/kubeadm/kubeadm-upgrade/
- etcd snapshot save / restore: etcd.io/docs/v3.5/op-guide/recovery/
- 関連 ADR（採用検討中）: ADR-TEST-008（コンプライアンス）/ ADR-TEST-009（観測性 E2E）
