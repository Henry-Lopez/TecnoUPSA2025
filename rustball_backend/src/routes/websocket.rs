//! routes/websocket.rs
//! Mejorado: Canales por partida, validación, pings, snapshot etiquetado, filtro por uid y snapshot en memoria

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension, Path,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use futures_util::{SinkExt, StreamExt};
use once_cell::sync::OnceCell;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::{
    sync::broadcast::{self, error::RecvError, Receiver, Sender},
    time,
};
use tracing::{debug, error, info, warn};

use crate::handlers::get_snapshot;
use axum::extract::Path as AxumPath;
use http_body_util::BodyExt;

static PARTIDA_CHANNELS: OnceCell<Mutex<HashMap<i32, Sender<String>>>> = OnceCell::new();
static LAST_SNAPSHOTS: OnceCell<Mutex<HashMap<i32, String>>> = OnceCell::new(); // 🆕

fn get_or_create_channel(partida: i32) -> Sender<String> {
    let map = PARTIDA_CHANNELS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = map.lock().unwrap();
    guard
        .entry(partida)
        .or_insert_with(|| {
            let (tx, _rx) = broadcast::channel::<String>(100);
            tx
        })
        .clone()
}

// 🧹 Elimina el canal si ya no hay nadie conectado
fn remove_channel_if_empty(partida: i32) {
    if let Some(map) = PARTIDA_CHANNELS.get() {
        let mut guard = map.lock().unwrap();
        if let Some(tx) = guard.get(&partida) {
            if tx.receiver_count() == 0 {
                guard.remove(&partida);
                info!("🧹 Canal de partida {} eliminado por estar vacío.", partida);
            }
        }
    }
}

// 🧠 Guarda el último snapshot en memoria
pub fn save_last_snapshot(partida: i32, snapshot_json: String) {
    let map = LAST_SNAPSHOTS.get_or_init(|| Mutex::new(HashMap::new()));
    map.lock().unwrap().insert(partida, snapshot_json);
}

// 📦 Obtiene el último snapshot de memoria (si existe)
fn get_last_snapshot(partida: i32) -> Option<String> {
    let map = LAST_SNAPSHOTS.get_or_init(|| Mutex::new(HashMap::new()));
    map.lock().unwrap().get(&partida).cloned()
}

/// Handler de la ruta `/ws/:partida/:uid`
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path((partida, uid)): Path<(i32, i32)>,
    Extension(pool): Extension<MySqlPool>,
) -> impl IntoResponse {
    info!("🌐 WS-OPEN partida={} uid={}", partida, uid);

    let ok: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) as count FROM FormacionElegida WHERE id_usuario = ? AND id_partida = ?",
        uid,
        partida
    )
        .fetch_one(&pool)
        .await
        .unwrap_or(0);

    if ok == 0 {
        info!("🚫 WS-RECHAZADO: uid={} no pertenece a partida={}", uid, partida);
        return StatusCode::FORBIDDEN.into_response();
    }

    let tx = get_or_create_channel(partida);
    ws.on_upgrade(move |socket| client_session(socket, partida, uid, tx, pool.clone()))
}

async fn client_session(
    socket: WebSocket,
    partida: i32,
    uid: i32,
    tx: Sender<String>,
    pool: MySqlPool,
) {
    let (mut outbound, mut inbound) = socket.split();
    let mut rx: Receiver<String> = tx.subscribe();
    let mut ping_interval = time::interval(time::Duration::from_secs(30));

    let forward = tokio::spawn({
        async move {
            loop {
                tokio::select! {
                    _ = ping_interval.tick() => {
                        let _ = outbound.send(Message::Ping(b"ping".to_vec())).await;
                    }
                    msg = rx.recv() => {
                        match msg {
                            Ok(text) => {
                                if let Ok(json_msg) = serde_json::from_str::<Value>(&text) {
                                    if json_msg["uid_origen"] == json!(uid) {
                                        continue;
                                    }
                                }
                                if outbound.send(Message::Text(text)).await.is_err() {
                                    error!("❌ Error enviando a WS uid={}", uid);
                                    break;
                                }
                            }
                            Err(RecvError::Lagged(n)) => {
                                warn!("⚠️  WS lag ({} mensajes perdidos) uid={}", n, uid);
                                if let Some(snapshot_str) = get_last_snapshot(partida) {
                                    let snapshot_wrapped = json!({
                                        "uid_origen": 0,
                                        "tipo": "snapshot",
                                        "contenido": serde_json::from_str::<Value>(&snapshot_str).unwrap_or(json!({}))
                                    });
                                    let _ = outbound.send(Message::Text(snapshot_wrapped.to_string())).await;
                                } else {
                                    warn!("📭 No hay snapshot en memoria para partida={}", partida);
                                }
                            }
                            Err(RecvError::Closed) => {
                                warn!("🔒 Canal cerrado para uid={}", uid);
                                break;
                            }
                        }
                    }
                }
            }
        }
    });

    while let Some(result) = inbound.next().await {
        match result {
            Ok(Message::Text(txt)) => {
                debug!("📨 part={} uid={} → {}", partida, uid, txt);
                let contenido_json: Value = serde_json::from_str(&txt).unwrap_or(json!(null));
                let mensaje = json!({
                "uid_origen": uid,
                "contenido": contenido_json
            });
                if tx.send(mensaje.to_string()).is_err() {
                    warn!("📴 Nadie suscrito WS uid={}", uid);
                }
            }
            Ok(Message::Close(reason)) => {
                info!("❌ Cliente cerró WS uid={} razón={:?}", uid, reason);
                break;
            }
            Ok(_) => {}
            Err(e) => {
                error!("❌ Error en WS uid={}: {}", uid, e);
                break;
            }
        }
    }

    forward.abort();
    info!("🔌 WS-CLOSE partida={} uid={}", partida, uid);
    remove_channel_if_empty(partida); // ✅ limpieza al desconectarse
}

// ❌ Ya no se usa: ahora usamos snapshot en memoria
#[allow(dead_code)]
async fn get_snapshot_json(partida: i32, pool: MySqlPool) -> Result<String, ()> {
    let response: Response = get_snapshot(AxumPath(partida), Extension(pool))
        .await
        .into_response();
    let body = response.into_body().collect().await.map_err(|_| ())?.to_bytes();
    String::from_utf8(body.to_vec()).map_err(|_| ())
}
