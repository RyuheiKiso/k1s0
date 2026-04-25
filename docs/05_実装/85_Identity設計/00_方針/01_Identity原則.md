# 01. Identity 原則

本ファイルは k1s0 の ID 基盤（Keycloak / SPIRE-SPIFFE / OpenBao / cert-manager / Istio Ambient mTLS）を実装する際に常に参照する 7 本の設計原則を定義する。新規サービスの認証経路追加、Secret 参照方式の変更、証明書発行ルートの追加が発生した際、本原則との整合を確認することで、「どこかに残った局所認証が退職時 revoke を漏らす」破綻を構造的に防ぐ。

![Identity 原則俯瞰](../img/85_Identity4軸統合.svg)

## 原則が必要な理由

採用側組織の 10 年保守サイクルでは、従業員・委託先・業務委託先の入退場が年単位で発生する。退職者の ID が 1 箇所でも残っていれば、退職翌日の不正アクセスという最悪インシデントが発生する。Keycloak / OpenBao / cert-manager / SPIRE を「個別に」運用すると、以下の破綻が常態化する。

- Keycloak を無効化しても、特定サービスにハードコードされた API キーが生きており不正ログインが成立する
- SPIRE の SVID を廃止しても、旧 Pod に焼き込んだ静的証明書で横移動される
- OpenBao の root token が手渡しで流通し、退職者の手元に残る
- cert-manager による自動更新を止めて手更新が発生し、期限切れで mTLS が崩壊する
- Istio Ambient の mTLS を一部 Namespace でバイパスする例外が累積し、平文通信が残存する

本原則は、これらの破綻を「Keycloak 1 アカウント無効化で全経路 revoke」という単一の運用点に収斂させるための 7 本である。個別最適ではなく、退職時 revoke 演習（GameDay）で失敗しない構造を優先する。

## 原則 1: 人間 ID は Keycloak に一本化する（IMP-SEC-POL-001）

**人間の認証は Keycloak OIDC を経由するもののみ許容する。ローカルパスワード・ローカル LDAP・サービス固有の管理画面ログインは禁止する。**

Keycloak 以外に人間 ID の発行元が存在すると、退職時 revoke 手順が「Keycloak 無効化 + 各サービスの独自ログイン削除」に膨れ上がり、漏れの温床となる。ADR-SEC-001 で Keycloak を選定済であるため、実装層では「Keycloak 以外の人間 ID 経路を作らない」ことを強制する。Keycloak realm は 採用側組織の組織構造（本体 / 子会社 / 委託先）に対応する形で 3 段階に分割し、group claim に所属ラベルを含めることで、tenant_id 検証（NFR-E-AC-003）の上流として機能させる。

具体的な帰結として、以下を固定する。

- tier1 公開 11 API はすべて OIDC Bearer を必須とし、JWT 検証は Envoy Gateway + authservice で集約する（NFR-E-AC-001）
- 管理 UI・運用コンソール・Backstage は Keycloak SSO に統合し、個別ログイン画面を持たない
- CI から tier1 を叩く場合も Keycloak client credential grant とし、固定トークンは発行しない
- DB への直接ログインは Keycloak group claim を参照する認可プロキシ経由のみ
- MFA（NFR-E-AC-005）は Keycloak の authenticator flow で全ユーザー必須化し、例外を ADR 承認制とする

## 原則 2: ワークロード ID は SPIFFE/SPIRE SVID で発行する（IMP-SEC-POL-002）

**サービス間通信の ID は SPIRE が発行する SVID（SPIFFE Verifiable Identity Document）のみを使う。静的 credential（長期 API キー / 固定 mTLS 証明書 / basic auth）の利用を禁止する。**

ADR-SEC-003 で SPIFFE/SPIRE を選定済であり、ADR-0001 で Istio Ambient を選定済である。両者は SVID を Ambient mesh に載せる前提で組み合わさる。静的 credential を 1 つでも残すと、Pod がコンテナエスケープされた際に攻撃者が当該 credential を持ち出し、SVID の短寿命性（デフォルト 1 時間）による封じ込めが効かなくなる。

SVID の取得経路は Workload API（Unix Domain Socket）経由に限定し、アプリコードが直接 X.509 を読み書きする実装を禁止する。これにより、「鍵の置き場所」という設計論点がアプリ層から消える。具体的な帰結として、以下を固定する。

- Workload attestation は Kubernetes Service Account + Namespace + Pod label の組み合わせで実施
- SVID TTL は 1 時間（デフォルト）を上限とし、延長の例外は ADR 起票必須
- アプリコードから参照する SPIFFE ID 命名規則は `spiffe://k1s0/ns/<namespace>/sa/<serviceaccount>` に固定
- SPIRE Server の可用性は HA 構成（3 レプリカ以上）を `infra/security/spire/` に配置して担保

## 原則 3: 退職時は Keycloak 1 アカウント無効化で全経路 revoke する（IMP-SEC-POL-003）

**Keycloak group claim / role を OpenBao ポリシー・cert-manager 発行ポリシー・Kubernetes RBAC の権限判定に連動させ、Keycloak 上の 1 アクション（アカウント無効化）で全経路が失効する設計とする。**

退職時 revoke 手順が複数システムを個別に叩く構成は、運用手順書の長大化と漏れを招く。Keycloak の group claim を「唯一の権限源」として扱い、OpenBao は JWT 認証メソッドで Keycloak を信頼し、cert-manager の ClusterIssuer も Keycloak 連動の承認プロキシを挟む。Kubernetes RBAC は OIDC group claim を RoleBinding の subject として参照する。

運用上は `ops/runbooks/identity/退職時revoke.md` を単一の手順書とし、退職時 revoke 演習（GameDay）で 60 分以内の全経路失効を計測する。NFR-E-AC-005（MFA / 退職時 revoke）の受け入れ基準を本原則で満たす。Refresh token の短寿命化（15 分）と session 強制失効を組み合わせ、無効化後の既発行トークンの残存時間を実運用で 60 分未満に抑える。

## 原則 4: シークレットは OpenBao に一元化する（IMP-SEC-POL-004）

**DB パスワード・外部 API キー・署名鍵を含むすべてのシークレットは OpenBao で発行・管理する。コード・設定ファイル・環境変数ファイル（`.env`）・Kubernetes Secret への平文配置を禁止する。**

ADR-SEC-002 で OpenBao を選定済であり、Dapr Secret Store コンポーネントで tier1 / tier2 から参照する（DS-SW-COMP-006）。平文 Secret が 1 つでもリポジトリに混入すると、`git log` の全履歴から掘り起こされるため、予防的に検出・遮断する必要がある。Secret 取得の監査ログは全件 OpenBao の audit device 経由で WORM バケットへ転送し、NFR-E-MON-002 の取得監査要件を満たす。

- pre-commit hook で gitleaks / trufflehog を必須化（`tools/git-hooks/`）
- CI でも secret scanning を重ねて実行
- Kubernetes Secret は External Secrets Operator で OpenBao から同期のみ許容（kubectl apply で生の Secret を作成しない）
- 短寿命 credential（DB dynamic secret、AWS STS、Kafka SASL）を優先する
- OpenBao のポリシーは Keycloak group claim にマップし、退職時 revoke と連動させる

## 原則 5: 証明書は cert-manager による自動更新のみ（IMP-SEC-POL-005）

**X.509 証明書の発行・更新は cert-manager の ClusterIssuer / Issuer を通じて行う。手動更新・手動 kubectl apply での証明書投入を禁止する。**

証明書の手更新は「担当者が属人化し、退職時に期限を迎えて障害化する」日本企業の伝統的運用パターンである。cert-manager による自動更新は更新漏れを構造的に防ぐ。発行元は SPIRE upstream authority / 社内 CA / Let's Encrypt のいずれかに限定し、自己署名証明書の長期利用を禁止する。

証明書期限の監視は `60_観測性設計/` の SLI として計測し、残存期限が 30 日を切ったものは Alert を発火する。cert-manager の Renewal 失敗は即時 PagerDuty 通知とする（NFR-E-ENC-002 に対応）。証明書の発行ログは監査用に WORM ストレージへ保管し、事後監査で「いつ何が発行されたか」を追跡可能にする（`90_ガバナンス設計/` の WORM 原則と連動）。

## 原則 6: mTLS は Istio Ambient で全サービス間強制する（IMP-SEC-POL-006）

**`infra/mesh/istio-ambient/` の PeerAuthentication を `STRICT` に固定し、mesh 内の平文通信を全 Namespace で禁止する。`PERMISSIVE` 例外は認めない。**

ADR-0001 で Istio Ambient を選定済である。Ambient mode の ztunnel は Pod 側の改変なしで L4 mTLS を自動付与するため、アプリ側の責務は「mTLS を外さない」ことに尽きる。`PERMISSIVE` や `DISABLE` の例外を 1 つでも認めると、移行期という口実で恒久化する典型パターンに陥る。

リリース時点で `STRICT` 固定し、レガシー連携が必要な場合は Envoy Gateway 経由の南北トラフィックとして扱う（mesh 外で平文を一切作らない）。Kyverno ポリシーで `PeerAuthentication.spec.mtls.mode: STRICT` 以外を reject する（`90_ガバナンス設計/` と連動）。L7 認可（AuthorizationPolicy）は ztunnel ではなく waypoint proxy で処理する構成とし、RBAC 不備による認可漏れを ambient layer で検出可能にする。

## 原則 7: 退職時 revoke 演習を GameDay として定期実施する（IMP-SEC-POL-007）

**退職時 revoke 手順は `ops/runbooks/identity/` に保管し、四半期ごとの GameDay として疑似退職者アカウントで失効経路を全通し検証する。**

手順書は書いた瞬間から腐り始める。GameDay を制度化することで「書かれているが動かない手順」を早期検出する。疑似退職者アカウントを 1 個恒久的に維持し、四半期ごとに `enable → 権限付与 → 各経路確認 → disable → 60 分以内全経路失効確認` のサイクルを回す。

GameDay の結果は `ops/runbooks/identity/gameday-log.md` に時系列で追記し、失効までの実測時間を SLI 化する。改善対象は本章の原則または各実装節に ADR 起票で反映する。NFR-E-AC-005 の受け入れ基準達成を本演習で継続検証する。GameDay の成否は Backstage Scorecards に反映し（`95_DXメトリクス/` 連動）、Security / SRE / DX の三者で共有する。失効経路の確認対象は以下に固定する。

- Keycloak session / refresh token の失効
- OpenBao の JWT 認証経路での token issuance 停止
- cert-manager 発行証明書の revoke 確認
- Kubernetes API での RoleBinding subject 解決失敗確認
- Backstage / Grafana / Argo CD の SSO ログイン失敗確認

## 図表

```
[Identity 7 原則の収斂先]
  人間 ID     ─┐
  ワーク ID    ─┤
  シークレット  ─┤→ Keycloak 1 アカウント無効化 → 全経路 revoke
  証明書       ─┤            ↑
  mTLS        ─┘            GameDay で四半期ごとに実効性検証
```

各原則と NFR / ADR の対応軸は以下である。

- 原則 1（人間 ID）: NFR-E-AC-001 / 003 / 005 × ADR-SEC-001
- 原則 2（ワークロード ID）: NFR-G-AC-001 × ADR-SEC-003 / ADR-0001
- 原則 3（退職時 revoke）: NFR-E-AC-005 × 本章全体の統合
- 原則 4（OpenBao）: NFR-E-AC-004 / NFR-E-MON-002 × ADR-SEC-002
- 原則 5（cert-manager）: NFR-E-ENC-001 / 002 × ADR-SEC-003 の upstream
- 原則 6（Istio Ambient mTLS）: NFR-E-ENC-002 × ADR-0001
- 原則 7（GameDay）: NFR-E-AC-005 受け入れ基準の継続検証

詳細な ID フロー図は [img/Identity原則俯瞰.drawio](../img/85_Identity4軸統合.drawio)（リリース時点点で svg 作成）を参照。

## 対応 IMP-SEC ID

本ファイルで採番する原則レベル ID は以下とする。

- `IMP-SEC-POL-001` : 人間 ID は Keycloak に一本化
- `IMP-SEC-POL-002` : ワークロード ID は SPIFFE/SPIRE SVID
- `IMP-SEC-POL-003` : 退職時は Keycloak 1 アカウント無効化で全経路 revoke
- `IMP-SEC-POL-004` : シークレットは OpenBao に一元化
- `IMP-SEC-POL-005` : 証明書は cert-manager 自動更新のみ
- `IMP-SEC-POL-006` : mTLS は Istio Ambient で全サービス間強制
- `IMP-SEC-POL-007` : 退職時 revoke GameDay の定期実施

## 対応 ADR / DS-SW-COMP / NFR

- ADR: [ADR-SEC-001](../../../02_構想設計/adr/ADR-SEC-001-keycloak.md)（Keycloak）/ [ADR-SEC-002](../../../02_構想設計/adr/ADR-SEC-002-openbao.md)（OpenBao）/ [ADR-SEC-003](../../../02_構想設計/adr/ADR-SEC-003-spiffe-spire.md)（SPIFFE/SPIRE）/ [ADR-0001](../../../02_構想設計/adr/ADR-0001-istio-ambient-vs-sidecar.md)（Istio Ambient mTLS）
- DS-SW-COMP: DS-SW-COMP-006（SECRET 運用形態）/ DS-SW-COMP-124（サイドカー統合）/ DS-SW-COMP-141（多層防御統括）
- NFR: NFR-E-AC-001（JWT 強制）/ NFR-E-AC-003（tenant_id 検証）/ NFR-E-AC-004（Secret 最小権限）/ NFR-E-AC-005（MFA / 退職時 revoke）/ NFR-E-ENC-001（保管暗号化）/ NFR-E-ENC-002（転送暗号化）/ NFR-E-MON-002（Secret 取得監査）/ NFR-G-AC-001（最小権限）

## 関連章

- `80_サプライチェーン設計/` : cosign keyless 署名における OIDC 連携（原則 1 と接続）
- `90_ガバナンス設計/` : Kyverno ポリシーによる mTLS / OIDC 強制（原則 6 と接続）
- `60_観測性設計/` : 認証系 SLI / 証明書期限 SLI（原則 5 と接続）
- `95_DXメトリクス/` : GameDay の成否 Scorecards 反映（原則 7 と接続）
