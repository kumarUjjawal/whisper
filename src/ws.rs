use axum::{
    extract::ws::{Message as AxumMessage, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::StreamExt;
// use futures::{SinkExt, StreamExt};
use tracing::info;

pub async fn web_socket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    info!("Web socket connection");
    while let Some(Ok(msg)) = socket.next().await {
        match msg {
            AxumMessage::Text(text) => {
                info!("Received..{}", text);
                if let Err(e) = socket
                    .send(AxumMessage::Text(format!("Echo: {}", text)))
                    .await
                {
                    info!("Error sending message {}", e);
                    break;
                }
            }
            AxumMessage::Close(_) => {
                info!("Websocket closed");
                break;
            }
            _ => {}
        }
    }
}

pub fn ws_routes() -> Router {
    Router::new().route("/ws", get(web_socket_handler))
}
