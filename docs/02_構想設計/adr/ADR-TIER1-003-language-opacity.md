# ADR-TIER1-003: tier2/tier3 から tier1 内部言語を不可視とする

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: tier1 開発チーム / tier2 リードエンジニア / tier3 リードエンジニア / Product Council

## コンテキスト

tier1 は Go + Rust のハイブリッド（ADR-TIER1-001）で実装されるが、tier2 ドメインアプリと tier3 UI/BFF の開発チームは、業務ロジックの記述に集中したい。tier1 の内部がどの言語で書かれているかは、tier2/tier3 開発者の知識として必要なものではなく、むしろ知らなくて良い情報である。

内部言語を tier2/tier3 が知ってしまうと以下の問題が発生する。

- **知識漏れ**: tier2 開発者が「Rust ならここで unwrap() できる」のような知識を混入させ、tier1 側の実装変更時に波及する
- **直呼出誘惑**: gRPC 抽象を迂回して tier1 の内部パッケージを直接 import する誘惑が生じ、境界が崩れる
- **言語変更の封鎖**: 将来、tier1 の Go 領域を Rust に統一する、あるいは Rust の一部を Go に戻すといった判断を取りにくくなる
- **採用制約**: tier2/tier3 開発者の採用条件に Go と Rust が必須となり、採用難易度が跳ね上がる

tier2/tier3 の開発チームは 20〜30 名規模に拡大予定。このチームが Go と Rust を学ぶ必要がない設計にすることで、採用プール（C#、Java、TypeScript、Python 等の開発者）を広く保つことが事業戦略上重要。

## 決定

**tier2/tier3 は tier1 の Protobuf IDL から生成されたクライアント SDK のみを使う。tier1 の内部実装言語・内部パッケージ構成・内部モジュール名は tier2/tier3 のコードベースに一切露出させない。**

具体的には以下を強制する。

1. tier1 の内部 Go/Rust パッケージは tier2/tier3 の go.mod / Cargo.toml から import できない（モノレポの workspace 境界で封鎖）
2. tier2/tier3 が使うのは buf で生成した言語別クライアント SDK（C#、Java、TypeScript、Go、Rust、Python 等）のみ
3. クライアント SDK は Nexus/Artifactory 内部レジストリから配布、tier2/tier3 チームに配布する最小単位
4. tier1 内部 API の破壊的変更は SemVer MAJOR、クライアント SDK の MAJOR バージョンアップとして配布し、tier2/tier3 側で段階移行

ドキュメント（Backstage Golden Path 等）でも「tier1 は何言語で書かれているか」を積極的に示さない。内部設計ドキュメント（02_構想設計/）でのみ明示する。

## 検討した選択肢

### 選択肢 A: クライアント SDK 経由のみで利用（採用）

- 概要: 上記の通り、IDL ベースの SDK 配布
- メリット:
  - tier1 実装言語と tier2/tier3 採用言語を完全分離
  - tier1 の内部言語変更が tier2/tier3 に影響しない
  - tier2/tier3 の採用プールを最大化
  - 契約違反（内部 API 直呼出）を技術的に不可能にできる
- デメリット:
  - SDK 配布インフラ（内部レジストリ）の整備が必要
  - tier2/tier3 開発者が tier1 の内部実装を学びにくい（デバッグ時に壁）

### 選択肢 B: モノレポで全言語から自由に import

- 概要: モノレポ内部でパッケージ自由アクセス
- メリット: 開発速度最大化
- デメリット:
  - tier1/tier2 の境界が技術的に崩れる
  - 言語変更時の影響範囲が読めない
  - tier2/tier3 の採用条件が tier1 言語に縛られる

### 選択肢 C: 言語非依存 REST/OpenAPI のみ公開

- 概要: 内部は gRPC でも、tier2/tier3 向けは REST
- メリット: クライアント SDK が不要
- デメリット:
  - 型安全性が低く、契約違反が実行時まで検知されない
  - gRPC の性能と可観測性の恩恵を失う

## 帰結

### ポジティブな帰結

- tier1 と tier2/tier3 の契約が IDL 1 点で定義される明確さ
- tier2/tier3 の採用市場が広くなる（Go/Rust 必須ではない）
- 将来の tier1 言語変更（Go → Rust、Rust → Go）が可逆
- クライアント SDK のバージョニングで段階移行が技術的に保証される

### ネガティブな帰結

- SDK 配布インフラ運用（内部 Nexus/Artifactory）が増える
- tier2/tier3 のデバッグで tier1 内部を覗けないため、障害切り分けが tier1 チーム依存になりやすい（観測性で補う必要）
- IDL 変更時のクライアント SDK リリース遅延が tier2/tier3 のブロッカーになる可能性

## 実装タスク

- モノレポの workspace 境界を物理的に分離（tier1/go.mod、tier1/Cargo.toml、tier2/...）
- クライアント SDK の言語別生成 CI パイプライン（Go/Rust/TypeScript/C#/Java/Python）
- 内部 Nexus/Artifactory を SDK 配布基盤として整備
- Backstage テンプレートに「SDK 追加」フローを組込み、tier2/tier3 がすぐ利用開始できる UX
- tier2/tier3 の PR で tier1 内部パッケージ import を検出する lint rule（GitHub Actions）

## 参考文献

- Buf Schema Registry: buf.build/docs/bsr
- Team Topologies (Skelton & Pais): プラットフォームチームと stream-aligned team の境界設計
- ThoughtWorks Tech Radar: Contract Testing / IDL First
