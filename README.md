# Rust WebSocket Chat Service

A real-time chat backend built using Rust, Axum, WebSockets, SeaORM, and PostgreSQL. This service allows users to authenticate, send private messages, and store chat history.

## 🚀 Features
- **User Authentication**: Users must provide a username to connect.
- **WebSocket Communication**: Real-time messaging using WebSockets.
- **Private Messaging**: Send messages directly to specific users.
- **Message Persistence**: Chat history is stored in PostgreSQL.
- **Online User Tracking**: Only online users receive instant messages.

---

## 🛠️ Tech Stack
- **Rust** (Axum for WebSocket handling)
- **Tokio** (Async runtime for Rust)
- **SQLx** (Database interactions)
- **PostgreSQL** (Relational database)
- **SeaORM** (Object Relational Mapper)
---

## 🏗️ Setup & Installation

### **1️⃣ Install Dependencies**
Ensure you have Rust and PostgreSQL installed.

```sh
# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install SQLx CLI for migrations
cargo install sqlx-cli --no-default-features --features postgres
```

### **2️⃣ Clone the Repository**
```sh
git clone https://github.com/yourusername/chat-service-rust.git
cd chat-service-rust
```

### **3️⃣ Setup Environment Variables**
Create a `.env` file and configure the PostgreSQL connection:
```env
DATABASE_URL=postgres://username:password@localhost/chat_db
```

### **4️⃣ Run Database Migrations**
```sh
sqlx database create
sqlx migrate run
```

### **5️⃣ Start the Server**
```sh
cargo run
```
Server runs on **http://127.0.0.1:3000**

---

## 🔌 WebSocket API

### **1️⃣ Connect to WebSocket**
```sh
wscat -c ws://127.0.0.1:3000/ws
```

### **2️⃣ Authenticate with a Username**
Send a username as the first message:
```sh
Alice
```

### **3️⃣ Send a Private Message**
Format: `recipient_username: message`
```sh
Bob: Hey Bob, how are you?
```

### **4️⃣ Receive Messages**
Bob will receive:
```sh
Alice: Hey Bob, how are you?
```

---

## 🛠️ Future Improvements
✅ JWT Authentication for users  
✅ Broadcast feature for group chats  
✅ Typing indicators  
✅ Read receipts  

---

## 📜 License
MIT License

---

## 📬 Contact
For issues or suggestions, feel free to open an issue or reach out!

