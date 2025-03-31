use crate::entity::{messages, users, Messages, Users};
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::info;
type SharedState = Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>;

pub async fn web_socket_handler(
    ws: WebSocketUpgrade,
    State((db, state)): State<(DatabaseConnection, SharedState)>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        let db = db.clone();
        async move {
            handle_socket(socket, db, state).await;
        }
    })
}

pub async fn handle_socket(socket: WebSocket, db: DatabaseConnection, state: SharedState) {
    info!("WebSocket connection");

    // Split the socket into a sender and receiver
    let (mut sender, mut receiver) = socket.split();

    let username = match receiver.next().await {
        Some(Ok(Message::Text(name))) => {
            let name = name.trim().to_string();
            info!("User {} connected", name);

            // Check if user exists in the database, create if not
            let user = Users::find()
                .filter(users::Column::Username.eq(&name))
                .one(&db)
                .await
                .unwrap();

            if user.is_none() {
                // Create new user
                let user = users::ActiveModel {
                    username: Set(name.clone()),
                    ..Default::default()
                };
                let _ = user.insert(&db).await.unwrap();
            }

            name
        }
        _ => {
            // Invalid connection attempt without username
            return;
        }
    };

    // Send message history to the user
    send_message_history(&mut sender, &username, &db).await;

    // Add user to shared state with a broadcast channel
    let (tx, _rx) = broadcast::channel::<String>(10);
    {
        let mut state_guard = state.lock().await;
        state_guard.insert(username.clone(), tx.clone());

        info!(
            "📡 Current online users: {:?}",
            state_guard.keys().collect::<Vec<_>>()
        );
    }

    // Subscribe to the channel
    let mut rx = tx.subscribe();

    // Spawn a task to listen for broadcast messages and send them to the WebSocket
    let user_clone = username.clone();
    let sender_handle = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            info!("🔔 Delivering message to {}: {}", user_clone, msg);
            if let Err(e) = sender.send(Message::Text(msg)).await {
                info!("❌ Error sending message to {}: {}", user_clone, e);
                break;
            }
        }
    });

    // Handle incoming messages from the user
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            let parts: Vec<&str> = text.splitn(2, ':').collect();
            if parts.len() == 2 {
                let recipient = parts[0].trim().to_string();
                let message_content = parts[1].trim().to_string();
                let full_message = format!("{}: {}", username, message_content);

                // Get user IDs
                let sender_user = Users::find()
                    .filter(users::Column::Username.eq(&username))
                    .one(&db)
                    .await
                    .unwrap()
                    .unwrap();

                let recipient_user = Users::find()
                    .filter(users::Column::Username.eq(&recipient))
                    .one(&db)
                    .await
                    .unwrap();

                match recipient_user {
                    Some(recipient_user) => {
                        // Send message to recipient if they are online
                        if let Some(receiver_tx) = state.lock().await.get(&recipient) {
                            let _ = receiver_tx.send(full_message.clone());
                        }

                        // Also send to sender so they see their own messages
                        if let Some(sender_tx) = state.lock().await.get(&username) {
                            if recipient != username {
                                // Only if not sending to self
                                let _ = sender_tx.send(full_message.clone());
                            }
                        }

                        // Store message in the database
                        let message = messages::ActiveModel {
                            sender_id: Set(sender_user.id),
                            receiver_id: Set(recipient_user.id),
                            message: Set(message_content),
                            ..Default::default()
                        };

                        let _ = message.insert(&db).await.unwrap();
                    }
                    None => {
                        // Notify sender that recipient doesn't exist
                        if let Some(sender_tx) = state.lock().await.get(&username) {
                            let _ = sender_tx
                                .send(format!("System: User '{}' does not exist", recipient));
                        }
                    }
                }
            }
        }
    }

    // Clean up when user disconnects
    let _ = sender_handle.abort(); // Abort the sender task
    state.lock().await.remove(&username);
    info!("User {} disconnected", username);
}

// Function to retrieve and send message history
async fn send_message_history(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    username: &str,
    db: &DatabaseConnection,
) {
    info!("Retrieving message history for {}", username);

    // Get current user
    let user = Users::find()
        .filter(users::Column::Username.eq(username))
        .one(db)
        .await
        .unwrap()
        .unwrap();

    // Get distinct conversation partners
    // Query for users where the current user is the sender
    let sent_partners: Vec<i32> = Messages::find()
        .filter(messages::Column::SenderId.eq(user.id))
        .select_only()
        .column(messages::Column::ReceiverId)
        .group_by(messages::Column::ReceiverId)
        .into_tuple::<i32>()
        .all(db)
        .await
        .unwrap();

    // Query for users where the current user is the receiver
    let received_partners: Vec<i32> = Messages::find()
        .filter(messages::Column::ReceiverId.eq(user.id))
        .select_only()
        .column(messages::Column::SenderId)
        .group_by(messages::Column::SenderId)
        .into_tuple::<i32>()
        .all(db)
        .await
        .unwrap();

    // Merge and remove duplicates
    let mut conversation_partners: Vec<i32> = sent_partners;
    conversation_partners.extend(received_partners);
    conversation_partners.sort();
    conversation_partners.dedup();

    if conversation_partners.is_empty() {
        return;
    }

    // Send a history marker to indicate start of history
    let _ = sender
        .send(Message::Text("--- Message History ---".to_string()))
        .await;

    // Process each conversation partner
    for partner_id in conversation_partners {
        // Get partner username
        let partner = Users::find_by_id(partner_id)
            .one(db)
            .await
            .unwrap()
            .unwrap();

        // Send conversation header
        let _ = sender
            .send(Message::Text(format!(
                "--- Conversation with {} ---",
                partner.username
            )))
            .await;

        // Get messages with this partner (limited to 50 most recent)
        let messages = Messages::find()
            .filter(
                sea_orm::Condition::any()
                    .add(
                        sea_orm::Condition::all()
                            .add(messages::Column::SenderId.eq(user.id))
                            .add(messages::Column::ReceiverId.eq(partner_id)),
                    )
                    .add(
                        sea_orm::Condition::all()
                            .add(messages::Column::SenderId.eq(partner_id))
                            .add(messages::Column::ReceiverId.eq(user.id)),
                    ),
            )
            .order_by(messages::Column::CreatedAt, sea_orm::Order::Asc)
            .limit(50)
            .all(db)
            .await
            .unwrap();

        // Process and send each message
        for msg in messages {
            // Find sender username
            let sender_name = Users::find_by_id(msg.sender_id)
                .one(db)
                .await
                .unwrap()
                .unwrap()
                .username;

            // Format and send message
            let formatted_msg = format!("{}: {}", sender_name, msg.message);
            let _ = sender.send(Message::Text(formatted_msg)).await;
        }
    }

    // Send end-of-history marker
    let _ = sender
        .send(Message::Text("--- End of History ---".to_string()))
        .await;
}

pub fn ws_routes(db: DatabaseConnection) -> Router {
    let shared_state: SharedState = Arc::new(Mutex::new(HashMap::new()));

    Router::new()
        .route("/ws", get(web_socket_handler))
        .with_state((db, shared_state))
}
