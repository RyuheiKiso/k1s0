---
id: detail.tier3.tier3_enforcement
axis: tier3
phase: detail
kind: enforcement
status: draft
depends_on:
  - arch.tier3.tier3_index
  - arch.tier3.responsibility
  - arch.tier3.auth_multi_tenant
  - arch.tier3.observability
  - detail.tier2.tier2_enforcement
  - detail.tier1.tier1_enforcement
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_program_correctness_proof
---

# tier3 強制機構

## 一文方針
- tier3 facade 規律（tier1 / OSS 直接利用禁止 / 業務管理 API 利用禁止 / tenant_id 受付禁止 / 認可 cache 禁止 / PII 保管禁止 / 監査 emit 抑制経路禁止 / 翻訳キー未定義検出 / 公開 type 独自生成禁止 / contract test 必須実行 / a11y 自動検査 / CSP セキュリティヘッダ宣言 / 内部レジストリ唯一化 / サプライチェーン）は CI で 13 層多重防御で物理 enforce、各層は独立に動作し、ある層が破られても別層で阻止する。

## 13 層 defense-in-depth

### 層 1: tier1 / OSS 直接 import 禁止
- Rust: cargo-deny の `[bans]` で tier1 / OSS のクライアント crate を deny。tier2 SDK crate のみ許可
- C# (.NET 8+): BannedApiAnalyzers の `BannedSymbols.txt` で tier1 / OSS の名前空間を禁止。Central Package Management で tier2 NuGet のみ参照許可
- Go: golangci-lint の `depguard` で tier1 / OSS パッケージの import を deny。tier2 Go module のみ許可
- TypeScript: `eslint-plugin-import` の `no-restricted-imports` と `eslint-plugin-boundaries` で tier1 / OSS パッケージを禁止
- tier3 が独自に OSS をラップする経路も同枠で禁止

### 層 2: 業務管理 API import 禁止
- tier3 業務 UI が業務管理 API（`/admin/v1/...` / `tier2.<pack>.admin.v1`）を import する経路を CI で禁止
- C#: 業務管理 NuGet パッケージを参照不可（Central Package Management の許可リストから除外）
- TypeScript: `eslint-plugin-import` の `no-restricted-imports` で `/admin/` パッケージを禁止
- Rust / Go: それぞれ cargo-deny / depguard で admin crate / module を deny

### 層 3: tenant_id 受付禁止
- tier3 公開 API / form 入力で `tenant_id` を引数として受け付ける経路を禁止
- `tenant_id` は Authorization context から取得する経路のみ許容
- CI 検査:
  - public な type / form schema に `tenant_id` 名のフィールドを置いていないか走査
  - URL routing / クエリパラメータに `tenant_id` を含まないことを検査

### 層 4: 認可 cache 禁止
- クライアント側で `AuthorizationDenied` を cache する経路を禁止
- 認可結果は tier2 が一次情報として保持。tier3 はそれを反映するだけで、独自 cache を持たない
- CI 検査:
  - localStorage / IndexedDB / cookie への認可結果保存を静的解析で検出
  - service worker の cache strategy に認可エンドポイントが含まれていないかを検査

### 層 5: PII 保管経路の禁止
- PII を localStorage / IndexedDB / cookie に保管する経路を禁止
- 静的解析:
  - PII フィールド（生成型のアノテーション情報）を localStorage / IndexedDB に書き込む箇所を検出
  - cookie 経由で PII を保持する箇所を検出
- 例外: オフライン業務での Outbox 一時保存は IndexedDB の暗号化領域で許容するが、保管期間を制限

### 層 6: 監査 emit 抑制経路の禁止
- tier2 SDK の業務 API を経由せずバックエンドを呼出する経路を禁止
- tier2 が業務 API 経由の Domain Event emit / 監査エントリ emit を強制している以上、tier3 が raw HTTP / raw gRPC で tier1 backend を呼ぶことを防ぐ
- CI 検査:
  - fetch / axios / gRPC client で tier1 endpoint を直接叩く箇所を検出（tier1 endpoint URL パターンの deny list）
  - tier2 SDK の業務 API ラッパを介していない通信は fail

### 層 7: 翻訳キーの未定義検出
- i18n の翻訳キーが未定義のまま参照されていないかを CI で検出
- 静的解析:
  - `t('xxx')` 形式の参照を抽出し、locale ファイルとの差分を検査
  - 未定義キー / 未使用キーを CI で fail
- pseudolocalization での文字長変動検出も CI で実施

### 層 8: 公開 type の独自生成禁止
- tier3 は tier2 .proto 由来の生成型のみを使用。独自型の宣言は禁止
- CI 検査:
  - tier3 リポジトリ内に `.proto` ファイルを置かないことを検査（tier2 が単一の真）
  - 業務 entity / 業務 event を独自に再宣言する type が無いかを静的解析で検出

### 層 9: contract test の必須実行
- tier2 が提供する contract test suite を tier3 リポジトリの CI で必須実行
- skip / 改変は CI で fail:
  - test runner の skip フラグの使用を検出
  - contract test の改変（assertion の弱体化）を CI で diff 検査
- tier2 minor version up 時、新版 contract test での再実行を CI に組み込む

### 層 10: ブラウザ要件 / a11y 要件の自動検査
- axe-core を CI に組み込み、WCAG 2.1 AA 違反は merge 阻止
- Web Vitals（LCP / INP / CLS）を Lighthouse CI で測定し、閾値違反は merge 阻止
- サポート対象ブラウザでの動作確認を Playwright クロスブラウザで CI 実行

### 層 11: CSP / セキュリティヘッダの宣言と検査
- CSP / HSTS / X-Content-Type-Options / Referrer-Policy / Permissions-Policy を宣言
- CI 検査:
  - 配信される HTML レスポンスにセキュリティヘッダが含まれていることを E2E で検査
  - `unsafe-inline` / `unsafe-eval` が CSP に含まれていないことを CI で fail
  - SRI（subresource integrity）が第三者 CDN リソースに付与されていることを検査

### 層 12: 内部レジストリ唯一化
- tier1 / tier2 と同枠で、tier3 配布物 / 依存も内部レジストリを唯一の取得元に固定
  - npm: 内部 Verdaccio
  - NuGet: 内部 feed のみ `nuget.config` に登録
  - cargo: 内部 cargo registry を `[source.crates-io]` で置換
  - Go: 内部 Athens module proxy を `GOPROXY` に固定
  - OCI image: exe 配布の場合の image は内部 Harbor に固定
- 内部レジストリには tier1 / tier2 / tier3 の承認済み配布物のみを公開

### 層 13: サプライチェーン
- Cosign / Trivy / SBOM を tier3 配布物にも適用
- SBOM 形式は CycloneDX。OCI image / package metadata に attest
- Trivy で依存パッケージの CVE を日次スキャン

## Backstage Software Template での組込
- tier3 リポジトリ生成時点で、上記すべての強制機構が組み込まれる
- tier3 担当は強制機構の新設作業を行わない
- 各層は defense-in-depth として独立に動作する。ある層が破られても別層で阻止する

## CI 不変条件（tier3 全体、merge 不可）
- 整合 1: 全 13 層が CI で実行される
- 整合 2: 各層の検査結果が green（skip / 改変は fail）
- 整合 3: layer 4 layer reducer の override がゼロ
- 整合 4: contract test 改変 / skip がゼロ
- 整合 5: WCAG 2.1 AA / Web Vitals 閾値違反がゼロ
- 整合 6: CSP / セキュリティヘッダの runtime 違反がゼロ
- 整合 7: dead spec 検出（参照されない方針 / 拡張点 / scenario は CI fail）

## 1.0.0 ship blocker
- 全 13 層 green
- クライアント状態適合仕様 11 整合 1〜9 green
- BFF auth-edge / Tauri Companion sidecar の cross-cutting 全 green

## 採用しない強制機構
- 文章のみの規律（必ず CI / lint / build artifact / Kyverno に物理転写）
- OSS 直接 import の例外承認 path
- tier3 公開 API での dev mode runtime check 緩和

## 至高路線における立ち位置
- tier3 facade 規律は文章に依存しない。13 層多重防御で物理 enforce
- 1.0.0 release 時に 7 不変条件 + 13 層が全 green であることを物理 ship blocker

## 関連参照
- [tier3 設計方針 index](../../03_概要設計/04_tier3設計方針/README.md)
- [責務](../../03_概要設計/04_tier3設計方針/01_責務.md)
- [認証認可マルチテナント](../../03_概要設計/04_tier3設計方針/07_認証認可マルチテナント.md)
- [クライアント状態適合仕様](../01_適合仕様/11_クライアント状態適合仕様.md)
- [BFF auth-edge](../03_クロスカッティング適合仕様/06_BFF_auth_edge.md)
- [Tauri Companion sidecar](../03_クロスカッティング適合仕様/07_Tauri_companion_sidecar.md)
- [tier2 強制機構](02_tier2強制機構.md)
- [tier1 強制機構](01_tier1強制機構.md)
