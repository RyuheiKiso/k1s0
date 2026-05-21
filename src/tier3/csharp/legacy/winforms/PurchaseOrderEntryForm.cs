// PurchaseOrderEntryForm.cs — k1s0 Tier3 Legacy WinForms 発注入力フォーム
// ERP 並行運用向け発注入力フォーム: 発注番号・品名・数量・納品日を入力して BFF に送信する
// docs/03_概要設計/04_tier3設計方針/09_レガシー資産統合.md の Companion + Gateway 経由 HTTP/1.1 設計に準拠する
// tier2 SDK の WinForms wrapper 経由で BFF に送信する（OSS 直接アクセス禁止）

// System 名前空間: 基本型 / 例外処理に使用する
using System;
// System.Net.Http 名前空間: HttpClient 経由の BFF 送信に使用する
using System.Net.Http;
// System.Text 名前空間: JSON 直列化に使用する
using System.Text;
// System.Threading.Tasks 名前空間: 非同期処理に使用する
using System.Threading.Tasks;
// System.Windows.Forms 名前空間: WinForms UI 構築に使用する
using System.Windows.Forms;

// k1s0 Legacy WinForms 名前空間
namespace K1s0.Tier3.Legacy.WinForms
{
    // PurchaseOrderEntryForm: 発注入力フォーム
    // 発注番号・品名・数量・納品日を入力して tier2 SDK WinForms wrapper 経由で BFF に HTTP POST する
    internal sealed class PurchaseOrderEntryForm : Form
    {
        // 発注番号入力フィールド（e.g. "PO-2026-00001"）
        private readonly TextBox _txtPurchaseOrderNumber;
        // 品名入力フィールド（発注対象の品目名称）
        private readonly TextBox _txtItemName;
        // 数量入力フィールド（正の整数）
        private readonly TextBox _txtQuantity;
        // 納品日入力フィールド（YYYY-MM-DD 形式）
        private readonly DateTimePicker _dtpDeliveryDate;
        // 送信ボタン（BFF に HTTP POST する）
        private readonly Button _btnSubmit;
        // キャンセルボタン（フォームを閉じる）
        private readonly Button _btnCancel;
        // 処理中インジケータラベル（送信中に表示する）
        private readonly Label _lblStatus;
        // BFF への HTTP/1.1 接続用 HttpClient（MainForm から受け取る）
        // Companion + Gateway 経由でのみ接続する（OSS 直接アクセス禁止）
        private readonly HttpClient _httpClient;
        // 発注 API エンドポイントのパス（v1_legacy_http11 専用 listener 経由）
        private const string PurchaseOrderApiPath = "/api/v1/purchase-orders";

        // PurchaseOrderEntryForm コンストラクタ: UI 構築と HttpClient 設定
        // httpClient: MainForm から受け取る HttpClient（Companion + Gateway 経由）
        public PurchaseOrderEntryForm(HttpClient httpClient)
        {
            // MainForm から受け取った HttpClient を保持する（Companion + Gateway 経由）
            _httpClient = httpClient;

            // フォームウィンドウタイトルを設定する
            Text = "発注入力フォーム";
            // フォームサイズを設定する（業務入力フォームの標準サイズ）
            Size = new System.Drawing.Size(520, 400);
            // フォームのサイズ変更を禁止する（レイアウト崩れを防ぐ）
            FormBorderStyle = FormBorderStyle.FixedDialog;
            // 最大化ボタンを非表示にする
            MaximizeBox = false;
            // フォームのスタート位置を親フォーム中央に設定する
            StartPosition = FormStartPosition.CenterParent;

            // ---- 発注番号ラベルと入力フィールドを生成する ----
            // 発注番号ラベルを生成する
            var lblPurchaseOrderNumber = new Label
            {
                // ラベルテキストを設定する
                Text = "発注番号 *:",
                // ラベルの表示位置を設定する
                Location = new System.Drawing.Point(20, 30),
                // ラベルのサイズを設定する
                Size = new System.Drawing.Size(120, 25),
                // テキストの垂直配置を中央に設定する
                TextAlign = System.Drawing.ContentAlignment.MiddleRight,
            };
            // 発注番号入力フィールドを生成する
            _txtPurchaseOrderNumber = new TextBox
            {
                // 入力フィールドの表示位置を設定する
                Location = new System.Drawing.Point(150, 30),
                // 入力フィールドのサイズを設定する
                Size = new System.Drawing.Size(320, 25),
                // プレースホルダーテキストを設定する（例示用）
                // WinForms の .NET Framework では PlaceholderText は .NET 5+ のみ対応
                Text = "",
                // Tab 操作順序を設定する
                TabIndex = 0,
                // 最大入力文字数を設定する（発注番号: 50 文字まで）
                MaxLength = 50,
            };

            // ---- 品名ラベルと入力フィールドを生成する ----
            // 品名ラベルを生成する
            var lblItemName = new Label
            {
                // ラベルテキストを設定する
                Text = "品名 *:",
                // ラベルの表示位置を設定する
                Location = new System.Drawing.Point(20, 75),
                // ラベルのサイズを設定する
                Size = new System.Drawing.Size(120, 25),
                // テキストの垂直配置を中央に設定する
                TextAlign = System.Drawing.ContentAlignment.MiddleRight,
            };
            // 品名入力フィールドを生成する
            _txtItemName = new TextBox
            {
                // 入力フィールドの表示位置を設定する
                Location = new System.Drawing.Point(150, 75),
                // 入力フィールドのサイズを設定する
                Size = new System.Drawing.Size(320, 25),
                // Tab 操作順序を設定する
                TabIndex = 1,
                // 最大入力文字数を設定する（品名: 200 文字まで）
                MaxLength = 200,
            };

            // ---- 数量ラベルと入力フィールドを生成する ----
            // 数量ラベルを生成する
            var lblQuantity = new Label
            {
                // ラベルテキストを設定する
                Text = "数量 *:",
                // ラベルの表示位置を設定する
                Location = new System.Drawing.Point(20, 120),
                // ラベルのサイズを設定する
                Size = new System.Drawing.Size(120, 25),
                // テキストの垂直配置を中央に設定する
                TextAlign = System.Drawing.ContentAlignment.MiddleRight,
            };
            // 数量入力フィールドを生成する
            _txtQuantity = new TextBox
            {
                // 入力フィールドの表示位置を設定する
                Location = new System.Drawing.Point(150, 120),
                // 入力フィールドのサイズを設定する
                Size = new System.Drawing.Size(120, 25),
                // Tab 操作順序を設定する
                TabIndex = 2,
                // 最大入力文字数を設定する（数量: 10 桁まで）
                MaxLength = 10,
            };

            // ---- 納品日ラベルと DateTimePicker を生成する ----
            // 納品日ラベルを生成する
            var lblDeliveryDate = new Label
            {
                // ラベルテキストを設定する
                Text = "納品日 *:",
                // ラベルの表示位置を設定する
                Location = new System.Drawing.Point(20, 165),
                // ラベルのサイズを設定する
                Size = new System.Drawing.Size(120, 25),
                // テキストの垂直配置を中央に設定する
                TextAlign = System.Drawing.ContentAlignment.MiddleRight,
            };
            // 納品日 DateTimePicker を生成する
            _dtpDeliveryDate = new DateTimePicker
            {
                // DateTimePicker の表示位置を設定する
                Location = new System.Drawing.Point(150, 165),
                // DateTimePicker のサイズを設定する
                Size = new System.Drawing.Size(200, 25),
                // 表示形式を short date にする（YYYY/MM/DD 形式）
                Format = DateTimePickerFormat.Short,
                // 初期値を今日の 1 週間後に設定する（典型的な納品日）
                Value = DateTime.Today.AddDays(7),
                // Tab 操作順序を設定する
                TabIndex = 3,
            };

            // ---- ステータスラベルを生成する ----
            // ステータスラベルを生成する（送信中 / 成功 / 失敗を表示する）
            _lblStatus = new Label
            {
                // 初期テキストを空にする
                Text = "",
                // ラベルの表示位置を設定する
                Location = new System.Drawing.Point(20, 215),
                // ラベルのサイズを設定する
                Size = new System.Drawing.Size(460, 50),
                // テキストの配置を左揃えにする
                TextAlign = System.Drawing.ContentAlignment.MiddleLeft,
            };

            // ---- 送信ボタンを生成する ----
            // 送信ボタンを生成する（BFF に HTTP POST する）
            _btnSubmit = new Button
            {
                // ボタンのテキストを設定する
                Text = "送信",
                // ボタンの表示位置を設定する
                Location = new System.Drawing.Point(280, 290),
                // ボタンのサイズを設定する
                Size = new System.Drawing.Size(100, 35),
                // Tab 操作順序を設定する
                TabIndex = 4,
                // デフォルトボタンとして設定する（Enter キーで送信できる）
                DialogResult = DialogResult.None,
            };
            // 送信ボタンのクリックイベントを登録する
            _btnSubmit.Click += OnSubmitClick;

            // ---- キャンセルボタンを生成する ----
            // キャンセルボタンを生成する（フォームを閉じる）
            _btnCancel = new Button
            {
                // ボタンのテキストを設定する
                Text = "キャンセル",
                // ボタンの表示位置を設定する
                Location = new System.Drawing.Point(390, 290),
                // ボタンのサイズを設定する
                Size = new System.Drawing.Size(100, 35),
                // Tab 操作順序を設定する
                TabIndex = 5,
                // キャンセルボタンとして設定する（Escape キーでフォームを閉じる）
                DialogResult = DialogResult.Cancel,
            };

            // ---- フォームにコントロールを追加する ----
            // 発注番号ラベルをフォームに追加する
            Controls.Add(lblPurchaseOrderNumber);
            // 発注番号入力フィールドをフォームに追加する
            Controls.Add(_txtPurchaseOrderNumber);
            // 品名ラベルをフォームに追加する
            Controls.Add(lblItemName);
            // 品名入力フィールドをフォームに追加する
            Controls.Add(_txtItemName);
            // 数量ラベルをフォームに追加する
            Controls.Add(lblQuantity);
            // 数量入力フィールドをフォームに追加する
            Controls.Add(_txtQuantity);
            // 納品日ラベルをフォームに追加する
            Controls.Add(lblDeliveryDate);
            // 納品日 DateTimePicker をフォームに追加する
            Controls.Add(_dtpDeliveryDate);
            // ステータスラベルをフォームに追加する
            Controls.Add(_lblStatus);
            // 送信ボタンをフォームに追加する
            Controls.Add(_btnSubmit);
            // キャンセルボタンをフォームに追加する
            Controls.Add(_btnCancel);
            // キャンセルボタンをフォームのキャンセルボタンとして設定する
            CancelButton = _btnCancel;
        }

        // OnSubmitClick: 送信ボタンのクリックイベントハンドラ
        // 入力値を検証して BFF に HTTP POST する（tier2 SDK WinForms wrapper 経由）
        private async void OnSubmitClick(object? sender, EventArgs e)
        {
            // 入力値を検証する
            if (!ValidateInput(out var validationError))
            {
                // バリデーションエラーを表示する
                _lblStatus.Text = string.Format("入力エラー: {0}", validationError);
                // テキスト色を赤に設定する
                _lblStatus.ForeColor = System.Drawing.Color.Red;
                // 送信を中止する
                return;
            }
            // 送信中状態にする（ボタンを無効化してステータスを表示する）
            _btnSubmit.Enabled = false;
            // ステータスラベルを送信中に更新する
            _lblStatus.Text = "BFF に送信中...";
            // テキスト色を黒に設定する（通常色）
            _lblStatus.ForeColor = System.Drawing.Color.Black;
            // 発注データを BFF に送信する
            await SubmitPurchaseOrderAsync();
            // 送信後にボタンを有効化する
            _btnSubmit.Enabled = true;
        }

        // ValidateInput: 入力値の検証を行う
        // エラーメッセージを out パラメータで返す（正常時は null）
        private bool ValidateInput(out string? errorMessage)
        {
            // 発注番号の空チェックを行う
            if (string.IsNullOrWhiteSpace(_txtPurchaseOrderNumber.Text))
            {
                // 発注番号が空の場合はエラーを返す
                errorMessage = "発注番号を入力してください";
                // バリデーション失敗を返す
                return false;
            }
            // 品名の空チェックを行う
            if (string.IsNullOrWhiteSpace(_txtItemName.Text))
            {
                // 品名が空の場合はエラーを返す
                errorMessage = "品名を入力してください";
                // バリデーション失敗を返す
                return false;
            }
            // 数量の数値チェックを行う
            if (!int.TryParse(_txtQuantity.Text.Trim(), out var quantity) || quantity <= 0)
            {
                // 数量が正の整数でない場合はエラーを返す
                errorMessage = "数量は 1 以上の整数を入力してください";
                // バリデーション失敗を返す
                return false;
            }
            // 納品日の過去日チェックを行う（今日以降の日付を要求する）
            if (_dtpDeliveryDate.Value.Date < DateTime.Today)
            {
                // 納品日が過去の場合はエラーを返す
                errorMessage = "納品日は本日以降の日付を選択してください";
                // バリデーション失敗を返す
                return false;
            }
            // バリデーション成功: エラーメッセージを null にする
            errorMessage = null;
            // バリデーション成功を返す
            return true;
        }

        // SubmitPurchaseOrderAsync: 発注データを BFF に HTTP POST する
        // tier2 SDK WinForms wrapper 経由で送信する（Companion + Gateway 経由）
        private async Task SubmitPurchaseOrderAsync()
        {
            try
            {
                // 発注データの JSON ペイロードを組み立てる（tier2 SDK の proto 派生スキーマに準拠する）
                // tenant_id は BFF cookie から注入されるため、ペイロードに含めない（tier3/CLAUDE.md 規約）
                var json = string.Format(
                    // JSON テンプレート（シンプルな string.Format で組み立てる）
                    "{{\"purchaseOrderNumber\":\"{0}\",\"itemName\":\"{1}\",\"quantity\":{2},\"deliveryDate\":\"{3}\"}}",
                    // 発注番号（JSON エスケープを適用する）
                    EscapeJson(_txtPurchaseOrderNumber.Text.Trim()),
                    // 品名（JSON エスケープを適用する）
                    EscapeJson(_txtItemName.Text.Trim()),
                    // 数量（整数値）
                    int.Parse(_txtQuantity.Text.Trim()),
                    // 納品日（ISO 8601 形式 YYYY-MM-DD）
                    _dtpDeliveryDate.Value.ToString("yyyy-MM-dd")
                );
                // HTTP POST リクエストの StringContent を組み立てる（application/json）
                using (var content = new StringContent(json, Encoding.UTF8, "application/json"))
                {
                    // BFF の発注 API に HTTP POST する（Companion + Gateway 経由）
                    var response = await _httpClient
                        .PostAsync(PurchaseOrderApiPath, content)
                        .ConfigureAwait(false);
                    // レスポンスステータスを確認する
                    if (response.IsSuccessStatusCode)
                    {
                        // 送信成功: ステータスラベルを更新する（UI スレッドで実行する）
                        Invoke(new Action(() =>
                        {
                            // 送信成功メッセージを表示する
                            _lblStatus.Text = string.Format(
                                "送信完了（HTTP {0}）: 発注番号 {1} が正常に登録されました",
                                (int)response.StatusCode,
                                _txtPurchaseOrderNumber.Text.Trim()
                            );
                            // テキスト色を緑に設定する（成功を視覚的に示す）
                            _lblStatus.ForeColor = System.Drawing.Color.Green;
                        }));
                    }
                    else
                    {
                        // 送信失敗: HTTP エラーステータスをステータスラベルに表示する
                        Invoke(new Action(() =>
                        {
                            // 送信失敗メッセージを表示する
                            _lblStatus.Text = string.Format(
                                "送信失敗（HTTP {0}）: BFF からエラーが返されました",
                                (int)response.StatusCode
                            );
                            // テキスト色を赤に設定する（失敗を視覚的に示す）
                            _lblStatus.ForeColor = System.Drawing.Color.Red;
                        }));
                    }
                }
            }
            catch (Exception ex)
            {
                // 例外発生: ステータスラベルをエラー状態に更新する（UI スレッドで実行する）
                Invoke(new Action(() =>
                {
                    // 例外メッセージを表示する
                    _lblStatus.Text = string.Format(
                        "通信エラー: {0}",
                        ex.Message
                    );
                    // テキスト色を赤に設定する（エラーを視覚的に示す）
                    _lblStatus.ForeColor = System.Drawing.Color.Red;
                }));
            }
        }

        // EscapeJson: JSON 文字列のエスケープ処理を行う（最低限の実装）
        // System.Text.Json が .NET Framework 4.8 で利用不可のため手動エスケープする
        private static string EscapeJson(string input)
        {
            // null チェックを行う
            if (input == null)
            {
                // null の場合は空文字を返す
                return string.Empty;
            }
            // バックスラッシュをエスケープする
            return input
                .Replace("\\", "\\\\")
                // ダブルクォートをエスケープする
                .Replace("\"", "\\\"")
                // 改行コード (CR) をエスケープする
                .Replace("\r", "\\r")
                // 改行コード (LF) をエスケープする
                .Replace("\n", "\\n")
                // タブ文字をエスケープする
                .Replace("\t", "\\t");
        }
    }
}
