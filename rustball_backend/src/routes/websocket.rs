//! routes/websocket.rs
//! -----------------------------------------------------------------
//!   GET /ws/{partida}/{uid}  →  WebSocket broadcast
//!
//!   • Cada cliente envía texto        (Message::Text)
//!   • El servidor lo re-difunde a todos los suscritos
//! -----------------------------------------------------------------

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension, Path,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast::{error::RecvError, Sender};
use tracing::{debug, info, warn};

/// Handler de la ruta `/ws/:partida/:uid`
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path((partida, uid)): Path<(i32, i32)>,
    Extension(tx): Extension<Sender<String>>,
) -> impl IntoResponse {
    info!("🌐 WS-OPEN  partida={partida}  uid={uid}");
    // devolvemos directamente el upgrade
    ws.on_upgrade(move |socket| client_session(socket, partida, uid, tx))
}

/// Bucle principal de un cliente WebSocket
async fn client_session(socket: WebSocket, partida: i32, uid: i32, tx: Sender<String>) {
    let (mut outbound, mut inbound) = socket.split();
    let mut rx = tx.subscribe();

    /* ─── Task 1 : broadcast ➜ cliente ─────────────────────────── */
    let forward = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(text) => {
                    if outbound.send(Message::Text(text)).await.is_err() {
                        break; // cliente cerró
                    }
                }
                Err(RecvError::Lagged(n)) => {
                    warn!("⚠️  WS lag ({n} mensajes) uid={uid}");
                }
                Err(RecvError::Closed) => break,
            }
        }
    });

    /* ─── Task 2 : cliente ➜ broadcast ─────────────────────────── */
    while let Some(Ok(msg)) = inbound.next().await {
        match msg {
            Message::Text(txt) => {
                debug!("📨 WS  part={partida} uid={uid} → {txt}");
                let _ = tx.send(txt);           // ignorar error sin oyentes
            }
            Message::Close(_) => break,
            _ => {} // Ping/Pong/Bin… ignorados
        }
    }

    forward.abort();                            // cerramos tarea secundaria
    info!("🔌 WS-CLOSE partida={partida} uid={uid}");
}
