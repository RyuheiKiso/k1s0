# ADR-TEST-005: 環境マトリクスを pairwise 抽出で 8〜12 jobs に圧縮し、matrix.yaml で正典化する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: 起案者 / 採用検討組織 / SRE / QA リード

## コンテキスト

ADR-TEST-001 portable 制約 7 で「マトリクス定義を `tests/qualify/matrix.yaml` で外出しする」と決定した。これは Phase 1 で CI に移行する際に GitHub Actions の `strategy.matrix` へ machine-translation できる構造を最初から持つためである。しかし「matrix の中身として何を扱うか」は未定で、軸の選定・代表値・jobs 数の上限を ADR で正典化しないと、起案者の主観でマトリクスが決まり、採用検討者から見て「なぜこの軸でこの値が選ばれたのか」が説明できなくなる。

理論上、k1s0 が動作保証すべき環境軸は以下が候補となる:

- **K8s version**: N-2 / N-1 / N の 3 世代サポート（CNCF / Kubernetes 公式の支援サイクル）
- **CNI**: Cilium（本番、ADR-NET-001）/ Calico / Flannel
- **CSI**: Longhorn（本番、ADR-STOR-001）/ local-path / NFS / cloud disk（EBS / GCE PD / Azure Disk）
- **LB**: MetalLB（本番、ADR-STOR-002）/ cloud-provider-kind / cloud LB（ELB / GLB / Azure LB）
- **IP stack**: IPv4 / IPv6 / dual-stack
- **node OS**: Ubuntu 24.04 / Debian 12 / Bottlerocket / Talos
- **arch**: amd64 / arm64
- **container runtime**: containerd / CRI-O

これらを全部掛け算すると **3 × 3 × 4 × 3 × 3 × 4 × 2 × 2 = 5184 組み合わせ** になり、ローカルでも CI でも回せない。CNCF Graduated 級 OSS（Cilium / Istio / ArgoCD）が実際に運用しているマトリクス jobs 数は 8〜30 程度で、全組み合わせの 0.5〜5% 範囲に圧縮されている。圧縮の手段は実用上 ① 起案者が「重要そうな組み合わせ」を手選定（属人性が爆発）② **pairwise testing**（all-pairs combinatorial testing、すべての軸ペアの組み合わせを 1 回以上カバーする最小 jobs を組合論で算出）③ orthogonal array（より厳密だが軸が多いと不可能）④ 完全 matrix を諦めてリファレンス構成 1 つだけ、の 4 案がある。

加えて、リリース時点（Phase 0）と Phase 3（cloud sponsor 獲得後）で扱える jobs 数の上限が異なる。リリース時点は起案者のローカルマシン 1〜2 台で nightly / release qualify を回すため、jobs 数は **8 jobs 以内** が現実的上限。Phase 3 で sponsor cluster（EKS / GKE / AKS / 専用 hardware）が利用可能になれば 30 jobs 程度まで拡張できる。matrix.yaml は Phase 0 / Phase 3 の両方を表現できる構造でなければならない。

選定では以下を満たす必要がある:

- **軸の網羅性**: 採用組織が変更しうる軸を漏らさず捕捉する（CNI / CSI / LB / OS / arch / IP stack）
- **jobs 数の Phase 別上限**: Phase 0 ≤ 8 jobs / Phase 3 ≤ 30 jobs を構造的に満たす
- **代表値の客観性**: 「なぜ Cilium / Calico / Flannel の 3 つで Cilium 1 つではないか」「なぜ Ubuntu / Debian / Bottlerocket の 3 つで Ubuntu 1 つではないか」を ADR で説明できる
- **pairwise 算出の自動化**: 軸を 1 つ追加するたびに手作業で再計算しない（YAML を更新したら matrix.yaml が自動再生成される）
- **CI 移植容易性**: `tests/qualify/matrix.yaml` がそのまま GitHub Actions `strategy.matrix` に変換可能（ADR-TEST-001 portable 制約 7 と整合）

## 決定

**環境マトリクスは pairwise testing で圧縮し、Phase 0 では 8 jobs 以内、Phase 3 では 30 jobs 以内に収める。** マトリクス軸と代表値、Phase 別の jobs 数を `tests/qualify/matrix.yaml` で宣言的に管理する。

軸と代表値は以下に固定する。

| 軸 | Phase 0 代表値 | Phase 3 で拡張 | 選定根拠 |
|----|---------------|---------------|---------|
| K8s version | N-1 / N | N-2 を追加 | リリース時点で 2 世代、Phase 3 で 3 世代サポート |
| CNI | Cilium / Calico | Flannel を追加 | 本番は Cilium、Calico は採用組織で実績多い代替候補 |
| CSI | Longhorn / local-path | NFS / cloud disk を追加 | 本番は Longhorn、local-path は kind での速度層検証用 |
| LB | MetalLB / cloud-provider-kind | cloud LB を追加 | 本番は MetalLB、cloud-provider-kind は kind で LB 検証 |
| IP stack | IPv4 / dual-stack | IPv6-only を追加 | リリース時点は dual-stack、IPv6-only は将来対応 |
| node OS | Ubuntu 24.04 のみ | Debian 12 / Bottlerocket を追加 | リリース時点は Ubuntu 一本、ADR-DEV-002 のリファレンスと整合 |
| arch | amd64 のみ | arm64 を追加 | リリース時点は amd64 一本、Phase 1 で arm64 補助機検証 |
| container runtime | containerd のみ | CRI-O を追加 | 本番は containerd、CRI-O は Phase 3 で互換性検証 |

**Phase 0 の jobs 数算出**: 上記表から Phase 0 に拡張対象がない軸（OS / arch / runtime）は 1 値に固定、拡張対象がある軸（K8s version 2 / CNI 2 / CSI 2 / LB 2 / IP stack 2）は pairwise で組み合わせる。allpairspy で算出すると 4〜6 jobs に収束し、安全マージン込みで **Phase 0 上限 8 jobs** に収まる。

**Phase 3 の jobs 数算出**: 全軸が 2〜3 値に拡張されるが、orthogonal array ではなく pairwise を維持。allpairspy で約 20〜25 jobs に収束、安全マージン込みで **Phase 3 上限 30 jobs** に収まる。

`tests/qualify/matrix.yaml` の構造は以下:

```yaml
phases:
  phase0:
    description: "リリース時点。ローカル 1〜2 台で qualify 完結。jobs ≤ 8"
    axes:
      kubernetes_version: ["1.30", "1.31"]
      cni: ["cilium", "calico"]
      csi: ["longhorn", "local-path"]
      lb: ["metallb", "cloud-provider-kind"]
      ip_stack: ["ipv4", "dual-stack"]
      node_os: ["ubuntu-24.04"]
      arch: ["amd64"]
      runtime: ["containerd"]
    strategy: pairwise
    max_jobs: 8

  phase3:
    description: "sponsor cluster 確保後。jobs ≤ 30"
    axes:
      kubernetes_version: ["1.29", "1.30", "1.31"]
      cni: ["cilium", "calico", "flannel"]
      csi: ["longhorn", "local-path", "nfs", "cloud-disk"]
      lb: ["metallb", "cloud-provider-kind", "cloud-lb"]
      ip_stack: ["ipv4", "dual-stack", "ipv6"]
      node_os: ["ubuntu-24.04", "debian-12", "bottlerocket"]
      arch: ["amd64", "arm64"]
      runtime: ["containerd", "crio"]
    strategy: pairwise
    max_jobs: 30
```

matrix 算出は `tools/qualify/matrix-gen.py` で実行する。POSIX shell では pairwise 算出が現実的でないため、Python（標準 `itertools` + `allpairspy` パッケージ）を補助ツールとして許容する。生成された具体 jobs 一覧は `tests/qualify/matrix-resolved.yaml` に commit され、matrix.yaml の変更は PR レビューで matrix-resolved.yaml の差分が見える形で管理する。

各層（L4 / L5 / L7 / L9 / L10）が matrix のどの軸に依存するかを `tests/e2e/<layer>/MATRIX.md` で宣言する。例: L7 chaos は CNI に強く依存（NetworkPolicy / network-partition の挙動が CNI 実装で変わる）、L10 DR は CSI に強く依存（PV snapshot / restore の挙動が CSI 実装で変わる）、のような層別 matrix 重み付けを散文で記述する。

## 検討した選択肢

### 選択肢 A: 完全 matrix（5184 組み合わせ全実行）

- 概要: K8s version × CNI × CSI × LB × IP stack × OS × arch × runtime の 5184 組み合わせをすべて実行
- メリット:
  - 理論的に全組み合わせを網羅、未検証組み合わせがゼロ
  - 採用検討者が「すべての環境で動作保証されている」と最強の主張ができる
- デメリット:
  - **リソース不可能**: ローカルマシンでも cloud でも回らない（1 jobs 平均 30 分として 5184 jobs = 108 日）
  - L8 scale / L10 DR を含めると数年単位の連続実行になり、qualify が完了する前に次のリリースが来る
  - jobs 1 つあたりの flaky 率が 0.1% でも、5184 jobs では 99% 以上 1 つは fail する数学的保証になり、qualify 全体が常に赤になる

### 選択肢 B: 起案者が手作業で代表値選定（pairwise なし）

- 概要: 起案者が「これとこれは重要」と主観で 8〜12 組み合わせを選ぶ
- メリット:
  - 算出ツール不要、シンプル
  - 起案者の経験値が反映される
- デメリット:
  - **属人性が爆発**: 起案者交代 / contributor 増加で「なぜこの 8 組み合わせなのか」が説明できない
  - 軸ペア（CNI × CSI / OS × arch 等）のカバー率が保証されず、未検証ペアが残る可能性
  - 軸を 1 つ追加するたびに 8 組み合わせを手作業で再選定する必要があり、保守コストが線形以上で増える

### 選択肢 C: pairwise 抽出（採用）

- 概要: all-pairs combinatorial testing（pairwise testing）で、すべての軸ペアを 1 回以上カバーする最小 jobs 数を組合論で算出
- メリット:
  - **軸ペアのカバー率が数学的に保証**: 任意の 2 軸の組み合わせがすべて 1 回以上検証される（CNI × CSI の組み合わせが全部 / CNI × OS の組み合わせが全部 / 等）
  - **jobs 数が劇的に圧縮**: 5184 組み合わせ → Phase 0 で 4〜6 jobs / Phase 3 で 20〜25 jobs。リソース制約に収まる
  - **採用検討者への説明が組合論で済む**: 「なぜ 8 jobs か」の答えが「pairwise で必要な最小 jobs 数」と機械的に定まる
  - **業界標準**: Microsoft PICT / Hexawise / allpairspy 等のツール群が成熟、CNCF プロジェクトでも採用例多数
  - **軸追加が低コスト**: matrix.yaml に軸を 1 つ追加すれば matrix-gen.py が再算出
- デメリット:
  - **3 軸以上の組み合わせはカバーされない**: CNI = X / CSI = Y / OS = Z の三つ組のうち未検証の組み合わせは残る。重要な三つ組（例: Cilium + Longhorn + Ubuntu）はリファレンス jobs として固定する追加ロジックが要る
  - Python 依存が発生（allpairspy パッケージ）。POSIX shell + Make 縛りに対する例外として ADR で明示する必要
  - pairwise の結果が組合論的に最適化されるため、jobs 一覧を見て「なぜこの組み合わせが選ばれたか」が直感的でない場合がある

### 選択肢 D: matrix なし（リファレンス構成 1 つだけ）

- 概要: matrix を放棄し、リファレンス構成（K8s 1.31 / Cilium / Longhorn / MetalLB / dual-stack / Ubuntu / amd64 / containerd）の 1 jobs だけを qualify で回す
- メリット:
  - jobs 数 1 で qualify が爆速
  - matrix 算出ツール不要
- デメリット:
  - **採用組織が異なる構成を選んだ瞬間に動作保証が無い**: 採用組織が Calico を選びたい / arm64 で動かしたい / NFS を使いたい等の希望が、リリース時点で「未検証」になる
  - 採用検討者が「k1s0 は Ubuntu+amd64+Cilium 以外で動くか不明」と評価し、採用判断が消極的になる
  - ADR-INFRA-001 が「Infrastructure Provider 切替で vSphere / AWS / GCP / OpenStack / Bare Metal の差分を overlay で吸収」と決定しているのと矛盾（環境差分を ADR で謳いながらテストでは検証しない）

## 決定理由

選択肢 C（pairwise 抽出）を採用する根拠は以下。

- **jobs 数とカバー率の最適バランス**: pairwise は理論上 jobs 数の対数的圧縮を提供しつつ、軸ペアのカバー率を 100% 保証する。完全 matrix（A）はカバー率 100% だが jobs 数が爆発、手作業（B）は jobs 数を圧縮できるがカバー率が保証されない、matrix なし（D）はカバー率がほぼゼロ。pairwise は「カバー率 100%（軸ペア） + jobs 数 8〜30」という他選択肢と桁違いのバランスを提供する
- **採用検討者への説明合理性**: 「なぜこの 8 jobs を選んだか」の答えが「pairwise で必要な最小 jobs 数」と組合論で機械的に定まる。選択肢 B では起案者の主観に依存するため、Phase 2 で contributor が増えた段階で議論が紛糾する。pairwise であれば「軸を変えれば jobs が変わる、軸が同じなら jobs は同じ」という客観性が保たれる
- **Phase 移行への対応性**: matrix.yaml に Phase 0 / Phase 3 の axes を併記することで、Phase 移行で jobs 数が機械的に拡張される。完全 matrix（A）は Phase 移行に関わらず破綻、matrix なし（D）は Phase 移行で軸を追加しても jobs 数が増えない（拡張余地ゼロ）
- **ADR-INFRA-001 / ADR-NET-001 / ADR-STOR-001 / ADR-STOR-002 との整合**: 既存 ADR で Cilium / Longhorn / MetalLB / Ubuntu を本番採用と決めているが、それぞれの ADR で「将来 Calico / NFS / cloud LB / 他 OS への移行可能性」を「帰結」セクションで言及している。pairwise matrix で代替候補を最初から検証することで、将来の移行コストを下げる証跡が積み上がる
- **CI 移植容易性**: `tests/qualify/matrix.yaml` を allpairspy で resolve した結果（`tests/qualify/matrix-resolved.yaml`）は、GitHub Actions `strategy.matrix` の `include:` 形式と構造的に一致する。Phase 1 で CI を導入した瞬間、matrix-resolved.yaml をそのまま `.github/workflows/qualify.yml` に流し込めば matrix jobs として展開される。選択肢 B / D は CI 化時に再設計が要る
- **Python 依存の例外性は許容範囲**: ADR-TEST-001 portable 制約 1（POSIX shell + Make）に対する例外として、matrix 算出のみ Python（allpairspy）を許容する。これは「Python が壊れていれば matrix を再算出できないが、resolved.yaml が commit 済みなので qualify 自体は走る」という非対称な依存で、実害が局所化される。matrix-gen.py は qualify 実行時には呼ばれず、軸追加時のみ手動で実行される

## 影響

### ポジティブな影響

- 軸ペアのカバー率が pairwise で数学的に 100% 保証され、採用検討者に「k1s0 はリリース時点で 8 環境組み合わせを検証している」と組合論ベースで主張できる
- jobs 数が Phase 0 で 8 以内 / Phase 3 で 30 以内に圧縮され、ローカル / sponsor cluster の双方で現実的に回せる
- matrix.yaml の axes に軸を 1 つ追加すれば matrix-gen.py が再算出するため、軸追加の保守コストが線形以下に抑えられる
- Phase 0 / Phase 3 の axes を 1 ファイルで併記することで、Phase 移行が ADR-TEST-001 の Phase 表と一意に対応する
- 既存 ADR（INFRA-001 / NET-001 / STOR-001/002）の代替候補が pairwise matrix で最初から検証されるため、将来の本番技術選択肢の移行コストが下がる
- `tests/qualify/matrix-resolved.yaml` が PR レビュー対象になることで、軸変更が起案者の主観ではなく組合論的に検証可能な変更として記録される

### ネガティブな影響 / リスク

- 3 軸以上の組み合わせ（例: Cilium + Longhorn + Ubuntu の三つ組）は pairwise でカバーされない場合がある。重要な三つ組はリファレンス jobs として固定する追加ロジックを matrix-gen.py に組み込む必要がある（matrix.yaml の `must_include:` セクションで明示）
- Python（allpairspy）依存が ADR-TEST-001 portable 制約 1 の例外となる。ADR で例外性を明示しているが、CI 移植時に Python runtime を CI image に含める手間が発生する
- pairwise の結果が組合論的に最適化されるため、jobs 一覧を見て「なぜこの組み合わせが選ばれたか」が直感的でないことがある。`docs/governance/QUALIFY-POLICY.md` で pairwise の数学的根拠を散文で説明する必要
- 軸を 9 個目以上に増やすと jobs 数が 30 を超える危険があり、Phase 3 上限を超える。新軸追加時は matrix-gen.py の dry-run で jobs 数を事前確認する規律が要る
- multipass kubeadm（ADR-TEST-004）で Phase 0 の 8 jobs を順次実行すると、L5 conformance 全体で 8 jobs × 30 分 = 4 時間の release qualify 時間が発生する。release tag 切る作業に 4 時間以上の連続マシン占有が必要

### 移行・対応事項

- `tests/qualify/matrix.yaml` を新設し、Phase 0 / Phase 3 の axes と pairwise strategy を宣言的に記述する
- `tools/qualify/matrix-gen.py` を新設し、allpairspy で matrix を resolve する。Python 依存を `.devcontainer/Dockerfile` で `pip install allpairspy` として固定
- `tests/qualify/matrix-resolved.yaml` を生成 commit し、PR レビュー対象にする。matrix.yaml が変更されたら必ず matrix-resolved.yaml も再生成する規律を `docs/governance/QUALIFY-POLICY.md` で明文化
- 各層 `tests/e2e/<layer>/MATRIX.md` を新設し、層別 matrix 重み付け（L7 は CNI 重視、L10 は CSI 重視等）を散文で記述
- `tools/qualify/runner.sh` を新設し、matrix-resolved.yaml の jobs を順次（Phase 0）または並列（Phase 3）で実行する。Phase 0 では順次実行のみサポート
- `docs/governance/QUALIFY-POLICY.md` に pairwise の数学的根拠と「なぜ全組み合わせではなく pairwise か」を散文で記述、採用検討者の説明動線を確立
- リリース時点で Phase 0 の 8 jobs を 1 回手動実行し、結果を `tests/qualify-report/release-initial/matrix-baseline/` に保存する（採用検討者向けの初期証跡）
- ADR-NET-001 / ADR-STOR-001 / ADR-STOR-002 / ADR-INFRA-001 の代替候補（Calico / NFS / cloud LB / arm64 等）が pairwise matrix で検証されることを各 ADR の「帰結」に追記する relate-back 作業

## 参考資料

- ADR-TEST-001（CI 留保 + qualify portable 設計）— portable 制約 7「マトリクス定義を YAML で外出し」の実装
- ADR-TEST-002（devcontainer + HW 要件）— Python（allpairspy）を含む toolchain 固定
- ADR-TEST-003（テストピラミッド L0–L10）— 各層と matrix の対応（層別 MATRIX.md）
- ADR-TEST-004（kind + multipass 二層 E2E）— matrix が回るクラスタ実装
- ADR-INFRA-001（Cluster API + kubeadm）— 環境差分の overlay 吸収思想
- ADR-NET-001（Cilium / Calico / kindnet）— CNI 軸の本番 / 代替候補
- ADR-STOR-001（Longhorn）— CSI 軸の本番値
- ADR-STOR-002（MetalLB）— LB 軸の本番値
- ADR-DEV-002（WSL2 + Docker）— OS 軸のリファレンス値
- NFR-F-CHR-002（環境差分への耐性）— matrix の存在根拠
- allpairspy: github.com/thombashi/allpairspy
- Pairwise testing 解説: pairwise.org
- Microsoft PICT: github.com/microsoft/pict
- 関連 ADR（採用検討中）: ADR-TEST-006（chaos / scale / soak）/ ADR-TEST-007（upgrade / DR）
