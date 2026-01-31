# ADR-0010: Android フロントエンドサポートの追加

## ステータス

承認済み

## コンテキスト

k1s0 は React と Flutter の 2 つのフロントエンドフレームワークをサポートしている。しかし、ネイティブ Android アプリケーション開発の需要がある。特に以下の要因がある:

- Android プラットフォーム固有の機能（カメラ、センサー、バックグラウンド処理）を活用するアプリの需要
- Jetpack Compose の成熟によるモダンな宣言的 UI 開発の実現
- Kotlin バックエンドとの言語統一によるチーム効率の向上
- Google の Material 3 デザインシステムへのネイティブ対応

## 決定

k1s0 において、`frontend-android` テンプレートタイプを追加し、Jetpack Compose ベースの Android フロントエンドサポートを導入する。

### 具体的な変更内容

1. **CLI 拡張**: `new-feature` コマンドで `--type frontend-android` を選択可能にする
2. **テンプレート追加**: `CLI/templates/frontend-android/` に feature テンプレートを作成する
3. **フレームワークパッケージ**: `framework/frontend/android/` に共通パッケージを提供する
   - k1s0-navigation, k1s0-config, k1s0-http, k1s0-ui, k1s0-auth, k1s0-observability, k1s0-state, k1s0-realtime
4. **Lint 対応**: K020/K022 の既存 lint ルールを Android/Kotlin コードに対応させる
5. **CI/CD**: `frontend-android.yml` ワークフローを追加する
6. **Clean Architecture 準拠**: 他フロントエンドと同じ 4 層構造を実現する

### 技術選定

| 項目 | 選定 | 理由 |
|------|------|------|
| UI フレームワーク | Jetpack Compose | Google 推奨のモダン宣言的 UI、Material 3 ネイティブ対応 |
| ビルドツール | Gradle Kotlin DSL | Android 標準 |
| DI | Koin | backend-kotlin と統一、軽量 |
| HTTP | Ktor Client | Kotlin ネイティブ、マルチプラットフォーム対応 |
| ナビゲーション | Navigation Compose | Jetpack 標準 |
| 状態管理 | ViewModel + StateFlow | Android 標準の MVVM パターン |
| テスト | JUnit 5 + Compose UI Test | Android 標準 |

## 理由

- **Jetpack Compose の選定**: XML レイアウトと比較して宣言的で、React/Flutter と同様のコンポーネント指向開発が可能
- **Koin の選定**: backend-kotlin と統一でき、Hilt/Dagger と比較して設定が簡潔
- **Ktor Client の選定**: backend-kotlin と HTTP クライアントライブラリを統一でき、Kotlin コルーチン対応
- **ネイティブ Android の価値**: Flutter ではカバーしきれないプラットフォーム固有機能へのフルアクセスが可能

## 結果

### ポジティブ

- k1s0 がサポートするフロントエンドが 3 つに増加する（React, Flutter, Android）
- Android ネイティブ開発チームが k1s0 プラットフォームを採用可能になる
- backend-kotlin とフロントエンド Android で Kotlin を共通言語として使用可能になる

### ネガティブ

- 3 つ目のフロントエンドフレームワークのメンテナンスコストが発生する
- Android 固有のビルド設定（minSdk, compileSdk 等）の保守が必要になる
- Android エミュレータを含む CI 環境の構築コストが発生する

### 関連 ADR

- [ADR-0001](ADR-0001-scope-and-prerequisites.md): スコープと前提条件
- [ADR-0006](ADR-0006-three-layer-architecture.md): 三層アーキテクチャ
- [ADR-0009](ADR-0009-kotlin-backend-support.md): Kotlin バックエンドサポート
