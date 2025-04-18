use crate::auth::verify_firebase_token;
use crate::entity::{messages, users, Messages, Users};
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::info;
type SharedState = Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>;

#[derive(Deserialize)]
pub struct WsParams {
    token: String,
}

pub async fn web_socket_handler(
    ws: WebSocketUpgrade,
    Query(WsParams { token }): Query<WsParams>,
    State((db, state)): State<(DatabaseConnection, SharedState)>,
) -> impl IntoResponse {
    match verify_firebase_token(&token).await {
        Ok(claims) => {
            let uid = claims.sub.clone(); // Firebase UID

            ws.on_upgrade(move |socket| {
                let db = db.clone();
                let state = state.clone();
                let uid = uid.clone();

                // return an async block
                async move {
                    handle_socket(socket, db, state, uid).await;
                }
            })
        }
        Err(_) => (
            axum::http::StatusCode::UNAUTHORIZED,
            "Invalid or expired token",
        )
            .into_response(),
    }
}

pub async fn handle_socket(
    socket: WebSocket,
    db: DatabaseConnection,
    state: SharedState,
    uid: String,
) {
    info!("WebSocket connection");

    // Split the socket into a sender and receiver
    let (mut sender, mut receiver) = socket.split();

    // Fetch user from DB using UID (Firebase UID is stored as username)
    let user = Users::find()
        .filter(users::Column::Username.eq(&uid))
        .one(&db)
        .await
        .unwrap();

    let user = match user {
        Some(user) => user,
        None => {
            info!("❌ No user found with UID {}, closing connection", uid);
            return;
        }
    };

    let username = user.username.clone();
    fn broadcast_status_update(
        state: &HashMap<String, broadcast::Sender<String>>,
        username: &str,
        is_online: bool,
    ) {
        let status_message = if is_online {
            format!("System: User '{}' is online", username)
        } else {
            format!("System: User '{}' went offline", username)
        };

        // Send status update to all connected users
        for (user, tx) in state {
            // Don't send the notification to the user who triggered it
            if user != username {
                let _ = tx.send(status_message.clone());
            }
        }
    }
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
                        // Mark message as delivered
                        mark_as_delivered(&db, sender_user.id, recipient_user.id)
                            .await
                            .unwrap();
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
                            status: Set("unread".to_string()),
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
                                   // Broadcast that user went offline before removing from state
    {
        let state_guard = state.lock().await;
        broadcast_status_update(&state_guard, &username, false);
    }
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

        mark_as_read(db, user.id, partner_id).await.unwrap();

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

async fn mark_as_delivered(
    db: &DatabaseConnection,
    sender_id: i32,
    receiver_id: i32,
) -> Result<(), sea_orm::DbErr> {
    // Build an ActiveModel with the updated value
    let active_model = messages::ActiveModel {
        status: Set("delivered".to_string()), // Set the status field
        ..Default::default()                  // Make sure the other fields are left as they are
    };

    // Update the matching records
    messages::Entity::update_many()
        .set(active_model) // Use the ActiveModel for the update
        .filter(messages::Column::SenderId.eq(sender_id))
        .filter(messages::Column::ReceiverId.eq(receiver_id))
        .filter(messages::Column::Status.eq("sent"))
        .exec(db)
        .await?;

    Ok(())
}

async fn mark_as_read(
    db: &DatabaseConnection,
    sender_id: i32,
    receiver_id: i32,
) -> Result<(), sea_orm::DbErr> {
    // Build an ActiveModel with the updated value
    let active_model = messages::ActiveModel {
        status: Set("read".to_string()), // Set the status field
        ..Default::default()             // Make sure the other fields are left as they are
    };

    // Update the matching records
    messages::Entity::update_many()
        .set(active_model) // Use the ActiveModel for the update
        .filter(messages::Column::SenderId.eq(sender_id))
        .filter(messages::Column::ReceiverId.eq(receiver_id))
        .filter(messages::Column::Status.eq("delivered"))
        .exec(db)
        .await?;

    Ok(())
}

pub fn ws_routes(db: DatabaseConnection) -> Router {
    let shared_state: SharedState = Arc::new(Mutex::new(HashMap::new()));

    Router::new()
        .route("/ws", get(web_socket_handler))
        .with_state((db, shared_state))
}
