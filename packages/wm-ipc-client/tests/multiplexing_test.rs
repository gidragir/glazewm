use futures_util::SinkExt;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use uuid::Uuid;
use wm_common::{
  ClientResponseMessage, EventSubscriptionMessage, ServerMessage, WmEvent,
};
use wm_ipc_client::IpcClient;

#[tokio::test]
async fn test_lossless_event_and_response_multiplexing() {
  let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
  let port = listener.local_addr().unwrap().port();

  let sub_id = Uuid::new_v4();

  let server_handle = tokio::spawn(async move {
    let (tcp_stream, _) = listener.accept().await.unwrap();
    let mut ws = accept_async(tcp_stream).await.unwrap();

    // 1. Send an EventSubscription message first
    let event1 = ServerMessage::EventSubscription(EventSubscriptionMessage {
      subscription_id: sub_id,
      data: Some(WmEvent::ApplicationExiting),
      error: None,
      success: true,
    });
    ws.send(Message::Text(serde_json::to_string(&event1).unwrap().into()))
      .await
      .unwrap();

    // 2. Send a ClientResponse message second
    let resp = ServerMessage::ClientResponse(ClientResponseMessage {
      client_message: "query workspaces".into(),
      data: None,
      error: None,
      success: true,
    });
    ws.send(Message::Text(serde_json::to_string(&resp).unwrap().into()))
      .await
      .unwrap();

    // 3. Send another EventSubscription message third
    let event2 = ServerMessage::EventSubscription(EventSubscriptionMessage {
      subscription_id: sub_id,
      data: Some(WmEvent::PauseChanged { is_paused: true }),
      error: None,
      success: true,
    });
    ws.send(Message::Text(serde_json::to_string(&event2).unwrap().into()))
      .await
      .unwrap();
  });

  let mut client =
    IpcClient::connect_to(&format!("ws://127.0.0.1:{port}"))
      .await
      .unwrap();

  // Client requests response for "query workspaces"
  let response = client.client_response("query workspaces").await;
  assert!(response.is_some(), "Client should receive the response");
  assert_eq!(response.unwrap().client_message, "query workspaces");

  // Client should STILL receive the first event (it should NOT have been dropped while polling for client_response)
  let received_event1 = client.event_subscription(&sub_id).await;
  assert!(
    received_event1.is_some(),
    "Event 1 must NOT be dropped while awaiting response"
  );
  assert!(
    matches!(
      received_event1.unwrap().data,
      Some(WmEvent::ApplicationExiting)
    ),
    "Event 1 data must match ApplicationExiting"
  );

  // Client should also receive the second event
  let received_event2 = client.event_subscription(&sub_id).await;
  assert!(received_event2.is_some(), "Event 2 must be received");
  assert!(
    matches!(
      received_event2.unwrap().data,
      Some(WmEvent::PauseChanged { is_paused: true })
    ),
    "Event 2 data must match PauseChanged"
  );

  server_handle.await.unwrap();
}
