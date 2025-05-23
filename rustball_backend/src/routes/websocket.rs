//! routes/websocket.rs
//! -----------------------------------------------------------------
//!   GET /ws/{partida}/{uid}  →  WebSocket broadcast
//!
//!   • Cada cliente envía texto        (Message::Text)
//!   • El servidor lo re-difunde a todos los suscritos
//!   • Se agregan logs y manejo robusto de errores
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
use tracing::{debug, error, info, warn};

/// Handler de la ruta `/ws/:partida/:uid`
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path((partida, uid)): Path<(i32, i32)>,
    Extension(tx): Extension<Sender<String>>,
) -> impl IntoResponse {
    info!("🌐 WS-OPEN  partida={partida}  uid={uid}");
    ws.on_upgrade(move |socket| client_session(socket, partida, uid, tx))
}

/// Bucle principal de un cliente WebSocket
async fn client_session(socket: WebSocket, partida: i32, uid: i32, tx: Sender<String>) {
    let (mut outbound, mut inbound) = socket.split();
    let mut rx = tx.subscribe();

    /* ─── Task 1 : Servidor ➜ Cliente ───────────────────────────── */
    let forward = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(text) => {
                    if let Err(e) = outbound.send(Message::Text(text)).await {
                        error!("❌ Error enviando a cliente WS uid={uid}: {e}");
                        break;
                    }
                }
                Err(RecvError::Lagged(n)) => {
                    warn!("⚠️  WS lag ({n} mensajes perdidos) uid={uid}");
                }
                Err(RecvError::Closed) => {
                    warn!("🔒 Canal cerrado para WS uid={uid}");
                    break;
                }
            }
        }
    });

    /* ─── Task 2 : Cliente ➜ Servidor ───────────────────────────── */
    while let Some(result) = inbound.next().await {
        match result {
            Ok(Message::Text(txt)) => {
                debug!("📨 WS  part={partida} uid={uid} → {txt}");
                if tx.send(txt).is_err() {
                    warn!("📴 Nadie suscrito al canal WS (uid={uid})");
                }
            }
            Ok(Message::Close(reason)) => {
                info!("❌ Cliente cerró WS uid={uid} razón={:?}", reason);
                break;
            }
            Ok(_) => {
                // Ignoramos Binary/Ping/Pong
            }
            Err(e) => {
                error!("❌ Error en mensaje WS uid={uid}: {e}");
                break;
            }
        }
    }

    forward.abort(); // cancela la tarea secundaria
    info!("🔌 WS-CLOSE partida={partida} uid={uid}");
}
