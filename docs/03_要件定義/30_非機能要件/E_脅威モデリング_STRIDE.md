# E. 脅威モデリング（STRIDE + DFD + Attack Tree）

本書はセキュリティ要件（[E_セキュリティ.md](E_セキュリティ.md)）の根拠となる脅威分析を STRIDE モデルで体系化する。データフロー図（DFD）でトラストバウンダリを可視化し、境界を跨ぐエッジごとに脅威カテゴリを当て、各脅威に対する緩和策を要件 ID と紐付ける。Attack Tree では代表的な攻撃シナリオを分解し、最弱リンクを特定する。

## なぜ脅威モデリングが必要か

セキュリティ要件は「ISO 27001 に準拠」「TLS 1.3 を採用」といった手段の宣言に陥りやすいが、**なぜその手段が必要か** を辿ると脅威モデルに行き着く。脅威モデルがないまま要件を書くと、1）必要以上の実装（過剰防御）、2）本来必要な対策の漏れ、3）監査や脆弱性診断の質疑で根拠を説明できない、の 3 つが発生する。

本書は **Microsoft STRIDE**（Spoofing / Tampering / Repudiation / Information Disclosure / Denial of Service / Elevation of Privilege）を分類軸に採用する。STRIDE は Threat Modeling 分野の業界標準で、Adam Shostack の Threat Modeling: Designing for Security でも一次軸として使われる。

## DFD とトラストバウンダリ

![k1s0 DFD + Trust Boundary](img/dfd_tier1_trust_boundary.svg)

図は k1s0 の主要データフローを 4 つの Trust Boundary（TB）で区切っている。

- **TB0: Internet / 社外**: 外部ユーザ、レガシー資産、管理者、攻撃者の起点
- **TB1: DMZ**: Ingress Gateway、WAF、Keycloak、API Gateway を配置。外部トラフィックの最初の境界
- **TB2: Kubernetes Cluster**: tier1/tier2/tier3、Istio Ambient、SPIRE、OTel、Temporal、flagd 等の実行面。mTLS 必須
- **TB3: Data Plane**: PostgreSQL、Kafka、MinIO、OpenBao、Valkey 等の永続層

境界を跨ぐエッジが攻撃面になるため、各エッジに対して STRIDE 6 カテゴリを網羅的に評価する。

## STRIDE 脅威分析

以下は主要なエッジごとの脅威と緩和策。要件 ID（NFR-E-\*）は [E_セキュリティ.md](E_セキュリティ.md) を参照。

### T0 → T1: 外部ユーザ → Ingress Gateway

- **S (Spoofing)**: 正規ユーザへのなりすまし。緩和: Keycloak OIDC、MFA（NFR-E-AUTH-002）、セッション短寿命化
- **T (Tampering)**: HTTPS 中間者改竄。緩和: TLS 1.3、HSTS、証明書ピン（NFR-E-CRYPT-001）
- **R (Repudiation)**: 操作の否認。緩和: Audit WORM（NFR-E-AUD-001）
- **I (Information Disclosure)**: セッション情報漏洩。緩和: HttpOnly/Secure Cookie、SameSite=Strict
- **D (DoS)**: L7 DDoS、Slowloris 等。緩和: Envoy Rate Limit、Cloudflare 等の前段保護（NFR-E-NET-003）
- **E (Elevation)**: 認証後の権限昇格。緩和: OIDC claims 検証、tier1 で tenant_id 強制付与

### T0 → T1: レガシー .NET 資産 → Sidecar/API Gateway

- **S**: VPN クライアント偽装。緩和: SPIFFE ID mTLS、Workload Identity（ADR-SEC-003）
- **T**: 転送データ改竄。緩和: mTLS、MAC 検証
- **I**: 平文転送漏洩。緩和: IPSec + mTLS、TLS 1.3 必須
- **D**: レガシー起因の大量同時接続。緩和: API Gateway 接続数上限、Circuit Breaker

### T0 → T1: 管理者 → Backstage / kubectl

- **S**: 管理者アカウント盗用。緩和: ハードウェアトークン MFA、特権 SSO、短寿命トークン
- **T**: kubectl 操作の改竄。緩和: kubectl context ログ、Kyverno admission control
- **R**: 特権操作の否認。緩和: kube-apiserver audit ログ、Audit API WORM 転送
- **I**: etcd 秘匿情報閲覧。緩和: etcd 暗号化、管理者権限最小化（RBAC）
- **E**: サービスアカウント経由の cluster-admin 奪取。緩和: SPIRE/SPIFFE、RBAC 最小権限、Kyverno ポリシー

### T1 → T2: Ingress/Gateway → tier3/tier2

- **S**: 内部サービスなりすまし。緩和: SPIFFE ID + mTLS（ADR-SEC-003）
- **T**: リクエスト Body 改竄。緩和: mTLS、業務レベル署名（重要決定はデジタル署名）
- **I**: gRPC メタデータ漏洩。緩和: Pod 間 mTLS、ログマスキング
- **D**: 内部サービスの過剰呼出。緩和: Istio Rate Limit、Circuit Breaker

### T2 → T2: tier3 → tier2 → tier1 Dapr

- **S**: 不正 Pod が SPIFFE ID 詐称。緩和: SPIRE Agent + Node Attestor（k8s_sat / k8s_psat）
- **T**: 内部 gRPC 改竄。緩和: mTLS 必須（ztunnel）
- **R**: tier2 操作の否認。緩和: Audit API 自動書込（trace_id 付与）
- **I**: トレース / ログの PII 漏洩。緩和: Pii API Masking（ADR-0001 / G_データ保護とプライバシー.md）
- **E**: tier1 API 権限昇格。緩和: Dapr Access Control、tier1 で tenant_id 強制

### T2 → T3: tier1 Dapr → PG / Kafka / MinIO / OpenBao / Valkey

- **S**: Dapr 偽装による DB アクセス。緩和: SPIFFE ID mTLS、DB 側で Client Cert 検証
- **T**: DB 書込み改竄。緩和: PG RLS、AuditLog hash_chain（改竄検出）
- **R**: データ削除の否認。緩和: AuditLog WORM、hash_chain、WAL アーカイブ
- **I**: DB dump 漏洩。緩和: 保存時暗号化（TDE）、MinIO SSE-KMS、OpenBao Transit
- **D**: 大量クエリで DB 枯渇。緩和: Connection Pool 上限、Slow Query 検知
- **E**: DBA 権限の不正利用。緩和: OpenBao Dynamic Secret（短寿命 DB ロール）、特権操作 Audit

## Attack Tree: 「個人情報 1 万件を持ち出し」

代表的な攻撃シナリオを分解し、最弱リンクを特定する例。

```
Goal: 個人情報 1 万件持ち出し
├── A1: 正規ユーザ権限で SELECT
│   ├── A1-1: 管理者アカウント盗用（フィッシング）
│   │   └── 緩和: MFA、短寿命セッション、異常ログイン検知
│   └── A1-2: Insider による業務目的外閲覧
│       └── 緩和: Audit WORM、DLP、定期監査
├── A2: DB 直接接続（内部）
│   ├── A2-1: Pod 侵害後に DB 接続情報入手
│   │   └── 緩和: OpenBao Dynamic Secret、Secret は環境変数禁止
│   └── A2-2: etcd 漏洩から Secret 入手
│       └── 緩和: etcd 暗号化、etcd アクセス最小権限
├── A3: バックアップ窃取
│   ├── A3-1: MinIO バケット誤公開
│   │   └── 緩和: MinIO ポリシー IaC 管理、Public 拒否 Kyverno ポリシー
│   └── A3-2: WAL アーカイブ保管先侵害
│       └── 緩和: MinIO 側暗号化、WORM モード
└── A4: サプライチェーン侵害
    ├── A4-1: 悪意ある依存ライブラリ
    │   └── 緩和: SBOM、Sigstore 署名、Kyverno で未署名拒否
    └── A4-2: 悪意あるコンテナイメージ
        └── 緩和: Cosign 署名必須、Trivy スキャン、Base Image 限定
```

最弱リンクは A1-1（管理者アカウント盗用）と A4-1（依存ライブラリ侵害）。MFA 強制とサプライチェーン防御（Sigstore + Kyverno）を最優先で投資対象とする。

## 脅威分析の運用サイクル

脅威モデルは設計時に一度書いて終わりではない。以下のサイクルで継続更新する。

- **設計変更時**: 新しいコンポーネント追加、データフロー変更時に DFD を更新し、STRIDE 再評価
- **四半期レビュー**: TB 境界の妥当性、緩和策の実装状況を SRE + セキュリティチームで棚卸し
- **インシデント後**: 実際のインシデントが脅威モデルに含まれていたかを照合、含まれていなければモデル更新
- **ペネトレーションテスト前**: スコープ策定の入力として DFD と Attack Tree を提供

## 関連ドキュメント

- [E_セキュリティ.md](E_セキュリティ.md): 要件本体（NFR-E-\*）
- [G_データ保護とプライバシー.md](G_データ保護とプライバシー.md): PII Masking、法令根拠
- [H_アーティファクト完全性とコンプライアンス.md](H_アーティファクト完全性とコンプライアンス.md): SBOM、Sigstore、Kyverno
- [40_運用ライフサイクル/06_FMEA分析.md](../40_運用ライフサイクル/06_FMEA分析.md): 故障モード（セキュリティ起因を含む）
- ADR-SEC-001（Keycloak）、ADR-SEC-002（OpenBao）、ADR-SEC-003（SPIFFE/SPIRE）、ADR-CICD-003（Kyverno）
- Microsoft STRIDE Threat Modeling 資料
- Adam Shostack "Threat Modeling: Designing for Security"
