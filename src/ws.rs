use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
// use futures::{SinkExt, StreamExt};
use sqlx::PgPool;
use tracing::info;

pub async fn web_socket_handler(
    ws: WebSocketUpgrade,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        let pool = pool.clone();
        async move {
            handle_socket(socket, pool).await;
        }
    })
}

async fn handle_socket(mut socket: WebSocket, pool: PgPool) {
    info!("Web socket connection");

    //Load previous messages

    let messages = sqlx::query!("SELECT username, message FROM messages ORDER BY timestamp ASC")
        .fetch_all(&pool)
        .await
        .unwrap();

    for msg in messages {
        let _ = socket
            .send(axum::extract::ws::Message::Text(format!(
                "{}: {}",
                msg.username, msg.message
            )))
            .await;
    }

    while let Some(Ok(msg)) = socket.recv().await {
        if let axum::extract::ws::Message::Text(text) = msg {
            info!("Received: {}", text);

            // Save message in the database
            if let Err(e) = sqlx::query!(
                "INSERT INTO messages (username, message) VALUES ($1, $2)",
                "User1",
                text
            )
            .execute(&pool)
            .await
            {
                info!("Failed to save message: {}", e);
            }

            if let Err(e) = socket
                .send(axum::extract::ws::Message::Text(format!("Echo: {}", text)))
                .await
            {
                info!("Error sending message: {}", e);
                break;
            }

            // Send the message back to the client
            let _ = socket
                .send(axum::extract::ws::Message::Text(format!("User1: {}", text)))
                .await;
        }
    }
}

pub fn ws_routes(pool: PgPool) -> Router {
    Router::new()
        .route("/ws", get(web_socket_handler))
        .with_state(pool)
}
