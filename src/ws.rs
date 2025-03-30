use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
// use futures::{SinkExt, StreamExt};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::info;

type SharedState = Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>;

pub async fn web_socket_handler(
    ws: WebSocketUpgrade,
    State((pool, state)): State<(PgPool, SharedState)>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        let pool = pool.clone();
        async move {
            handle_socket(socket, pool, state).await;
        }
    })
}

pub async fn handle_socket(mut socket: WebSocket, pool: PgPool, state: SharedState) {
    info!("WebSocket connection");

    let mut username = String::new();

    // Step 1: Identify the user
    if let Some(Ok(Message::Text(name))) = socket.recv().await {
        username = name.trim().to_string();
        info!("User {} connected", username);

        // Check if user exists in the database
        let user = sqlx::query!("SELECT id FROM users WHERE username = $1", username)
            .fetch_optional(&pool)
            .await
            .unwrap();

        if user.is_none() {
            sqlx::query!("INSERT INTO users (username) VALUES ($1)", username)
                .execute(&pool)
                .await
                .unwrap();
        }
    }

    // Step 2: Add user to shared state
    let (tx, mut rx) = broadcast::channel::<String>(10);
    state.lock().await.insert(username.clone(), tx.clone());

    // Step 3: Use Arc<tokio::sync::Mutex<WebSocket>> for async access
    let socket = Arc::new(Mutex::new(socket));

    // Spawn a task to listen for incoming messages for this user
    let socket_rx = Arc::clone(&socket);
    let user_clone = username.clone();
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let mut socket_guard = socket_rx.lock().await;
            if let Err(e) = socket_guard.send(Message::Text(msg)).await {
                info!("Error sending message to {}: {}", user_clone, e);
                break;
            }
        }
    });

    // Step 4: Handle incoming messages from the user
    while let Some(Ok(msg)) = socket.lock().await.recv().await {
        if let Message::Text(text) = msg {
            let parts: Vec<&str> = text.splitn(2, ':').collect();
            if parts.len() == 2 {
                let recipient = parts[0].trim().to_string();
                let message = parts[1].trim().to_string();
                let full_message = format!("{}: {}", username, message);

                // Send message to recipient if they are online
                if let Some(receiver_tx) = state.lock().await.get(&recipient) {
                    let _ = receiver_tx.send(full_message.clone());
                }

                // Store message in the database
                sqlx::query!(
                    "INSERT INTO messages (sender_id, receiver_id, message) 
                     VALUES ((SELECT id FROM users WHERE username=$1), 
                             (SELECT id FROM users WHERE username=$2), $3)",
                    username,
                    recipient,
                    message
                )
                .execute(&pool)
                .await
                .unwrap();
            }
        }
    }

    // Step 5: Remove user from shared state when they disconnect
    state.lock().await.remove(&username);
}

pub fn ws_routes(pool: PgPool) -> Router {
    let shared_state: SharedState = Arc::new(Mutex::new(HashMap::new()));

    Router::new()
        .route("/ws", get(web_socket_handler))
        .with_state((pool, shared_state))
}
