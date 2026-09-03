#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

use anyhow::{Context, bail};
use futures_util::{
  SinkExt, StreamExt,
  stream::SplitSink,
};
use tokio::{
  net::TcpStream,
  sync::{broadcast, mpsc},
  task::JoinHandle,
};
use tokio_tungstenite::{
  MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message,
};
use uuid::Uuid;
use wm_common::{
  ClientResponseMessage, DEFAULT_IPC_PORT, EventSubscriptionMessage,
  ServerMessage,
};

const BROADCAST_CHANNEL_CAPACITY: usize = 4096;

pub struct IpcClient {
  outgoing: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
  response_rx: mpsc::UnboundedReceiver<ClientResponseMessage>,
  event_rx: broadcast::Receiver<EventSubscriptionMessage>,
  raw_rx: broadcast::Receiver<ServerMessage>,
  reader_task: JoinHandle<()>,
}

impl IpcClient {
  pub async fn connect() -> anyhow::Result<Self> {
    Self::connect_to(&format!("ws://127.0.0.1:{DEFAULT_IPC_PORT}")).await
  }

  pub async fn connect_to(server_addr: &str) -> anyhow::Result<Self> {
    let (stream, _) = connect_async(server_addr)
      .await
      .context("Failed to connect to IPC server.")?;

    Ok(Self::from_stream(stream))
  }

  pub fn from_stream(
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
  ) -> Self {
    let (outgoing, mut incoming) = stream.split();

    let (response_tx, response_rx) = mpsc::unbounded_channel();
    let (event_tx, event_rx) =
      broadcast::channel(BROADCAST_CHANNEL_CAPACITY);
    let (raw_tx, raw_rx) =
      broadcast::channel(BROADCAST_CHANNEL_CAPACITY);

    let reader_task = tokio::spawn(async move {
      while let Some(msg_result) = incoming.next().await {
        let Ok(msg) = msg_result else {
          break;
        };

        if let Ok(text) = msg.to_text()
          && let Ok(server_msg) = serde_json::from_str::<ServerMessage>(text)
        {
          let _ = raw_tx.send(server_msg.clone());
          match server_msg {
            ServerMessage::ClientResponse(resp) => {
              let _ = response_tx.send(resp);
            }
            ServerMessage::EventSubscription(event) => {
              let _ = event_tx.send(event);
            }
          }
        }
      }
    });

    Self {
      outgoing,
      response_rx,
      event_rx,
      raw_rx,
      reader_task,
    }
  }

  /// Sends a message to the IPC server.
  pub async fn send(&mut self, message: &str) -> anyhow::Result<()> {
    self
      .outgoing
      .send(Message::Text(message.into()))
      .await
      .context("Failed to send command.")?;

    Ok(())
  }

  /// Waits and returns the next reply from the IPC server.
  pub async fn next_message(&mut self) -> anyhow::Result<ServerMessage> {
    loop {
      match self.raw_rx.recv().await {
        Ok(msg) => return Ok(msg),
        Err(broadcast::error::RecvError::Lagged(_)) => {}
        Err(broadcast::error::RecvError::Closed) => {
          bail!("Connection to IPC server closed");
        }
      }
    }
  }

  pub async fn client_response(
    &mut self,
    client_message: &str,
  ) -> Option<ClientResponseMessage> {
    while let Some(response) = self.response_rx.recv().await {
      if response.client_message == client_message {
        return Some(response);
      }
    }

    None
  }

  pub async fn event_subscription(
    &mut self,
    subscription_id: &Uuid,
  ) -> Option<EventSubscriptionMessage> {
    loop {
      match self.event_rx.recv().await {
        Ok(event) => {
          if &event.subscription_id == subscription_id {
            return Some(event);
          }
        }
        Err(broadcast::error::RecvError::Lagged(_)) => {}
        Err(broadcast::error::RecvError::Closed) => return None,
      }
    }
  }
}

impl Drop for IpcClient {
  fn drop(&mut self) {
    self.reader_task.abort();
  }
}
