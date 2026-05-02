# ADR-TEST-001: リリース時点では CI を導入せず、qualify 基盤を CI portable に設計する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: 起案者 / 採用検討組織 / SRE / 開発者体験チーム

## コンテキスト

k1s0 はオンプレ完結（NFR-F-SYS-001）を要件とする個人 OSS で、採用検討組織が「10 年保守する」前提で評価する 採用側 PaaS である。リリース時点における品質保証の検証経路（テスト・lint・format・supply chain 検証・SLO assertion 等）をどこで走らせるか、すなわち CI（Continuous Integration）の導入要否は、起案者一人で運用する個人 OSS にとって**コスト・運用工数・採用検討者の信頼**の三軸で正面から衝突する判断である。

CI を持たないと、採用検討者が「テスト体制が機械的に検証されていない」と判定し、CNCF Sandbox 申請時の testing maturity 軸（後述）や OSSF Scorecard の `CI-Tests` チェックで低評価を受けるリスクがある。一方で、個人 OSS でクラウド CI（GitHub Actions の有償枠 / GitLab CI / マネージド SaaS）や self-hosted runner を常時運用するには、月額予算の継続的な持ち出し、runner 環境の更新追従、ジョブ失敗時のトリアージ工数が必要で、起案者一人が他のロードマップ作業と並行して背負うのは持続困難である。リリース時点では contributor も sponsor も存在せず、cost を分担する相手がいない。

この問題の核心は「CI を恒久放棄するのか、それとも将来導入するための足場を残すのか」にある。前者なら採用検討者向けの説明責務が永続的に重くなり、後者なら**今 CI を持たないことと、将来 CI を持つことの両方に整合する設計**を最初から採る必要がある。リリース時点の qualify 基盤を場当たり的に作ると、Phase 1 で CI を後付けする際に基盤の総書き換えが発生し、移行コストが累積負債として顕在化する。

加えて、k1s0 は「バス係数 2」（NFR-A-CONT-001 / ADR-OPS-001）を運用面の柱としており、起案者不在の夜間休日でも協力者が単独で品質を回せる構造でなければならない。CI に強く依存した品質モデルは、CI 障害時に協力者が手元で再現できないリスクを孕む。逆に「ローカルで完結する qualify が一次経路」「CI は二次的なオプション」という非対称な配置にすれば、CI が無くても品質は崩れず、CI が増えれば加点されるだけ、という頑健な構造になる。

以上から、リリース時点の品質モデルは以下を同時に満たす必要がある:

- **コスト 0 円**で成立する（個人 OSS が継続的に予算を持ち出さない）
- **採用検討者の信頼**を release artifact 経由で得る（CI badge が無くても testing maturity が証跡で示せる）
- **bus 係数 2** と整合する（CI に依存しない、起案者不在で回る）
- **将来の CI 移行**が低コスト（Phase 1 で YAML 1 本の追加レベルで済む）
- **portable 制約の遵守**が CI なし期間でも崩れない（仮に基盤が CI 非対応で書かれてしまうと、Phase 1 で総書き換えになる）

## 決定

**リリース時点では CI（GitHub Actions / GitLab CI / 自前 runner / マネージド SaaS CI 等の機械的な PR 自動検証基盤）を一切導入しない。** 品質保証の一次経路はローカルマシン上の `make qualify-release` に集約し、release tag を切る行為そのものを `tools/release/cut.sh` ラッパで qualify 強制に紐付ける（実装の詳細は ADR-TEST-003 / ADR-TEST-004）。

ただし「CI 不採用」は恒久決定ではなく**将来構想として留保**する。Phase 1 以降の移行を「YAML 1 本の追加」レベルで済ませるため、qualify 基盤を最初から CI portable に設計する。具体的な portable 制約は以下 7 項目とする。

1. **実行系は POSIX shell + GNU Make に統一する**。bash 拡張・zsh 拡張・PowerShell・Python script 等の特定ランタイム前提を qualify 基盤の中核に持ち込まない。GitHub Actions / GitLab CI / Buildkite / 自前 runner のいずれにも追加実装なしで載せられる状態を維持する。
2. **環境前提は env var で抽象化する**。バイナリの絶対パス（例: `/home/ryuhei_kiso/...`）、kubeconfig パス、artifact 出力先、kind cluster 名などを hardcode せず、`KIND_BIN` / `KUBEADM_BIN` / `ARTIFACT_DIR` / `KUBECONFIG` / `QUALIFY_LAYER` 等で外部注入する。
3. **log / report は JSON Lines と Markdown を必ず両方出す**。人間が読む Markdown は採用検討者の release artifact 同梱用、JSON Lines は将来 CI dashboard / 集計スクリプトの入力用。両方が常に同時生成される構造にする。
4. **artifact upload を `tools/qualify/artifact.sh push <path>` で抽象化する**。リリース時点では local（`tests/qualify-report/`）への保存と minio (`tools/local-stack/` で起動するローカル S3 互換) への push のみを実装、Phase 1 で同 interface のまま GitHub Release Asset / S3 / GCS への push を後付けできる構造にする。
5. **開発環境は devcontainer を SoT として固定する**（ADR-TEST-002 で詳細）。これにより qualify が想定する toolchain / OS / 依存バイナリのバージョンが完全固定され、CI runner image の生成が devcontainer 設定の使い回しで済む。
6. **secret は OpenBao 経由で取得する**（ADR-SEC-002 と整合）。`.env` / kubeconfig / cloud credentials を平文で扱う経路を qualify 基盤に作らない。Phase 1 で CI secret store（GitHub Actions Secrets 等）に移行する際、OpenBao からの注入経路を CI secret 注入に差し替えるだけで済む。
7. **マトリクス定義を `tests/qualify/matrix.yaml` で外出しする**。「どの K8s version / どの CNI / どの CSI」のような matrix を shell script のループに埋め込まず、宣言的 YAML から生成する。Phase 1 で同じ YAML を GitHub Actions の `strategy.matrix` に machine-translation できる。

Phase 移行のロードマップは以下のとおり。リリース時点（Phase 0）の射程と、Phase 1 以降の移行条件を客観条件で明記することで、「将来構想」が言い訳として永続化することを防ぐ。

| Phase | 時期 | 範囲 | 移行条件（客観条件） |
|-------|------|------|--------------------|
| **Phase 0** | リリース時点 | qualify 全層をローカル `make qualify-release` で必須化、release artifact が唯一の品質公開経路 | — |
| **Phase 1** | 採用初期 | GitHub Actions free tier で L0–L3（contract / unit / integration / smoke）のみ自動化 | コスト 0 円で成立すること |
| **Phase 2** | — | L4 standard E2E を CI 化、PR gate に組み込み | contributor 2 名以上参画、または sponsor 月 50 USD 以上獲得 |
| **Phase 3** | — | L6 portability（EKS / GKE / AKS）/ L8 scale 5000 node / 24h soak を sponsor cluster で本格化 | 月予算 200 USD 以上、または cluster sponsor 提供 |
| **Phase 4** | — | testgrid 相当の dashboard / multi-arch / multi-OS matrix 完備 | 専任 0.5 FTE 以上の運用体制 |

「リリース時点」「採用初期」は k1s0 ロードマップの既存タームと対応する。Phase 0 の射程はテストピラミッド全層（L0–L10）を含むが、L6 portability のみ「Phase 0 で 1 回だけ手動 EKS qualify run を実走し、証跡を release artifact に同梱、自動化は Phase 3」とする例外を持つ（詳細は ADR-TEST-003）。

採用検討者向けには `docs/governance/CI-ROADMAP.md` を新設し、本 ADR の Phase 表を抜粋して公開する。これにより「CI を持たない」が「永続的に持たない」ではなく「Phase 移行条件が客観的に充足したら即移行する義務を負う」立場であることを明示する。

## 検討した選択肢

### 選択肢 A: CI 完全不採用（恒久放棄）

- 概要: リリース時点も将来も CI を導入しない。qualify は永続的にローカル完結とする。本 ADR の portable 制約を持たず、ローカル特化で実装する
- メリット:
  - portable 制約の遵守工数（+2〜3 人日 + 継続的な規律）が不要
  - shell / Python / Rust 製ツール等を自由に選択でき、qualify 基盤の実装速度が上がる
  - 「CI なし」を OSS のスタンス（local-first / offline-first 哲学）として打ち出せる
- デメリット:
  - 採用検討者が CI badge を期待する場面（OSSF Scorecard の `CI-Tests`、CNCF Sandbox 申請の testing maturity 軸）で構造的に低評価を受ける
  - sponsor / contributor が出現した時点で qualify 基盤の総書き換えが発生し、Phase 1 移行コストが工数 +20〜40 人日に膨張する
  - 「永続的に CI を持たない」は採用検討者から見ると敗北宣言に近く、OSS としての成長余地を自ら閉じる
  - bus 係数 2 を満たす局所最適解だが、commun­ity 形成という大局では逆効果

### 選択肢 B: リリース時点から GitHub Actions を導入（free tier 範囲）

- 概要: リリース時点で GitHub Actions の free tier（public repo 無制限）に L0–L3 を載せる。qualify はローカルと CI の二重実装で並走させる
- メリット:
  - 採用検討者が PR / commit ごとの CI status badge を見られる
  - GitHub Actions free tier は public repo で無制限のため月額コスト 0 円
  - OSSF Scorecard の `CI-Tests` チェックを満たせる
- デメリット:
  - **runner 環境差異の検証コスト**が発生する。`ubuntu-latest` runner と起案者の WSL2 + devcontainer 環境の差で flaky が起きると、ローカル再現できないバグの triage が個人 OSS で吸収しきれない
  - **GitHub への単一依存**が固定化される。GitHub が将来 free tier を縮小する / k1s0 が GitHub から離れる選択を取った場合、qualify 経路が崩れる
  - **二重実装の保守工数**が継続的に発生する。ローカル qualify と CI workflow の差異を起案者一人で常に同期する必要があり、bus 係数 2 と矛盾
  - リリース時点で contributor が居ないため、CI が拾うバグの量に対して triage capacity が足りない（CI が「鳴っているのに誰も応答しない」アラート疲労状態を生む）

### 選択肢 C: CI 導入の留保 + Phase 制移行 + qualify portable 設計（採用）

- 概要: リリース時点は CI なし、qualify はローカル完結。ただし qualify 基盤に portable 制約 7 項目を課し、Phase 1 以降の CI 移行を YAML 1 本の追加で済ませる
- メリット:
  - リリース時点でコスト 0 円、契約交渉ゼロ、第三者依存ゼロ
  - qualify が手元マシンで完結し、bus 係数 2 と整合
  - portable 制約により Phase 1 移行コストが工数 +2〜3 人日に圧縮できる
  - release artifact が唯一の品質公開経路となり、採用検討者は手元で `make qualify` を再走することで CI badge と等価な検証を自分で行える
  - 「将来構想」を Phase 移行条件で客観化することで、先送りの言い訳化を構造的に防ぐ
- デメリット:
  - portable 制約の遵守が崩れると Phase 1 移行コストが累積負債化する。常時のレビュー規律が要る
  - 採用検討者が CI badge を期待した瞬間に説明責務（`docs/governance/CI-ROADMAP.md` を読ませる動線）が発生する
  - リリース時点の qualify 基盤実装に portable 制約のための追加工数 +10〜15% が発生

### 選択肢 D: self-hosted runner 即時導入

- 概要: 起案者の手元マシン or 専用 hardware に GitHub Actions self-hosted runner を立て、リリース時点から CI を回す
- メリット:
  - クラウド runner の environment 差異問題が解消する（runner = 自分の環境）
  - free tier の制限を超える長時間ジョブ・大型 RAM ジョブが回せる
  - L7 chaos / L8 scale / L9 upgrade 等の重量級も最初から CI に載せられる
- デメリット:
  - **runner 自体の運用工数**が爆発する。OS update / GitHub Actions runner binary 追従 / 障害対応 / セキュリティ update が起案者一人に乗る
  - **常時稼働ハードウェア**が前提となり、電気代 / 騒音 / 物理スペースの個人負担が発生
  - **public repo に self-hosted runner を晒すセキュリティリスク**が大きい（fork PR からの runner 乗っ取り）。GitHub 公式も非推奨
  - bus 係数 2 と矛盾する（runner 障害時に協力者が代替できない）

## 決定理由

選択肢 C を採用する根拠は以下。

- **コスト 0 円とコストゼロ運用の同時成立**: 選択肢 B / D はコストまたは工数のいずれかが個人 OSS に重すぎ、選択肢 A はコスト最小だが採用検討者信頼が失われる。選択肢 C は「リリース時点 = コスト 0 円、運用工数最小、信頼は release artifact で代替」の三立を唯一実現する
- **bus 係数 2 と整合**: ADR-OPS-001 の「夜間休日に起案者不在で協力者が単独対応する」前提は、品質保証経路にも適用される。CI 依存度を下げてローカル qualify を一次経路にすることで、CI 障害時の品質崩壊リスクを構造的に排除できる。選択肢 B / D は CI 障害が即座に品質経路を断つ
- **将来移行コストの先払い vs 後払い**: 選択肢 A は「CI 後付け時に総書き換え（+20〜40 人日）」、選択肢 C は「リリース時点で portable 制約遵守（+2〜3 人日）+ Phase 1 で YAML 追加（1 人日）」となり、トータル工数で C が圧倒的に安い。先払いの 2〜3 人日は portable 制約という形で品質に直接寄与するため、純粋な追加工数ではない
- **Phase 移行条件の客観化による先送り防止**: 選択肢 C の Phase 表は「contributor 2 名以上」「sponsor 月 200 USD」のように客観条件で書かれており、達成時点で機械的に Phase 移行が起動する。「採用検討者が出現したら考える」のような主観的留保ではないため、CLAUDE.md ポリシー「未来への先送りは許さない」と整合する
- **L6 portability の例外的扱い**: 選択肢 C のみが「Phase 0 で 1 回だけ手動 EKS qualify run を実走し、自動化は Phase 3」という非対称な扱いを許容できる。選択肢 A は portability を恒久放棄、選択肢 B / D は最初から自動化を強要する設計でリリース時点コストが膨張する
- **release artifact 中心の品質公開経路の確立**: 選択肢 C は「release tag を切る = qualify 強制 = artifact 添付」の三位一体を成立させる（ADR-TEST-003 で詳細）。これは CI 中心モデル（PR ごとに status を出す）とは異なる「リリース粒度で品質を担保する」モデルであり、個人 OSS の release cadence（月次〜四半期）と整合する

## 影響

### ポジティブな影響

- リリース時点でクラウド CI の月額予算 0 円が達成され、個人 OSS の経済的持続性が担保される
- qualify が手元マシンで完結し、CI 障害時の品質崩壊リスクが構造的に排除される（CI が無いので CI 障害が起きない）
- portable 制約 7 項目の遵守により、Phase 1 移行が「`.github/workflows/qualify.yml` の追加 + matrix.yaml の `strategy.matrix` への machine-translation」で済む
- release artifact に qualify report（JSON + Markdown + chaos timeline + Sonobuoy report + SLSA provenance + SBOM）が必ず同梱され、採用検討者が手元で再現実行できる
- `docs/governance/CI-ROADMAP.md` の Phase 表により、「将来 CI を持つ意思があるが、客観条件が揃うまで先送る」という立場が採用検討者に対して明示される
- bus 係数 2 と整合し、起案者不在でも協力者が `make qualify-release` を回せば品質経路が崩れない

### ネガティブな影響 / リスク

- 採用検討者が CI status badge を期待する場面（GitHub repo top の README badge 一覧、OSSF Scorecard 自動採点）で説明責務が発生する。`docs/governance/CI-ROADMAP.md` への動線を README に明示する必要
- 開発者ローカルマシンが nightly qualify / release qualify で長時間（夜間 2〜4 時間 / release 時 半日〜1 日）占有される。SSD 寿命・ファン稼働音・電気代が個人負担で増える
- ハードウェア要件（32GB RAM 以上 / NVMe 1TB 以上 / arm64 検証用に追加機 1 台）が個人投資として 30〜50 万円規模で発生する（詳細は ADR-TEST-002）
- portable 制約 7 項目の遵守が崩れると、Phase 1 移行コストが累積負債化する。レビュー時に「shell 拡張を持ち込んでいないか」「hardcoded path が無いか」を継続的に問う規律が要る
- OSSF Scorecard の `CI-Tests` チェックは満点（10/10）にならない。これは採点上の不利として顕在化するが、`docs/governance/CI-ROADMAP.md` で意図を説明することで採用検討者の理解を求める
- リリース時点で CI による fork PR の自動検証が無いため、外部 contributor が増えた瞬間にレビュー工数が増える。Phase 2 の移行条件（contributor 2 名以上）はこのリスクを inflection point として扱う

### 移行・対応事項

- `tools/qualify/` 配下に POSIX shell + GNU Make ベースの qualify 基盤を新設し、portable 制約 7 項目を実装段階から強制する
- `tools/qualify/matrix.yaml` を起票し、qualify が走らせるべき matrix（K8s version / CNI / CSI 等）を宣言的に記述する。詳細は ADR-TEST-005 で定義
- `tools/qualify/artifact.sh push <path>` を実装し、artifact upload の抽象化レイヤを確立する。リリース時点は local + minio のみ実装、Phase 1 で GitHub Release Asset / S3 / GCS の subcommand を追加可能な構造とする
- `tools/release/cut.sh` を新設し、release tag を切る唯一の入口とする。`git tag` の直接実行を `core.hooksPath = .githooks` 経由の pre-push hook で塞ぐ（詳細は ADR-TEST-003）
- `docs/governance/CI-ROADMAP.md` を新設し、Phase 0–4 のロードマップと移行条件を採用検討者向けに公開する
- `docs/governance/QUALIFY-POLICY.md` を新設し、「k1s0 の品質は CI ではなく release artifact が証跡である」を宣言する
- README.md に「k1s0 は CI を持たず、release artifact が品質証跡である」旨を明記し、`CI-ROADMAP.md` / `QUALIFY-POLICY.md` への動線を貼る
- 各 release tag に qualify report (JSON + Markdown) を asset として添付する運用を `tools/release/cut.sh` で強制する
- ADR-TEST-002〜009 で portable 制約の各論を補強する（環境固定 / テストピラミッド / 二層 E2E / matrix / chaos / upgrade / コンプライアンス / 観測性）
- `docs/03_要件定義/00_要件定義方針/08_ADR索引.md` に TEST 系列を新設し、本 ADR を「テスト戦略」カテゴリで登録する
- OSSF Scorecard 採点時の `CI-Tests` 項目低評価への対応として、`docs/governance/CI-ROADMAP.md` を SECURITY.md / README から linkback して、採点コンテキストを明示する

## 参考資料

- ADR-DEV-002（Windows + WSL2 + Docker runtime）— 開発環境のリファレンス OS 定義
- ADR-OPS-001（Runbook 標準化 + バス係数 2 の構造化）— 本 ADR の bus 係数 2 整合の前提
- ADR-POL-002（local-stack を構成 SoT に統一）— ローカル完結思想の先例、本 ADR は同じ哲学を qualify 経路まで拡張する
- ADR-CNCF-001（vanilla K8s + CNCF Conformance 維持）— Phase 3 で自動化される L6 portability / L5 conformance の整合性根拠
- ADR-SEC-002（OpenBao 採用）— portable 制約 6（secret 抽象化）の依存
- NFR-F-SYS-001（オンプレ完結）— qualify がクラウド CI に依存しない根拠
- NFR-C-NOP-001（小規模運用）— bus 係数 2 と CI 依存度低減の整合
- NFR-A-CONT-001（HA / RTO 4 時間）— ローカル qualify の協力者単独実行可能性の根拠
- 関連 ADR（採用検討中）: ADR-TEST-002（開発環境標準化）/ ADR-TEST-003（テストピラミッド L0–L10）/ ADR-TEST-004（kind + multipass 二層 E2E）/ ADR-TEST-005（環境マトリクス）/ ADR-TEST-006（chaos / scale / soak）/ ADR-TEST-007（upgrade / DR）/ ADR-TEST-008（コンプライアンス）/ ADR-TEST-009（観測性 E2E）
- OSSF Scorecard `CI-Tests` 項目: scorecard.dev/checks/#ci-tests
- OpenSSF Best Practices Badge: bestpractices.coreinfrastructure.org
- GitHub Actions self-hosted runner security: docs.github.com/en/actions/hosting-your-own-runners
