# /adr — ADR 起票

新しいアーキテクチャ決定記録（ADR）を `docs/02_構想設計/adr/` に起票する。

## 引数

$ARGUMENTS

自然な日本語で「何を決めるか」の短いタイトルを指定する。

例:
- `postgres に pgvector を使う`
- `observability プロバイダに grafana cloud を採用`
- `legacy api ゲートウェイを kong から envoy に移す`

## 作成手順

以下のステップを必ず順番どおりに実行すること。

### Step 0: 前提規約の読み込み

1. `.claude/skills/docs-delivery-principles/SKILL.md` を読み、納品品質原則を内面化する。
2. `.claude/skills/docs-adr-authoring/SKILL.md` を読み、ADR 書式規約を把握する。

### Step 1: 系列とドメイン判別

引数から対象ドメインを自動判別し、既存系列のどれに属するかを決める。

| 系列 | 主題 | キーワード例 |
|------|------|--------------|
| `ADR-BS-*` | Backstage | Backstage / IDP / developer portal |
| `ADR-CICD-*` | CI/CD | ArgoCD / Argo Rollouts / Kyverno / GitOps |
| `ADR-DATA-*` | データ基盤 | Postgres / Kafka / MinIO / Valkey / pgvector |
| `ADR-DEP-*` | 依存管理 | Renovate / SBOM / ライセンス |
| `ADR-DEV-*` | 開発者体験 | DevContainer / 雛形 / Golden Path |
| `ADR-DIR-*` | ディレクトリ構造 | ディレクトリ / モノレポ / sparse-checkout |
| `ADR-DX-*` | DX メトリクス | DORA / SPACE |
| `ADR-FM-*` | Feature Management | flagd / OpenFeature / feature flag |
| `ADR-MIG-*` | 移行 | legacy / .NET Framework / 移行 |
| `ADR-OBS-*` | 観測性 | Grafana / LGTM / OpenTelemetry / Tempo |
| `ADR-POL-*` | ポリシー | ポリシー / オーナーシップ / 二分所有 |
| `ADR-REL-*` | リリース | Progressive Delivery / Canary / Rollout |
| `ADR-RULE-*` | ルールエンジン | ZEN Engine / Temporal / ワークフロー |
| `ADR-SEC-*` | セキュリティ | Keycloak / OpenBao / SPIRE / SPIFFE |
| `ADR-000x` | 横断的決定 | どのドメインにも属さない根幹的な判断 |

判別が曖昧な場合、近い系列を複数候補として提示し、ユーザーに確認する。

### Step 2: 次番号の採番

`docs/02_構想設計/adr/` を走査し、該当系列の最大番号 + 1 を次番号とする。

```bash
ls docs/02_構想設計/adr/ | grep "ADR-<SERIES>-" | sort -V | tail -1
```

### Step 3: ファイル作成

`docs/00_format/ADR-xxxx.md` を雛形として、以下の命名で新規作成する:

```
docs/02_構想設計/adr/ADR-<SERIES>-<NNN>-<short-name>.md
```

- `<SERIES>`: Step 1 で判別した系列（BS / CICD / DATA / ...）
- `<NNN>`: Step 2 で決めた 3 桁ゼロ埋め番号
- `<short-name>`: 引数から英語ケバブケースで生成（例: `postgres-pgvector`）

### Step 4: ヘッダ項目の初期化

- ステータス: `Proposed`
- 起票日: 今日の日付（`YYYY-MM-DD` 形式）
- 決定日: 空欄（承認時に記入）
- 起票者: `kiso ryuhei`
- 関係者: 空欄（ユーザーが埋める）

### Step 5: 5 段構成の下書き

`docs-adr-authoring` Skill の規約に従い、以下 5 セクションの下書きを作成する:

1. **コンテキスト** — 何を悩んでいるか、制約・前提・トレードオフ
2. **決定** — 採用した選択肢の内容
3. **検討した選択肢** — **最低 3 件**、各選択肢にメリット / デメリット
4. **決定理由** — 「なぜ他の選択肢ではないのか」
5. **影響** — ポジティブ / ネガティブ / 移行・対応事項（3 サブセクション必須）

引数だけでは情報が不足する場合、ユーザーに追加質問する:
- 「現状の課題は何か」
- 「どの選択肢を検討したか（最低 3 件必要）」
- 「採用する選択肢は決まっているか」

### Step 6: 自己検証

`docs-adr-authoring` Skill の自己チェックリスト 7 項目を通す。全項目 OK を確認してからユーザーに提示する。

### Step 7: 索引更新の案内

以下のファイルを更新する必要があることをユーザーに伝える（本コマンドでは自動更新しない、レビュー後の手動更新推奨）:

- `docs/03_要件定義/00_要件定義方針/08_ADR索引.md`
- `docs/05_実装/99_索引/10_ADR対応表/`
- `docs/05_実装/99_索引/20_DS-SW-COMP対応表/`

## 記述ルール

- 本文は日本語
- 日付は `YYYY-MM-DD` 形式
- 図が必要な場合は `drawio-authoring` Skill を呼び、`img/` サブディレクトリに drawio と svg を配置
- アスキーアート禁止
