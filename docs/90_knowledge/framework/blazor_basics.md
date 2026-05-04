# Blazor: 概要

- 対象読者: HTML・CSS・JavaScript の基本を理解しており、C# の基本構文（クラス・メソッド・LINQ 程度）に触れたことがある開発者
- 学習目標: Blazor の設計思想とホスティングモデル / レンダーモードを理解し、Razor コンポーネントで簡単な対話的 UI を書けるようになる
- 所要時間: 約 45 分
- 対象バージョン: .NET 10（Blazor Web App）
- 最終更新日: 2026-04-28

## 1. このドキュメントで学べること

- Blazor が「なぜ」必要とされるかを説明できる
- Razor コンポーネント・パラメーター・データバインディングの基本を理解できる
- Blazor Server / Blazor WebAssembly / Blazor Hybrid の違いを使い分けられる
- .NET 10 の Blazor Web App におけるレンダーモード（Static SSR / InteractiveServer / InteractiveWebAssembly / InteractiveAuto）を選択できる

## 2. 前提知識

- HTML のタグ構造と基本属性
- CSS によるスタイリングの基礎
- C# の関数・変数・クラスの基本構文
- HTTP / WebSocket の通信モデルの概念

## 3. 概要

Blazor は Microsoft が開発する .NET ベースの Web UI フレームワークである。`.razor` 拡張子を持つコンポーネントファイルに HTML と C# を混在させて記述し、JavaScript の代わりに C# でクライアント側のロジックを書ける。バックエンドと UI で同じ言語・同じ型・同じライブラリを共有できることが最大の特徴である。

従来の SPA フレームワーク（React / Vue / Angular）は JavaScript / TypeScript を前提とし、サーバー側の言語との型共有には別途コード生成（OpenAPI / gRPC / GraphQL）を要する。Blazor は WebAssembly 上で .NET ランタイムを動かす、もしくはサーバー側で UI を実行して差分のみ送る、という二系統の実行モデルを提供することで、この二重実装の問題を解消する。.NET 8 以降は **Blazor Web App** という統合プロジェクトテンプレートが導入され、ページ単位・コンポーネント単位で実行モードを混在させられるようになった。

## 4. 用語の整理

| 用語 | 説明 |
|------|------|
| Razor コンポーネント | UI を構成する `.razor` ファイル。HTML マークアップと C# コードを 1 ファイルに記述する |
| パラメーター（Parameter） | 親コンポーネントから子に渡すデータ。`[Parameter]` 属性を付けたプロパティで受け取る |
| データバインディング | UI とコンポーネントの値を双方向に同期する仕組み。`@bind` ディレクティブで指定する |
| ホスティングモデル | アプリ全体としての実行配置（Blazor Server / Blazor WebAssembly / Blazor Hybrid） |
| レンダーモード | 個々のコンポーネントの実行・対話方式（Static SSR / InteractiveServer / InteractiveWebAssembly / InteractiveAuto） |
| SignalR | Blazor Server がブラウザとサーバー間で UI 差分をやり取りする WebSocket 上のリアルタイム通信ライブラリ |
| WebAssembly（WASM） | ブラウザ上でネイティブ近い速度でコードを動かす標準バイナリ形式。.NET ランタイムを WASM 化したものをブラウザに配信する |
| Blazor Web App | .NET 8 以降のテンプレート。SSR とインタラクティブモードを単一プロジェクトで混在できる |

## 5. 仕組み・アーキテクチャ

Blazor は実行配置によって 3 つのホスティングモデルを持つ。サーバーで実行する Blazor Server は SignalR で UI 差分を送り、ブラウザ内で実行する Blazor WebAssembly は .NET ランタイムを WASM として配信し、ネイティブアプリに埋め込む Blazor Hybrid は WebView コントロールに UI を描画する。

![Blazor ホスティングモデル比較](./img/blazor_basics_hosting.svg)

.NET 8 以降の Blazor Web App では、これらの実行モデルを「レンダーモード」としてコンポーネント単位で切り替えられる。下図は典型的な選択フローである。対話の有無、初回ロード優先か、オフライン要件があるかで使い分ける。

![Blazor レンダーモード選択フロー](./img/blazor_basics_render-modes.svg)

## 6. 環境構築

### 6.1 必要なもの

- .NET 10 SDK
- テキストエディタ（Visual Studio 2026、VS Code + C# Dev Kit 拡張、JetBrains Rider など）

### 6.2 セットアップ手順

```bash
# .NET SDK のバージョンを確認する
dotnet --version

# Blazor Web App テンプレートで新規プロジェクトを作成する
dotnet new blazor -o MyBlazorApp --interactivity Auto

# プロジェクトディレクトリに移動する
cd MyBlazorApp
```

`--interactivity Auto` を指定すると、後述の InteractiveAuto モードを既定として有効にしたテンプレートが生成される。

### 6.3 動作確認

```bash
# 開発サーバーを起動する
dotnet run
```

ブラウザで `http://localhost:5000` を開き、初期ページが表示されればセットアップ完了である。

## 7. 基本の使い方

```razor
@* Counter.razor — クリックでカウントを増減する Razor コンポーネント *@

@* このコンポーネントを /counter ルートに割り当てる *@
@page "/counter"

@* このコンポーネントを InteractiveServer モードで実行する *@
@rendermode InteractiveServer

<h1>カウンター</h1>

@* バインドした count を画面に出力する *@
<p>現在のカウント: @count</p>

@* クリック時に Increment メソッドを呼び出すボタン *@
<button class="btn btn-primary" @onclick="Increment">+1</button>

@* 子コンポーネントに初期値をパラメーターとして渡す例 *@
<Greeting Name="Blazor" />

@code {
    // コンポーネント内部で保持するカウント値
    private int count = 0;

    // ボタン押下時に呼び出され、count を 1 増やす
    private void Increment()
    {
        // count を更新すると Blazor が自動で再レンダリングする
        count++;
    }
}
```

```razor
@* Greeting.razor — 名前を Props として受け取る子コンポーネント *@

@* 親から渡された名前を表示する *@
<h2>Hello, @Name!</h2>

@code {
    // 親からパラメーターとして受け取る公開プロパティ
    [Parameter]
    public string Name { get; set; } = string.Empty;
}
```

### 解説

- **`@page` ディレクティブ**: コンポーネントを URL ルートに割り当てる。指定しないと単なる部品として親から再利用される
- **`@rendermode` ディレクティブ**: コンポーネントの実行方式を指定する。指定しないと Static SSR となり対話イベントは動かない
- **`@onclick`**: HTML イベントを C# メソッドにバインドする。引数として `MouseEventArgs` を受け取れる
- **`[Parameter]` 属性**: 公開プロパティに付けることで、親コンポーネントから値を渡せる Props となる
- **`@code` ブロック**: 同ファイル内に C# のフィールドやメソッドを定義する領域

## 8. ステップアップ

### 8.1 双方向データバインディング

`@bind` ディレクティブは入力値とコンポーネント変数を双方向に同期する。下例は `oninput` イベントで即時反映する書き方である。

```razor
@* TodoInput.razor — 入力即時反映する双方向バインディングの例 *@

@page "/todo"
@rendermode InteractiveServer

@* text への入力をリアルタイムで title と同期する *@
<input @bind="title" @bind:event="oninput" placeholder="タイトル" />

@* 入力中のタイトルを即時プレビュー表示する *@
<p>プレビュー: @title</p>

@code {
    // 入力欄と双方向バインドする状態
    private string title = string.Empty;
}
```

### 8.2 レンダーモードの混在

Blazor Web App では同じアプリ内で SSR とインタラクティブを混在できる。トップページは静的に配信し、ダッシュボードだけクライアント実行にする、といった使い分けが可能である。

```razor
@* Dashboard.razor — 同一アプリ内で WASM 実行に切り替える例 *@

@page "/dashboard"

@* このコンポーネントだけクライアント側 WASM で実行する *@
@rendermode InteractiveWebAssembly

<h1>ダッシュボード</h1>
<p>このページはブラウザ内 .NET WASM ランタイムで動作している。</p>
```

## 9. よくある落とし穴

- **対話イベントが動かない**: `@rendermode` の指定漏れ。Static SSR は HTML を返すだけで `@onclick` が無効になる。コンポーネントまたはホストページにレンダーモード指定を追加する
- **Blazor Server で同期 IO**: 1 サーバープロセスで多数の SignalR 接続を捌くため、`Thread.Sleep` などのブロッキング処理は他ユーザーの応答性まで悪化させる。`async`/`await` を徹底する
- **Blazor WebAssembly の初回ロードが遅い**: 数十 MB の .NET ランタイム＋アセンブリを配信するため、初回は数秒かかる。AOT コンパイル、トリミング、圧縮、CDN 配信で緩和できる
- **State はコンポーネントを跨がない**: 親が再描画されると子のフィールドはリセットされる。永続化が必要なら DI のシングルトンサービスや `PersistentComponentState` を使う

## 10. ベストプラクティス

- **対話性は必要なコンポーネントのみに限定**: 全ページを InteractiveServer にすると SignalR 接続が常時必要になる。SSR で済む箇所は SSR のままにする
- **`InteractiveAuto` を既定に検討**: 初回は Server で素早く反応し、バックグラウンドで WASM をキャッシュする。次回以降はクライアント実行になり、サーバー負荷とレイテンシを両立できる
- **コンポーネント分割の単位**: 1 つの責任 / 1 つのレンダーモードに絞る。同じファイル内で SSR と Interactive が混在すると動作が混乱しやすい
- **データ取得は `OnInitializedAsync` で**: コンポーネント初期化時に呼ばれる非同期ライフサイクルメソッドを使い、UI スレッドをブロックしない

## 11. 演習問題

1. テキスト入力欄と「追加」ボタンを持つ Todo アプリを作成せよ。`List<string>` を `@code` で保持し、入力した文字列を追加・表示できること
2. `OnInitializedAsync` を使って外部 API（例: `https://api.example.com/quote`）から取得した文字列を表示するコンポーネントを作成せよ
3. 同一プロジェクト内に `/static` ページ（Static SSR）と `/live` ページ（InteractiveServer）を共存させ、それぞれで `@onclick` の有無を確認せよ

## 12. さらに学ぶには

- 公式ドキュメント: <https://learn.microsoft.com/ja-jp/aspnet/core/blazor/?view=aspnetcore-10.0>
- レンダーモード詳説: <https://learn.microsoft.com/en-us/aspnet/core/blazor/components/render-modes?view=aspnetcore-10.0>
- ホスティングモデル詳説: <https://learn.microsoft.com/en-us/aspnet/core/blazor/hosting-models?view=aspnetcore-10.0>
- 関連 Knowledge: `react_basics.md`（宣言的 UI の他言語比較として参照）

## 13. 参考資料

- ASP.NET Core Blazor 公式ドキュメント: <https://learn.microsoft.com/en-us/aspnet/core/blazor/>
- .NET ランタイムリポジトリ: <https://github.com/dotnet/aspnetcore>
- Blazor サンプル集: <https://github.com/dotnet/blazor-samples>
