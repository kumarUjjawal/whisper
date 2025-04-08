# 🗣️ Whisper — Rust WebSocket Chat Service

A real-time chat backend built using **Rust**, **Axum**, **WebSockets**, **SeaORM**, and **PostgreSQL**, with authentication powered by **Firebase Phone Auth**. This service enables users to authenticate via phone numbers, send private messages in real-time, and store chat history.

---

## 🚀 Features

- 📱 **Firebase Phone Auth**: Authenticate using phone numbers and verify tokens.
- 🔐 **JWT Verification**: Secured WebSocket access using Firebase ID tokens.
- 💬 **WebSocket Messaging**: Real-time private messaging with online user tracking.
- 💾 **Message Persistence**: All chats are saved in PostgreSQL via SeaORM.
- 🌐 **Cloud Deployment**: Easily deployable on [Railway](https://railway.app/).

---

## 🛠️ Tech Stack

- **Rust** – high-performance systems language  
- **Axum** – web framework for async HTTP & WebSockets  
- **Tokio** – async runtime  
- **SeaORM** – async ORM for PostgreSQL  
- **Firebase Auth** – phone number-based authentication  
- **Railway** – deployment platform for backend and database  

---

## 🏗️ Local Development Setup

### ✅ Prerequisites

- Rust 1.81+ installed
- PostgreSQL running locally or via Docker
- Firebase project with phone auth enabled

### 1️⃣ Clone the Repository

```bash
git clone https://github.com/yourusername/whisper.git
cd whisper
```

### 2️⃣ Environment Configuration

Create a `.env` file with the following contents:

```env
DATABASE_URL=postgres://username:password@localhost:5432/whisper
FIREBASE_API_KEY=your_firebase_web_api_key
JWT_SECRET=your_jwt_secret
```

### 3️⃣ Run Database Migrations (if using SeaORM CLI)

```bash
cargo install sea-orm-cli
sea-orm-cli migrate up
```

Or run your own migration setup as needed.

### 4️⃣ Run the Server

```bash
cargo run
```

The server will be available at: http://127.0.0.1:3000

---

## 🔌 WebSocket Usage

### Connect to WebSocket

```bash
wscat -c "ws://127.0.0.1:3000/ws?token=<FIREBASE_ID_TOKEN>"
```

### Send Private Message

Format:
```text
<recipient_id>: your message here
```

### Receive Messages

Messages from other users will be received in real-time if you're connected.

---

## 🚀 Deployment on Railway

### 1️⃣ Push Code to GitHub

Push your code to a public or private GitHub repo.

### 2️⃣ Create Railway Project

1. Go to [railway.app](https://railway.app)
2. Click New Project
3. Select Deploy from GitHub Repo
4. Choose your whisper repo

### 3️⃣ Add PostgreSQL Plugin

1. Inside your Railway project, click Add Plugin
2. Choose PostgreSQL
3. Copy the DATABASE_URL and add it to your environment variables

### 4️⃣ Set Environment Variables

Go to Variables tab and add:
- `DATABASE_URL` = value from PostgreSQL plugin
- `FIREBASE_API_KEY` = your Firebase Web API Key
- `JWT_SECRET` = your own secret

### 5️⃣ Done!

Railway will automatically build and deploy your project.

### 🔗 Production Endpoint

Example:
```bash
curl https://whisper-production-xxxx.up.railway.app/
```

Or WebSocket:
```bash
wscat -c "wss://whisper-production-xxxx.up.railway.app/ws?token=<FIREBASE_ID_TOKEN>"
```

---

## 🛠️ Future Improvements

- ⏳ Message read receipts
- ⏳ Online status syncing
- ⏳ Admin moderation tools

---

## 📜 License

MIT License

---

## 📬 Contact

For issues, suggestions, or collaboration, feel free to open an issue or reach out via GitHub!

---

Made with 🦀 and ☕
