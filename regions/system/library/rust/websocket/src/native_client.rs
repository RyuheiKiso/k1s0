use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message as TungsteniteMessage};

use crate::error::WsError;
use crate::message::{CloseFrame, WsMessage};
use crate::state::ConnectionState;
use crate::WsClient;

// バックグラウンドタスクへの送信チャネルの型エイリアス
type SendTx = mpsc::Sender<TungsteniteMessage>;
// バックグラウンドタスクからの受信チャネルの型エイリアス
type RecvRx = tokio::sync::Mutex<mpsc::Receiver<WsMessage>>;

/// TungsteniteWsClient は tokio-tungstenite を使用した本番用 WebSocket クライアント実装。
/// バックグラウンドタスクで接続管理・再接続・メッセージ転送を行い、
/// send/receive を非同期チャネル経由で提供する。
pub struct TungsteniteWsClient {
    config: crate::config::WsConfig,
    // 接続状態（state() が同期メソッドのため std::sync::RwLock を使用する）
    state: Arc<RwLock<ConnectionState>>,
    // バックグラウンドタスクへのメッセージ送信チャネル
    send_tx: Option<SendTx>,
    // バックグラウンドタスクからのメッセージ受信チャネル（Mutex で内部可変性を提供する）
    recv_rx: Option<Arc<RecvRx>>,
    // バックグラウンドタスクのハンドル（disconnect 時に abort する）
    task_handle: Option<tokio::task::JoinHandle<()>>,
}

impl TungsteniteWsClient {
    /// 指定した設定で TungsteniteWsClient を生成する。
    pub fn new(config: crate::config::WsConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            send_tx: None,
            recv_rx: None,
            task_handle: None,
        }
    }
}

#[async_trait]
impl WsClient for TungsteniteWsClient {
    async fn connect(&mut self) -> Result<(), WsError> {
        if *self.state.read().unwrap() != ConnectionState::Disconnected {
            return Err(WsError::AlreadyConnected);
        }
        *self.state.write().unwrap() = ConnectionState::Connecting;

        // 送受信チャネルを作成する
        let (send_tx, send_rx) = mpsc::channel::<TungsteniteMessage>(100);
        let (recv_tx, recv_rx) = mpsc::channel::<WsMessage>(100);

        // 初回接続の成否を通知するチャネルを作成する
        let (connected_tx, connected_rx) = tokio::sync::oneshot::channel::<Result<(), WsError>>();

        let state = Arc::clone(&self.state);
        let config = self.config.clone();

        // バックグラウンドタスクで接続管理・メッセージ転送を実行する
        let handle = tokio::spawn(connection_loop(
            config,
            state,
            send_rx,
            recv_tx,
            connected_tx,
        ));

        self.send_tx = Some(send_tx);
        self.recv_rx = Some(Arc::new(tokio::sync::Mutex::new(recv_rx)));
        self.task_handle = Some(handle);

        // 接続が確立するまで待機し、失敗した場合はエラーを返す
        connected_rx
            .await
            .map_err(|_| WsError::ConnectionError("connection task died".to_string()))?
    }

    async fn disconnect(&mut self) -> Result<(), WsError> {
        if *self.state.read().unwrap() == ConnectionState::Disconnected {
            return Err(WsError::NotConnected);
        }
        *self.state.write().unwrap() = ConnectionState::Closing;

        // send_tx を drop するとバックグラウンドタスクの送信ループが終了する
        self.send_tx = None;
        self.recv_rx = None;

        // バックグラウンドタスクを強制終了して完了を待つ
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
            let _ = handle.await;
        }

        *self.state.write().unwrap() = ConnectionState::Disconnected;
        Ok(())
    }

    async fn send(&self, message: WsMessage) -> Result<(), WsError> {
        if *self.state.read().unwrap() != ConnectionState::Connected {
            return Err(WsError::NotConnected);
        }

        let tung_msg = ws_to_tungstenite(message)?;

        match &self.send_tx {
            Some(tx) => tx
                .send(tung_msg)
                .await
                .map_err(|e| WsError::SendError(e.to_string())),
            None => Err(WsError::NotConnected),
        }
    }

    async fn receive(&self) -> Result<WsMessage, WsError> {
        if *self.state.read().unwrap() != ConnectionState::Connected {
            return Err(WsError::NotConnected);
        }

        let recv_rx = self.recv_rx.as_ref().ok_or(WsError::NotConnected)?;
        let mut guard = recv_rx.lock().await;

        guard
            .recv()
            .await
            .ok_or_else(|| WsError::ReceiveError("channel closed".to_string()))
    }

    fn state(&self) -> ConnectionState {
        *self.state.read().unwrap()
    }
}

/// connection_loop はバックグラウンドで WebSocket 接続管理とメッセージ転送を担当する。
/// 接続が切れた場合は設定に従って自動再接続を試みる。
async fn connection_loop(
    config: crate::config::WsConfig,
    state: Arc<RwLock<ConnectionState>>,
    mut send_rx: mpsc::Receiver<TungsteniteMessage>,
    recv_tx: mpsc::Sender<WsMessage>,
    connected_tx: tokio::sync::oneshot::Sender<Result<(), WsError>>,
) {
    // 初回接続を試みる
    let ws = match connect_async(&config.url).await {
        Ok((ws, _)) => ws,
        Err(e) => {
            *state.write().unwrap() = ConnectionState::Disconnected;
            let _ = connected_tx.send(Err(WsError::ConnectionError(e.to_string())));
            return;
        }
    };

    *state.write().unwrap() = ConnectionState::Connected;
    // 接続成功を通知する（send が失敗した場合は connect() 側がタイムアウトしている）
    let _ = connected_tx.send(Ok(()));

    let (mut ws_write, mut ws_read) = ws.split();

    // Ping インターバルタイマーをセットアップする
    let mut ping_interval = config
        .ping_interval_ms
        .map(|ms| tokio::time::interval(tokio::time::Duration::from_millis(ms)));

    loop {
        // Ping タイマーの tick を取得する（設定がない場合は pending で待機する）
        let ping_tick = async {
            match ping_interval.as_mut() {
                Some(i) => {
                    i.tick().await;
                }
                None => std::future::pending::<()>().await,
            }
        };

        tokio::select! {
            // 送信メッセージをバックグラウンドタスクから WebSocket に転送する
            msg = send_rx.recv() => {
                match msg {
                    Some(m) => {
                        if ws_write.send(m).await.is_err() {
                            // 送信失敗は接続断として扱い、再接続ループに移行する
                            break;
                        }
                    }
                    // send_tx が drop された（disconnect() が呼ばれた）ため終了する
                    None => return,
                }
            }

            // WebSocket からのメッセージを受信チャネルに転送する
            msg = ws_read.next() => {
                match msg {
                    Some(Ok(m)) => {
                        if let Some(ws_msg) = tungstenite_to_ws(m) {
                            let _ = recv_tx.send(ws_msg).await;
                        }
                    }
                    // 接続が切れた場合は再接続ループに移行する
                    _ => break,
                }
            }

            // 定期 Ping を送信して接続を維持する
            _ = ping_tick => {
                if ws_write.send(TungsteniteMessage::Ping(vec![].into())).await.is_err() {
                    break;
                }
            }
        }
    }

    // 接続断後の再接続ループ
    reconnect_loop(config, state, send_rx, recv_tx).await;
}

/// reconnect_loop は接続断後に自動再接続を試みる。
/// 最大試行回数に達した場合は Disconnected 状態に遷移して終了する。
async fn reconnect_loop(
    config: crate::config::WsConfig,
    state: Arc<RwLock<ConnectionState>>,
    mut send_rx: mpsc::Receiver<TungsteniteMessage>,
    recv_tx: mpsc::Sender<WsMessage>,
) {
    if !config.reconnect {
        *state.write().unwrap() = ConnectionState::Disconnected;
        return;
    }

    for _attempt in 0..config.max_reconnect_attempts {
        *state.write().unwrap() = ConnectionState::Reconnecting;

        // 再接続前に待機する
        tokio::time::sleep(tokio::time::Duration::from_millis(config.reconnect_delay_ms)).await;

        match connect_async(&config.url).await {
            Ok((ws, _)) => {
                *state.write().unwrap() = ConnectionState::Connected;
                let (mut ws_write, mut ws_read) = ws.split();

                let mut ping_interval = config
                    .ping_interval_ms
                    .map(|ms| tokio::time::interval(tokio::time::Duration::from_millis(ms)));

                // 再接続後のメッセージ転送ループ。接続が切れたら outer for loop に戻る
                'msg_loop: loop {
                    let ping_tick = async {
                        match ping_interval.as_mut() {
                            Some(i) => {
                                i.tick().await;
                            }
                            None => std::future::pending::<()>().await,
                        }
                    };

                    tokio::select! {
                        msg = send_rx.recv() => {
                            match msg {
                                Some(m) => {
                                    if ws_write.send(m).await.is_err() {
                                        break 'msg_loop;
                                    }
                                }
                                None => return,
                            }
                        }
                        msg = ws_read.next() => {
                            match msg {
                                Some(Ok(m)) => {
                                    if let Some(ws_msg) = tungstenite_to_ws(m) {
                                        let _ = recv_tx.send(ws_msg).await;
                                    }
                                }
                                _ => break 'msg_loop,
                            }
                        }
                        _ = ping_tick => {
                            if ws_write.send(TungsteniteMessage::Ping(vec![].into())).await.is_err() {
                                break 'msg_loop;
                            }
                        }
                    }
                }
                // 再接続後に再度切断された場合は次の試行（continue）に進む
            }
            Err(_) => {
                // 接続失敗：次の試行を継続する
            }
        }
    }

    // 最大試行回数に達した場合は Disconnected に遷移する
    *state.write().unwrap() = ConnectionState::Disconnected;
}

/// ws_to_tungstenite は WsMessage を tungstenite の Message に変換する。
fn ws_to_tungstenite(msg: WsMessage) -> Result<TungsteniteMessage, WsError> {
    match msg {
        WsMessage::Text(s) => Ok(TungsteniteMessage::Text(s.into())),
        WsMessage::Binary(b) => Ok(TungsteniteMessage::Binary(b.into())),
        WsMessage::Ping(b) => Ok(TungsteniteMessage::Ping(b.into())),
        WsMessage::Pong(b) => Ok(TungsteniteMessage::Pong(b.into())),
        WsMessage::Close(frame) => {
            let close_frame = frame.map(|f| {
                tokio_tungstenite::tungstenite::protocol::CloseFrame {
                    code: f.code.into(),
                    reason: f.reason.into(),
                }
            });
            Ok(TungsteniteMessage::Close(close_frame))
        }
    }
}

/// tungstenite_to_ws は tungstenite の Message を WsMessage に変換する。
/// Frame メッセージは内部処理用のため None を返す。
fn tungstenite_to_ws(msg: TungsteniteMessage) -> Option<WsMessage> {
    match msg {
        TungsteniteMessage::Text(s) => Some(WsMessage::Text(s.to_string())),
        TungsteniteMessage::Binary(b) => Some(WsMessage::Binary(b.to_vec())),
        TungsteniteMessage::Ping(b) => Some(WsMessage::Ping(b.to_vec())),
        TungsteniteMessage::Pong(b) => Some(WsMessage::Pong(b.to_vec())),
        TungsteniteMessage::Close(frame) => {
            let close_frame = frame.map(|f| CloseFrame {
                code: f.code.into(),
                reason: f.reason.to_string(),
            });
            Some(WsMessage::Close(close_frame))
        }
        // Frame は内部処理用のため無視する
        TungsteniteMessage::Frame(_) => None,
    }
}
