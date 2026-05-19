// notification_handler.ts — Mattermost webhook 通知ハンドラー（tier3 デプロイ通知）
// tier3 のデプロイイベント（Argo Rollouts / Companion / フィーチャーフラグ）を
// Mattermost チャンネルに Webhook 経由で送信する TypeScript ハンドラー

// Node.js 組み込み https モジュールをインポートする（外部 HTTP ライブラリ不使用）
import { request as httpsRequest } from 'https';
// Node.js 組み込み URL モジュールをインポートする
import { URL } from 'url';

// Mattermost Webhook ペイロードの型定義
interface MattermostPayload {
  // メッセージ本文（Markdown 形式）
  text: string;
  // チャンネル名（省略時は Webhook のデフォルトチャンネルを使用する）
  channel?: string;
  // Bot 表示名（省略時は Webhook 設定の名前を使用する）
  username?: string;
  // Bot アイコン URL（省略時は Webhook 設定のアイコンを使用する）
  icon_url?: string;
  // Attachment（リッチメッセージ）のリスト
  attachments?: MattermostAttachment[];
}

// Mattermost Attachment の型定義（リッチメッセージのコンポーネント）
interface MattermostAttachment {
  // 添付カードの左ボーダー色（RGB 16 進数）
  color: string;
  // 添付カードのタイトル
  title: string;
  // 添付カードの本文
  text: string;
  // フィールド（key-value ペア）のリスト
  fields?: Array<{
    // フィールドのラベル
    title: string;
    // フィールドの値
    value: string;
    // true の場合は横並びで表示する（短い値に使用する）
    short: boolean;
  }>;
}

// デプロイ通知のパラメーター型定義
interface DeployNotificationParams {
  // デプロイ対象の Rollout 名
  rolloutName: string;
  // デプロイのステータス（started / progressing / completed / aborted）
  status: 'started' | 'progressing' | 'completed' | 'aborted';
  // 現在のカナリアウェイト（%）
  canaryWeight: number;
  // デプロイしているバージョン（コンテナイメージタグ）
  version: string;
  // デプロイを実行した actor（CI ジョブ名または ChatOps 実行者）
  actor: string;
  // Argo CD / Rollouts の詳細 URL（省略可能）
  detailUrl?: string;
}

// カナリア rollback 通知のパラメーター型定義
interface RollbackNotificationParams {
  // rollback 対象の Rollout 名
  rolloutName: string;
  // rollback を実行した actor（ChatOps 実行者）
  actor: string;
  // rollback 前のカナリアウェイト（%）
  previousWeight: number;
  // rollback の理由（オプション）
  reason?: string;
}

// Companion デプロイ通知のパラメーター型定義
interface CompanionDeployNotificationParams {
  // デプロイしたプラットフォーム（macos / linux / windows）
  platform: 'macos' | 'linux' | 'windows';
  // デプロイしたバージョン
  version: string;
  // デプロイのステータス
  status: 'published' | 'failed';
  // バンドル形式（dmg / appimage / msix / msi）
  bundleFormat: 'dmg' | 'appimage' | 'msix' | 'msi';
  // ダウンロード URL（公開後に設定される）
  downloadUrl?: string;
}

// Mattermost Webhook URL を環境変数から取得する（ハードコード禁止）
// 環境変数が未設定の場合はエラーをスローする（起動時チェック）
function getMattermostWebhookUrl(): string {
  // 環境変数 MATTERMOST_WEBHOOK_URL を読み取る
  const webhookUrl = process.env['MATTERMOST_WEBHOOK_URL'];
  // 未設定の場合は設定エラーをスローする
  if (webhookUrl === undefined || webhookUrl === '') {
    throw new Error('MATTERMOST_WEBHOOK_URL environment variable is not set');
  }
  return webhookUrl;
}

// Mattermost Webhook にメッセージを送信するコア関数
// Promise を返す（async/await ではなく Promise チェーンで実装する）
export function sendToMattermost(payload: MattermostPayload): Promise<void> {
  // Webhook URL を取得する
  const webhookUrl = getMattermostWebhookUrl();
  // URL をパースする
  const parsedUrl = new URL(webhookUrl);
  // ペイロードを JSON 文字列に変換する
  const body = JSON.stringify(payload);

  // Promise を返す（Node.js https モジュールのコールバックを Promise でラップする）
  return new Promise((resolve, reject) => {
    // HTTPS リクエストを設定する
    const req = httpsRequest(
      {
        // ホスト名
        hostname: parsedUrl.hostname,
        // パス（Webhook エンドポイント）
        path: parsedUrl.pathname,
        // HTTP メソッド
        method: 'POST',
        // リクエストヘッダー
        headers: {
          // Content-Type を JSON に設定する
          'Content-Type': 'application/json',
          // Content-Length を設定する（POST 必須）
          'Content-Length': Buffer.byteLength(body),
        },
      },
      // レスポンスハンドラーを設定する
      (res) => {
        // レスポンスデータを蓄積する
        let responseBody = '';
        res.on('data', (chunk: Buffer) => {
          // レスポンスデータを結合する
          responseBody += chunk.toString();
        });
        // レスポンス完了時の処理
        res.on('end', () => {
          // 2xx 以外のステータスコードはエラーとして扱う
          if (res.statusCode !== undefined && res.statusCode >= 200 && res.statusCode < 300) {
            resolve();
          } else {
            reject(new Error(`Mattermost webhook returned ${res.statusCode}: ${responseBody}`));
          }
        });
      },
    );

    // リクエストエラーのハンドリング
    req.on('error', (err) => {
      // エラーを reject に渡す
      reject(err);
    });

    // リクエストボディを送信する
    req.write(body);
    // リクエストを完了させる
    req.end();
  });
}

// カナリアデプロイ通知を Mattermost に送信する
export function notifyDeploymentEvent(params: DeployNotificationParams): Promise<void> {
  // ステータスに応じたボーダー色を決定する
  const colorMap: Record<DeployNotificationParams['status'], string> = {
    // デプロイ開始: 青色
    started: '#0072B2',
    // 進行中: 黄色
    progressing: '#F0A500',
    // 完了: 緑色
    completed: '#00A651',
    // 中断: 赤色
    aborted: '#D32F2F',
  };
  // ステータスに対応するカラーを取得する
  const color = colorMap[params.status];

  // Mattermost ペイロードを構築する
  const payload: MattermostPayload = {
    // チャンネル名（デプロイ通知専用チャンネル）
    channel: 'k1s0-deployments',
    // Bot ユーザー名
    username: 'k1s0-deploy-bot',
    // メッセージテキスト（簡潔な概要）
    text: `**[tier3] ${params.rolloutName}** canary ${params.status}`,
    // Attachment でリッチな詳細情報を表示する
    attachments: [
      {
        // ステータスに対応したボーダー色
        color,
        // タイトル（Rollout 名とステータス）
        title: `${params.rolloutName} — ${params.status.toUpperCase()}`,
        // 詳細本文（Markdown 形式）
        text: params.detailUrl !== undefined
          ? `[Argo Rollouts で詳細を確認する](${params.detailUrl})`
          : '詳細 URL は利用できません',
        // フィールド（key-value 情報）
        fields: [
          {
            // バージョンフィールド
            title: 'Version',
            value: params.version,
            short: true,
          },
          {
            // カナリアウェイトフィールド
            title: 'Canary Weight',
            value: `${params.canaryWeight}%`,
            short: true,
          },
          {
            // 実行 actor フィールド
            title: 'Actor',
            value: params.actor,
            short: true,
          },
        ],
      },
    ],
  };

  // Mattermost に送信する
  return sendToMattermost(payload);
}

// カナリア rollback 通知を Mattermost に送信する
export function notifyCanaryRollback(params: RollbackNotificationParams): Promise<void> {
  // rollback 通知のペイロードを構築する
  const payload: MattermostPayload = {
    // チャンネル名（デプロイ通知専用チャンネル）
    channel: 'k1s0-deployments',
    // Bot ユーザー名
    username: 'k1s0-deploy-bot',
    // メッセージテキスト（緊急性を示す接頭辞を付ける）
    text: `:rotating_light: **[tier3 ROLLBACK]** ${params.rolloutName} をロールバックしました`,
    // Attachment でリッチな詳細情報を表示する
    attachments: [
      {
        // rollback は赤色のボーダーで視認性を高める
        color: '#D32F2F',
        // タイトル
        title: `ROLLBACK: ${params.rolloutName}`,
        // 詳細本文
        text: params.reason !== undefined ? `理由: ${params.reason}` : '理由: ChatOps から手動実行',
        // フィールド
        fields: [
          {
            // rollback 実行者フィールド
            title: 'Rollback Actor',
            value: params.actor,
            short: true,
          },
          {
            // rollback 前のカナリアウェイト
            title: 'Was at Weight',
            value: `${params.previousWeight}%`,
            short: true,
          },
        ],
      },
    ],
  };

  // Mattermost に送信する
  return sendToMattermost(payload);
}

// Companion デプロイ通知を Mattermost に送信する
export function notifyCompanionDeploy(params: CompanionDeployNotificationParams): Promise<void> {
  // ステータスに応じたボーダー色を決定する
  const color = params.status === 'published' ? '#00A651' : '#D32F2F';
  // プラットフォーム名と絵文字のマッピング
  const platformLabel: Record<CompanionDeployNotificationParams['platform'], string> = {
    // macOS プラットフォーム
    macos: ':apple: macOS',
    // Linux プラットフォーム
    linux: ':penguin: Linux',
    // Windows プラットフォーム
    windows: ':windows: Windows',
  };

  // Companion デプロイ通知のペイロードを構築する
  const payload: MattermostPayload = {
    // チャンネル名
    channel: 'k1s0-deployments',
    // Bot ユーザー名
    username: 'k1s0-deploy-bot',
    // メッセージテキスト
    text: `**[tier3 Companion]** ${platformLabel[params.platform]} ${params.bundleFormat.toUpperCase()} ${params.status}`,
    // Attachment でリッチな詳細情報を表示する
    attachments: [
      {
        // ステータスに対応したボーダー色
        color,
        // タイトル
        title: `Companion ${params.version} — ${params.platform}/${params.bundleFormat}`,
        // 詳細本文（ダウンロード URL があれば表示する）
        text: params.downloadUrl !== undefined
          ? `[ダウンロードリンク](${params.downloadUrl})`
          : '配布準備中',
        // フィールド
        fields: [
          {
            // バージョンフィールド
            title: 'Version',
            value: params.version,
            short: true,
          },
          {
            // バンドル形式フィールド
            title: 'Bundle Format',
            value: params.bundleFormat.toUpperCase(),
            short: true,
          },
        ],
      },
    ],
  };

  // Mattermost に送信する
  return sendToMattermost(payload);
}
