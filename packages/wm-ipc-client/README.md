# wm-ipc-client

## Component Role
Asynchronous WebSocket client library for establishing IPC communication with the local GlazeWM window manager server instance.

## Dependency Graph
- **Inherited (`workspace = true`)**:
  - `anyhow` (v1, workspace features: `["backtrace"]`)
  - `futures-util` (v0.3)
  - `serde_json` (v1, workspace features: `["raw_value"]`)
  - `tokio` (v1, workspace features: `["full"]`)
  - `tokio-tungstenite` (v0.26)
  - `uuid` (v1, workspace features: `["v4", "serde"]`)
- **Local Workspace Peers**:
  - `wm-common` (`path = "../wm-common"`)

## Public API

### Structs
- `pub struct IpcClient`
  - Encapsulates an asynchronous WebSocket stream (`tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>`).

### Public Methods
- `pub async fn connect() -> anyhow::Result<Self>`
  - Establishes a WebSocket connection to `ws://127.0.0.1:{DEFAULT_IPC_PORT}`.
- `pub async fn send(&mut self, message: &str) -> anyhow::Result<()>`
  - Transmits a UTF-8 text message over the WebSocket stream.
- `pub async fn next_message(&mut self) -> anyhow::Result<wm_common::ServerMessage>`
  - Receives the next incoming WebSocket text frame and deserializes it into `wm_common::ServerMessage`.
- `pub async fn client_response(&mut self, client_message: &str) -> Option<wm_common::ClientResponseMessage>`
  - Continually polls incoming server messages until a `ServerMessage::ClientResponse` matching `client_message` is encountered.
- `pub async fn event_subscription(&mut self, subscription_id: &uuid::Uuid) -> Option<wm_common::EventSubscriptionMessage>`
  - Continually polls incoming server messages until a `ServerMessage::EventSubscription` matching `subscription_id` is encountered.

## Architecture & Modules
- `wm-ipc-client` (`src/lib.rs`)
  - Root crate entry point containing `IpcClient` implementation and stream message handling primitives.

## Execution Context
- **Runtime Requirement**: Tokio asynchronous runtime (`tokio::net::TcpStream`, `connect_async`).
- **Sync/Async Boundaries**: Fully asynchronous API using `async/await` syntax and `futures_util::{SinkExt, StreamExt}` traits.
- **State Mutability**: Mutable reference (`&mut self`) required for operations modifying socket state or reading stream frames.
- **Locking & Concurrency**: Unlocked, single-stream handle. Concurrent reads/writes across task boundaries require stream splitting or outer synchronizations (e.g., `Arc<Mutex<..>>`).
