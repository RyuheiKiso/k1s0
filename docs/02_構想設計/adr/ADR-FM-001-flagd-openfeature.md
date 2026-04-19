# ADR-FM-001: Feature Flag に flagd / OpenFeature を採用

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: tier1 開発チーム / tier2 リードエンジニア / Product Council

## コンテキスト

tier1 Feature API（FR-T1-FEATURE-001〜004）は、Release/Experiment/Ops/Permission の 4 種類の Feature Flag を提供する必要がある。LaunchDarkly のような商用 SaaS は高機能だが、年間数千万円の費用とデータ外部送信が発生する。オンプレミス制約下では OSS 選定が必須。

候補は flagd (OpenFeature)、Unleash、GrowthBook、Flipt、Split.io OSS 等。

## 決定

**Feature Management は OpenFeature 仕様 + flagd（Reference Implementation、Apache 2.0、CNCF Incubating）を採用する。**

- OpenFeature SDK（各言語で提供）でアプリ側を統一
- flagd をサイドカーまたは中央デーモンで配置
- フラグ定義は JSON/YAML、Git で管理（GitOps）
- flagd Provider 経由で tier1 Feature API に接続、tier2/tier3 からは k1s0 API で呼出
- Release / Experiment / Ops / Permission の 4 種別は命名規則 + metadata で区別
- targetingKey（user_id 等）でセグメント分割

将来商用 SaaS（LaunchDarkly 等）へ移行する場合、OpenFeature SDK は Provider 差し替えのみで対応可能。これによりベンダーロックを回避しつつ現行 OSS を利用できる。

## 検討した選択肢

### 選択肢 A: OpenFeature + flagd（採用）

- 概要: CNCF Incubating、ベンダー中立標準
- メリット:
  - OpenFeature は業界標準化進行中、LaunchDarkly 等も Provider 提供
  - flagd は Reference Implementation でシンプル、軽量
  - フラグ定義を Git で管理、GitOps 親和性高
  - ベンダーロックなし、商用 SaaS への切替容易
- デメリット:
  - GUI 管理 UI が簡素（flagd-ui は基本機能のみ）
  - Experiment 分析機能は外部ツール（GrowthBook 等）との組合せ

### 選択肢 B: Unleash

- 概要: Open Source Feature Flag（Apache 2.0）
- メリット: UI が洗練、成熟した OSS
- デメリット:
  - ベンダー中立 API（OpenFeature）対応は後発
  - 自社ホスト版と SaaS 版のコード差異

### 選択肢 C: GrowthBook

- 概要: A/B テスト特化 OSS
- メリット: Experiment 分析が本命
- デメリット:
  - Release Flag 用途としては Unleash / flagd より機能薄
  - 統計計算エンジンが主軸

### 選択肢 D: Flipt

- 概要: Go 製軽量 Feature Flag（MIT）
- メリット: シンプル、gRPC API
- デメリット:
  - OpenFeature 対応は後発
  - コミュニティ規模が小さい

### 選択肢 E: LaunchDarkly（商用 SaaS）

- 概要: 業界デファクト商用
- メリット: 機能最大、UX 洗練
- デメリット:
  - オンプレ制約で SaaS 選べず
  - 商用ライセンス費用（年間数千万円）

## 帰結

### ポジティブな帰結

- ベンダー中立、将来の商用移行が Provider 差し替えのみ
- Git ベースで Feature Flag 変更の監査証跡が残る
- 4 種別 Flag の命名規則で用途分離、DX-FM-004 段階ロールアウトを実現
- 軽量で評価レイテンシ < 10ms 達成可能

### ネガティブな帰結

- GUI 管理 UI が簡素、ビジネス担当者向けには Backstage 内にカスタム UI を構築予定
- Experiment 統計分析は別ツール（GrowthBook / 独自）と連携必要
- flagd の Incubating ステータス、CNCF Graduated まで継続監視

## 実装タスク

- flagd Helm Chart バージョン固定、Argo CD 管理
- フラグ定義を Git で管理、PR レビュー必須
- Release / Experiment / Ops / Permission の 4 種別命名規則策定
- OpenFeature SDK を全言語で統一（Go / Rust / TypeScript / C# / Java / Python）
- Backstage 内に Feature Flag 管理 UI（カスタム）
- 段階ロールアウト（1% → 10% → 50% → 100%）の DX-FM-004 を flagd で実装

## 参考文献

- OpenFeature 公式: openfeature.dev
- flagd: flagd.dev
- CNCF Incubating Projects
- Feature Flag Best Practices (Martin Fowler)
