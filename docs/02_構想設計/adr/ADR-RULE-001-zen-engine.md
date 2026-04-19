# ADR-RULE-001: ルールエンジンに ZEN Engine + JDM を採用

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: tier1 開発チーム / tier2 リードエンジニア / Product Council

## コンテキスト

tier1 Decision API（FR-T1-DECISION-001〜004）は、業務ルール（与信判定、価格決定、承認経路、PII 分類など）を宣言的に記述して評価するエンジンを必要とする。業務ルールをコードに埋め込むと、ルール変更のたびにデプロイが必要になり、業務担当者の自主改善サイクルが閉じる。

要件は以下の通り。

- **高速評価**: p99 < 50ms（シンプルルール）、p99 < 200ms（複雑ツリー）
- **可視化可能**: 業務担当者が GUI で編集できる JDM（JSON Decision Model）フォーマット
- **テスト可能**: ゴールデンケースで回帰テスト可能、Determinism 保証
- **Rust ネイティブ**: tier1 の Rust 領域で動作、GC 停止なし
- **評価トレース**: デバッグ・監査のために評価経路を返せる

従来の BRMS（Drools、IBM ODM 等）は Java 前提で重量、評価レイテンシが不利。軽量な代替は DMN（Decision Model and Notation）実装（OpenL Tablets、Camunda DMN 等）、または JDM 系（gorules の ZEN Engine）。

## 決定

**ルールエンジンは ZEN Engine（Rust、MIT）+ JDM フォーマットを採用する。**

- ZEN Engine 0.30+（Rust 実装、C FFI と Go/Python/Node バインディング提供）
- JDM（JSON Decision Model）を標準フォーマット
- ルールは tier1 PostgreSQL に versioning 付きで保管、Decision API 経由で取得・評価
- 評価トレース機能で「どのノードが true/false と判定されたか」を返却可能
- GUI エディタ（gorules の Editor）でビジネス担当者が編集
- ゴールデンケース回帰テストを CI/CD に必須化（FMEA RPN 126 対策）

## 検討した選択肢

### 選択肢 A: ZEN Engine + JDM（採用）

- 概要: Rust 製高速 JDM 評価器、MIT ライセンス
- メリット:
  - Rust ネイティブで評価レイテンシがマイクロ秒オーダー
  - JDM は JSON ベースで Git 管理容易、Diff レビュー可能
  - 評価トレース機能で業務担当者がデバッグ可能
  - GUI エディタ（gorules Editor）で業務担当者が編集
  - MIT で商用利用自由
- デメリット:
  - ZEN Engine 自体が比較的新しい（2023〜）、エコシステムが DMN より薄い
  - JDM 記述規約を JTC 社内で標準化する必要

### 選択肢 B: Camunda DMN (Camunda Platform)

- 概要: BPMN/DMN の業界デファクト
- メリット: DMN 標準（OMG 標準）、業界実績最大
- デメリット:
  - Java ベース、評価レイテンシが Rust より不利
  - DMN の学習曲線が急、JDM より冗長

### 選択肢 C: Drools

- 概要: Red Hat 傘下、Java ベース BRMS
- メリット: 長年の実績、機能豊富
- デメリット:
  - 重量級、評価レイテンシで不利
  - Drools Rule Language の学習コスト大

### 選択肢 D: OpenL Tablets

- 概要: Excel ライクな DSL、Apache 2.0
- メリット: 業務担当者がExcel 感覚で記述
- デメリット:
  - Java ベース
  - コミュニティ規模が小さい

### 選択肢 E: 独自 DSL を Rust で実装

- 概要: PEST / nom で独自パーサを書く
- メリット: 完全制御
- デメリット:
  - 実装工数が膨大
  - 業務担当者の学習資産を社外と共有できない

## 帰結

### ポジティブな帰結

- Rust ネイティブで評価レイテンシが他選択肢を圧倒
- JDM は JSON で Diff 可能、Git レビュー・承認フローが回る
- 業務担当者が GUI エディタで自主改善、tier2 開発者の介入不要
- ゴールデンケーステストで回帰リスクを最小化
- MIT ライセンスで商用・改変自由

### ネガティブな帰結

- ZEN Engine のバグ（RPN 126）に対する運用対策が必須、評価トレース・Canary リリース
- JDM 記述規約（命名、深さ制限、循環参照検出）を社内標準化
- ZEN Engine メジャーアップデート時の JDM 互換性検証
- 業務担当者への GUI エディタ教育（BC-TRN）

## 実装タスク

- ZEN Engine の Rust crate バージョン固定
- JDM ストレージ（PostgreSQL）のスキーマ設計と versioning ポリシー
- 評価トレース機能を Decision API の include_trace オプションで公開（IDL 定義）
- ゴールデンケーステストを CI/CD 必須化（PR ブロック条件）
- GUI エディタ（gorules Editor OSS 版）を Backstage に埋込み
- 業務担当者向けトレーニング教材（BC-TRN）に JDM 基礎を収録
- Canary リリース時の評価結果比較（旧版・新版で同じ入力に対する出力差分）

## 参考文献

- ZEN Engine: github.com/gorules/zen
- JDM 仕様: gorules.io/docs/developers/core
- OMG DMN 1.5 仕様（比較用）
- Camunda DMN（比較用）
