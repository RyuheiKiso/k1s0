# 99. 索引 / 40. Backstage catalog 対応 / 01. Backstage catalog 対応表

本ファイルは k1s0 における Backstage の Software Catalog（`catalog-info.yaml`）と本章で採番された IMP-\* ID との対応方針を Phase 0 スケルトンとして確定する。Backstage はサービス運用の第一表示面であり（ADR-BS-001）、Phase 1a までに全 tier1 コンポーネント + examples が Backstage に登録される想定の下、IMP-\* ID を `annotations` で明示的に紐付ける形式を採用する。

## Phase 0 の位置付け

本ファイルは Phase 0 時点ではスケルトン（骨格とマッピング方針の確定）にとどまる。Phase 1a で Backstage 本体のデプロイが確定した段階で、全 Component / System / API 単位の実 ID を列挙してマトリクスに埋める運用に移行する。Phase 0 で固定するのは、以下の 4 点である。

第一に `catalog-info.yaml` の annotations スキーマ、第二に Scaffold CLI が生成する雛形の annotation プレースホルダ、第三に IMP-\* ID の粒度と Backstage kind（Component / System / API / Resource / Domain）の対応方針、第四に CI の整合チェック（Phase 1a で `tools/ci/trace-check/` に実装）。この 4 点を Phase 0 で確定しておくことで、Phase 1a 立ち上げ時に「Backstage 導入後に ID 対応を整備する」という典型的な遅延を回避する。

## annotations スキーマ

`catalog-info.yaml` の `metadata.annotations` に以下の k1s0 固有 annotation を追加する。Backstage 標準の annotations（`backstage.io/techdocs-ref` 等）と共存する。

- `k1s0.io/imp-ids`: カンマ区切りの IMP-\* ID 列挙（例: `IMP-SEC-KC-010,IMP-SEC-SP-020,IMP-OBS-SLO-040`）
- `k1s0.io/adr-ids`: カンマ区切りの ADR ID 列挙（例: `ADR-SEC-001,ADR-BS-001`）
- `k1s0.io/ds-sw-comp-ids`: カンマ区切りの DS-SW-COMP ID 列挙（例: `DS-SW-COMP-124,DS-SW-COMP-135`）
- `k1s0.io/nfr-ids`: カンマ区切りの NFR ID 列挙（例: `NFR-E-AC-001,NFR-I-SLO-001`）
- `k1s0.io/phase`: Phase 識別子（例: `0`, `1a`, `1b`, `2`）

複数 ID を列挙可能とする理由は、1 Component が複数の IMP-\* / ADR / NFR にまたがるのが通常であるため（IMP-TRACE-POL-003 の原子性は IMP 側の判断粒度で、Component は複数判断の集合である）。Backstage の plugin（`@k1s0/plugin-trace-matrix`）で各 annotation を parse し、IMP-\* 台帳への逆参照を UI で表示する設計を Phase 1a で追加する。

## Backstage kind と IMP-\* 粒度の対応

Backstage の entity kind は Component / System / API / Resource / Domain / Group / User / Location の 8 種類が基本である。k1s0 での採用 kind と IMP-\* 粒度の対応方針を以下に固定する。

- **Component**: tier1 / tier2 / tier3 の個別サービス（例: tier1 Decision API、tier2 Order Service）。典型的に 5〜15 個の IMP-\* が紐付く。主に IMP-BUILD / IMP-DEV / IMP-OBS / IMP-SEC / IMP-REL 系が該当する。
- **System**: tier 全体または横断論理面（例: tier1 全体、観測性スタック）。典型的に 20〜40 個の IMP-\* が紐付く。DS-SW-COMP の tier1 俯瞰層（001〜019）と対応する。
- **API**: tier1 公開 11 API、tier2 内部 API、SDK 公開 API。典型的に IMP-CODEGEN / IMP-OBS-SLO 系が紐付く。
- **Resource**: Harbor project、Keycloak realm、OpenBao mount point 等のインフラリソース。IMP-CI-HAR / IMP-SEC-KC 系が紐付く。
- **Domain**: ビジネス領域（JTC 業務ドメイン）。実装 IMP とは直接紐付けず、システム集合のグルーピング用途。

採番粒度が Backstage の entity 粒度と一致するかどうかは、1 Component = 1 Group の観点で事前にレビューされる。Phase 1a 立ち上げ時の entity 設計 ADR（ADR-BS-002 として Phase 1a で起票予定）で最終確定する。

## Scaffold CLI の雛形統合

`20_コード生成設計/30_Scaffold_CLI/01_Scaffold_CLI設計.md` の IMP-CODEGEN-SCF-033（`catalog-info.yaml` 自動生成）および IMP-CODEGEN-POL-006（catalog-info.yaml 必須生成）に従い、Scaffold CLI が生成する全テンプレートに以下のプレースホルダを含める。

```yaml
# 生成器が以下を {{ ... }} プレースホルダで置換する
metadata:
  annotations:
    k1s0.io/imp-ids: "{{ imp_ids | default('') }}"
    k1s0.io/adr-ids: "{{ adr_ids | default('') }}"
    k1s0.io/phase: "{{ phase | default('1a') }}"
```

テンプレート時点で確定している IMP-\* は `template.yaml` の `spec.parameters` 既定値で埋め、Scaffold 実行時に開発者が不明な値を入力するのは禁止する（誤った ID の混入を構造的に防ぐ）。後付けで追加される IMP-\* は PR レビュー時に Service Owner が annotation を追記する運用とする（IMP-CODEGEN-SCF-034 の SRE + Security 二重承認で確認される）。

## Phase 0 暫定マッピング方針

Phase 0 時点では Backstage 本体が未デプロイのため、実 `catalog-info.yaml` は存在しない。代替として `examples/` 配下の 4 つの Golden Path example（IMP-DEV-GP-020）が持つべき annotations を以下に列挙する。Phase 1a で Backstage にインポート後、本表と整合する。

| examples 配下 | 想定 kind | 主要 IMP-\* annotations | 主要 ADR annotations |
|---|---|---|---|
| `examples/tier2-minimal/` | Component | `IMP-DEV-GP-020`, `IMP-DEV-GP-022`, `IMP-DEV-GP-023`, `IMP-BUILD-GM-020`, `IMP-SEC-POL-001` | `ADR-DEV-001`, `ADR-TIER1-003`, `ADR-BS-001` |
| `examples/tier2-decision/` | Component | `IMP-DEV-GP-020`, `IMP-DEV-GP-022`, `IMP-DEV-GP-023`, `IMP-BUILD-GM-020` + Phase 1a で `IMP-DEV-GP-025`（decision 拡張） | `ADR-DEV-001`, `ADR-RULE-001`, `ADR-BS-001` |
| `examples/tier3-web-minimal/` | Component | `IMP-DEV-GP-020`, `IMP-DEV-GP-023`, `IMP-DEV-GP-024` | `ADR-DEV-001`, `ADR-BS-001` |
| `examples/tier3-bff-minimal/` | Component | `IMP-DEV-GP-020`, `IMP-DEV-GP-023`, `IMP-BUILD-GM-020` | `ADR-DEV-001`, `ADR-BS-001` |

tier1 公開 11 API は Phase 1a で API kind として登録される。その際 `k1s0.io/imp-ids` には `IMP-OBS-SLO-040〜047`、`IMP-CODEGEN-BUF-010〜017`、`IMP-SEC-POL-001`、`IMP-SEC-KC-010〜022` が annotation される見込みである。

## CI 整合チェック（Phase 1a で実装）

`tools/ci/trace-check/` に以下 3 種類のチェックを Phase 1a で実装する。IMP-TRACE-POL-005（双方向リンク）の CI 実装に相当する。

1. **annotations 内 IMP-\* ID の実在チェック**: `catalog-info.yaml` の `k1s0.io/imp-ids` に列挙された ID が本章の `00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md` に存在するか検証する。不在なら PR を失敗させる。
2. **IMP-\* ID 逆引きの網羅性チェック**: 採番済 IMP-\* ID が少なくとも 1 つ以上の `catalog-info.yaml` から参照されているか検証する。未参照の ID は warning を出す（error にはしない。実装 ID は必ずしも実サービスに紐付かないため）。
3. **Phase 整合チェック**: `k1s0.io/phase` と `k1s0.io/imp-ids` に列挙された IMP-\* の Phase 値が矛盾しないか（例: Phase 0 の Component が Phase 1b のみで効く IMP を annotation していたら警告）。

これら 3 チェックは `tools/ci/trace-check/` の Rust 実装で Phase 1a 確定する。Phase 0 時点では本ファイルと IMP-\* 台帳の双方向整合を人手レビューで確認する運用にとどめる。

## Phase 1a の追補計画

Phase 1a 開始時に本ファイルは以下を追加する。

- 全 Component / System / API / Resource の実マッピング表（上記暫定 4 Component → 全量）
- `catalog-info.yaml` のスキーマ定義ファイル（`tools/codegen/schemas/catalog-info.schema.yaml`）
- Backstage plugin `@k1s0/plugin-trace-matrix` の動作仕様
- ADR-BS-002（Backstage entity 粒度確定）の起票とリンク
- `tools/ci/trace-check/` 実装の詳細仕様書

これら追補は本章の改訂履歴（`../90_改訂履歴/01_改訂履歴.md`）に Phase 1a 着手時点で追記する。

## 関連ファイル

- 本章の原則: [`../00_方針/01_索引運用原則.md`](../00_方針/01_索引運用原則.md)
- IMP-ID 台帳: [`../00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md`](../00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md)
- Scaffold CLI: [`../../20_コード生成設計/30_Scaffold_CLI/01_Scaffold_CLI設計.md`](../../20_コード生成設計/30_Scaffold_CLI/01_Scaffold_CLI設計.md)
- Golden Path: [`../../50_開発者体験設計/20_Golden_Path_examples/01_Golden_Path_examples.md`](../../50_開発者体験設計/20_Golden_Path_examples/01_Golden_Path_examples.md)
- ADR-BS-001: [`../../../02_構想設計/adr/ADR-BS-001-backstage.md`](../../../02_構想設計/adr/ADR-BS-001-backstage.md)
