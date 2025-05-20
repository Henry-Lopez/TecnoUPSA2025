use axum::{
    extract::ws::{WebSocketUpgrade, WebSocket, Message},
    extract::{Extension, Path},
    response::IntoResponse,
};
use tokio::sync::broadcast;
use axum::debug_handler;
use futures_util::stream::StreamExt;
use futures_util::sink::SinkExt;

#[debug_handler]
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path((partida_id, user_id)): Path<(u32, u32)>,
    Extension(tx): Extension<broadcast::Sender<String>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, partida_id, user_id, tx))
}

async fn handle_socket(
    socket: WebSocket,
    partida_id: u32,
    user_id: u32,
    tx: broadcast::Sender<String>,
) {
    let (mut sender, mut receiver_ws) = socket.split();
    let mut rx = tx.subscribe();

    // Enviar mensajes del canal al cliente
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let _ = sender.send(Message::Text(msg)).await;
        }
    });

    // Escuchar mensajes del cliente
    while let Some(Ok(msg)) = receiver_ws.next().await {
        if let Message::Text(text) = msg {
            println!("📨 {user_id} en partida {partida_id}: {text}");
            let _ = tx.send(text);
        }
    }
}
