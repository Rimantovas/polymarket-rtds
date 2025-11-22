use crate::model::{Message, SubscriptionMessage};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::sync::mpsc;
use tokio::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};

const DEFAULT_HOST: &str = "wss://ws-live-data.polymarket.com";
const DEFAULT_PING_INTERVAL: u64 = 5000;

enum Command {
    Subscribe(SubscriptionMessage),
    Unsubscribe(SubscriptionMessage),
    Disconnect,
}

/// A client for managing real-time WebSocket connections, handling messages, subscriptions,
/// and automatic reconnections.
pub struct RealTimeDataClient {
    host: String,
    ping_interval: u64,
    command_tx: Option<mpsc::UnboundedSender<Command>>,
    message_rx: Option<mpsc::UnboundedReceiver<Result<Message, String>>>,
}

impl RealTimeDataClient {
    /// Creates a new client with default settings.
    pub fn new() -> Self {
        Self {
            host: DEFAULT_HOST.to_string(),
            ping_interval: DEFAULT_PING_INTERVAL,
            command_tx: None,
            message_rx: None,
        }
    }

    /// Establishes a WebSocket connection to the server.
    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (ws_stream, _) = connect_async(&self.host).await?;
        let (mut write, mut read) = ws_stream.split();

        let (command_tx, mut command_rx) = mpsc::unbounded_channel::<Command>();
        let (message_tx, message_rx) = mpsc::unbounded_channel::<Result<Message, String>>();

        self.command_tx = Some(command_tx);
        self.message_rx = Some(message_rx);

        let ping_interval = self.ping_interval;

        // Spawn the WebSocket handler
        tokio::spawn(async move {
            let mut ping_interval_timer = tokio::time::interval(Duration::from_millis(ping_interval));

            loop {
                tokio::select! {
                    Some(msg) = read.next() => {
                        match msg {
                            Ok(WsMessage::Text(text)) => {
                                if text.contains("payload") {
                                    if let Ok(message) = serde_json::from_str::<Message>(&text) {
                                        let _ = message_tx.send(Ok(message));
                                    }
                                }
                            }
                            Ok(WsMessage::Close(_)) => {
                                break;
                            }
                            Err(e) => {
                                eprintln!("WebSocket error: {}", e);
                                break;
                            }
                            _ => {}
                        }
                    }
                    Some(cmd) = command_rx.recv() => {
                        match cmd {
                            Command::Subscribe(msg) => {
                                let payload = json!({
                                    "action": "subscribe",
                                    "subscriptions": msg.subscriptions,
                                });
                                if write.send(WsMessage::Text(payload.to_string().into())).await.is_err() {
                                    eprintln!("Failed to send subscribe message");
                                    break;
                                }
                            }
                            Command::Unsubscribe(msg) => {
                                let payload = json!({
                                    "action": "unsubscribe",
                                    "subscriptions": msg.subscriptions,
                                });
                                if write.send(WsMessage::Text(payload.to_string().into())).await.is_err() {
                                    eprintln!("Failed to send unsubscribe message");
                                    break;
                                }
                            }
                            Command::Disconnect => {
                                let _ = write.send(WsMessage::Close(None)).await;
                                break;
                            }
                        }
                    }
                    _ = ping_interval_timer.tick() => {
                        if write.send(WsMessage::Text("ping".to_string().into())).await.is_err() {
                            eprintln!("Failed to send ping");
                            break;
                        }
                    }
                    else => break,
                }
            }
        });

        Ok(())
    }

    /// Receives the next message from the WebSocket connection.
    pub async fn recv(&mut self) -> Option<Result<Message, String>> {
        if let Some(ref mut rx) = self.message_rx {
            rx.recv().await
        } else {
            None
        }
    }

    /// Subscribes to data streams.
    pub async fn subscribe(&self, subscriptions: Vec<crate::model::Subscription>) -> Result<(), String> {
        let msg = SubscriptionMessage { subscriptions };
        if let Some(ref tx) = self.command_tx {
            tx.send(Command::Subscribe(msg))
                .map_err(|_| "Failed to send subscribe command".to_string())
        } else {
            Err("Socket not connected".to_string())
        }
    }

    /// Unsubscribes from data streams.
    pub async fn unsubscribe(&self, subscriptions: Vec<crate::model::Subscription>) -> Result<(), String> {
        let msg = SubscriptionMessage { subscriptions };
        if let Some(ref tx) = self.command_tx {
            tx.send(Command::Unsubscribe(msg))
                .map_err(|_| "Failed to send unsubscribe command".to_string())
        } else {
            Err("Socket not connected".to_string())
        }
    }

    /// Closes the WebSocket connection.
    pub async fn disconnect(&self) -> Result<(), String> {
        if let Some(ref tx) = self.command_tx {
            tx.send(Command::Disconnect)
                .map_err(|_| "Failed to send disconnect command".to_string())
        } else {
            Ok(())
        }
    }
}

impl Default for RealTimeDataClient {
    fn default() -> Self {
        Self::new()
    }
}