# ADR-MIG-001: .NET Framework 資産のサイドカー方式移行を採用

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: 移行チーム / レガシー所有部門 / Product Council

## コンテキスト

k1s0 を採用する 採用側組織の現行基幹系には、.NET Framework 4.x（Windows Server 2012R2 / 2016）で運用されている資産が多数存在することが想定される。これらは以下の理由で k1s0 への一括書換が現実的でない。

- **資産規模**: 数百万行級コード、ビジネスロジックが複雑で書換負荷が膨大
- **動作保証**: 監督官庁認可済みの挙動、書換時の挙動差異に法的リスク
- **開発要員**: 業務知識を持つ要員は Rust / Go の経験薄、教育期間が必要
- **運用移行**: 並行運用期間に旧系 / 新系の双方向通信が必要

BR-MIG-001（段階的移行）では「2 年間の並行運用」「挙動完全一致の検証」「撤退容易性」を求めており、一括書換は選択肢外。

業界標準の Strangler Fig Pattern（Martin Fowler）では、レガシーを段階的に包み替える手法として、1）Facade 方式、2）Event Interception、3）Sidecar 方式 の 3 種類が提案されている。

## 決定

**.NET Framework 資産の段階的移行は「サイドカー方式」を第一候補、API Gateway 方式（ADR-MIG-002）を補完として採用する。**

- .NET Framework アプリを Windows Container として Kubernetes（Windows Node）で稼働
- 同 Pod 内にサイドカーコンテナ（Linux Container、Dapr Go SDK）を配置
- サイドカーが Dapr ファサード（tier1 API）と通信、.NET Framework は localhost で HTTP/gRPC 呼出
- 新機能は tier2 側で Rust / Go で実装、サイドカー経由で旧アプリから呼出
- 旧機能の置換は Facade Replacement（機能単位で .NET → tier2 に移行）で段階進行
- 移行期間中は Dapr State / Audit / Pii API で二重書込み、差分監査で挙動一致検証

## 検討した選択肢

### 選択肢 A: サイドカー方式（採用、第一候補）

- 概要: 旧アプリは変更せず、同 Pod の Sidecar が k1s0 機能を提供
- メリット:
  - 旧アプリのコード変更最小（設定ファイルの localhost URL 変更のみ）
  - 機能単位で段階移行、リスク最小化
  - 旧系撤退容易（Sidecar 削除 → .NET 単独稼働に戻せる）
  - Windows Container 化で Kubernetes 運用統合
- デメリット:
  - Windows Node が必要、Linux Node と混在運用コスト
  - Windows Container のイメージサイズが大きい（数 GB）、ビルド時間増
  - Windows Container の Pod 起動時間がやや長い

### 選択肢 B: API Gateway 方式（ADR-MIG-002、補完採用）

- 概要: 旧系の前段に API Gateway を置き、新機能は tier2 へルーティング
- メリット: 旧系に一切変更不要、クライアントからも透過
- デメリット:
  - サイドカーで実現できる Dapr State / Secrets の統合は限定的
  - Gateway のルーティングロジックが肥大化

**→ サイドカー方式（Pod 内統合）と API Gateway 方式（外部ルーティング）は補完関係、機能特性で使い分け**

### 選択肢 C: .NET Framework → .NET 8+ 書換（ビッグバン）

- 概要: Framework 依存コードを .NET 8+ にポート、Linux Container 化
- メリット: 移行後は Linux ネイティブで運用統合
- デメリット:
  - 書換負荷膨大、採用側の現実的な移行スコープに収まらない
  - Framework 依存 API（WCF、WF、System.Web 等）の書換リスク
  - 挙動差異検証の負荷が膨大

### 選択肢 D: Event Interception

- 概要: データベース CDC でイベント駆動移行
- メリット: 旧系コード無変更、データ駆動型移行
- デメリット:
  - .NET Framework の DB アクセスは SQL Server 依存、k1s0 PostgreSQL との整合性検証複雑
  - 双方向同期の複雑さ

### 選択肢 E: 完全新規構築、旧系は凍結

- 概要: k1s0 上で完全新規、旧系は並行運用し自然消滅待ち
- メリット: 新規開発に集中
- デメリット:
  - 旧系のバグ修正・機能追加が止まる、業務影響
  - データ連携を都度個別実装、技術負債蓄積
  - BR-MIG-001 の「段階的」要件に反する

## 帰結

### ポジティブな帰結

- 旧系資産を活かしながら段階的に k1s0 機能で拡張可能
- 撤退容易性（Sidecar 外せば旧系単独運用に戻せる）で経営リスク最小化
- 機能単位の移行で各 PoC 段階で効果検証可能
- 旧系の業務知識保持要員が Rust / Go 学習と並行して実務継続

### ネガティブな帰結

- Windows Node 運用コスト、Linux Node と混在管理
- Windows Container ビルドパイプラインの構築
- サイドカー起動・終了順序の制御（旧アプリは Sidecar 準備完了を待つ必要）
- .NET Framework 側から Dapr HTTP/gRPC 呼出するクライアントライブラリの整備

## 実装タスク

- Windows Node Pool の Kubernetes クラスタ追加、Istio Ambient 対応検証
- Windows Container ベースイメージ（Windows Server Core 2019+）の社内標準化
- .NET Framework → Dapr HTTP/gRPC クライアントの内部ライブラリ作成
- サイドカー起動順序制御（initContainer、lifecycle hooks）
- 機能単位移行計画（採用側の移行ウェーブ別に移行対象リストアップ）
- 挙動一致検証フレームワーク（旧系 / 新系の二重実行結果比較）
- Runbook: サイドカー障害時の旧系 fallback 手順
- 移行進捗ダッシュボード（Grafana）

## 参考文献

- Martin Fowler: Strangler Fig Pattern
- Microsoft Docs: Windows Container on Kubernetes
- Dapr on Windows Container
- BR-MIG-001 段階的移行要件
