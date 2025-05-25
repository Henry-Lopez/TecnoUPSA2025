use bevy::prelude::*;
use serde::Serialize;
use serde_json::json;

use crate::{
    components::PlayerDisk,
    events::TurnFinishedEvent,
    resources::{BackendInfo, TurnState},
    snapshot::{NextTurn, MyTurn},
};

// 🔄 Recurso global que guarda la jugada pendiente
#[derive(Resource, Default)]
pub struct PendingTurn(pub Option<TurnPayload>);

// 📦 Estructura del payload que se envía al backend
#[derive(Serialize, Clone, Debug)]
pub struct TurnPayload {
    pub id_partida: i32,
    pub numero_turno: i32,
    pub id_usuario: i32,
    pub jugada: serde_json::Value,
}

// 📤 Armado del TurnPayload al finalizar el turno
pub fn send_turn_to_backend(
    mut ev_end: EventReader<TurnFinishedEvent>,
    backend: Res<BackendInfo>,
    _turn_state: Res<TurnState>,
    next_turn: Res<NextTurn>,
    query: Query<(Entity, &Transform, &PlayerDisk)>,
    mut commands: Commands,
) {
    for _ in ev_end.read() {
        info!("📤 Evento TurnFinished recibido. UID actual: {}", backend.my_uid);

        let piezas: Vec<_> = query
            .iter()
            .map(|(entity, transform, disk)| {
                json!({
                    "id": entity.index(),
                    "id_usuario_real": disk.id_usuario_real,
                    "x": transform.translation.x,
                    "y": transform.translation.y
                })
            })
            .collect();

        if piezas.is_empty() {
            warn!("⚠️ No se encontraron piezas en el Query. No se enviará jugada.");
            return;
        }

        let payload = TurnPayload {
            id_partida: backend.partida_id,
            numero_turno: next_turn.0,
            id_usuario: backend.my_uid,
            jugada: json!({ "piezas": piezas }),
        };

        info!("✅ Jugada lista para enviar:");
        info!("📦 id_partida = {}", payload.id_partida);
        info!("👤 id_usuario = {}", payload.id_usuario);
        info!("🔢 numero_turno = {}", payload.numero_turno);
        info!("📐 jugada = {}", payload.jugada);

        commands.insert_resource(PendingTurn(Some(payload)));
        commands.insert_resource(MyTurn(false));
    }
}

#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

// 🚀 Enviar jugada si es mi turno y hay jugada pendiente
pub fn maybe_send_pending_turn(
    my_turn: Res<MyTurn>,
    mut pending: ResMut<PendingTurn>,
) {
    if let Some(payload) = pending.0.take() {
        if !my_turn.0 {
            info!("⌛ Jugada armada antes del turno. Esperando activación.");
            pending.0 = Some(payload); // volver a guardarla para cuando toque
            return;
        }

        info!("📬 Enviando jugada POST al backend:");
        info!("📦 id_partida = {}", payload.id_partida);
        info!("👤 id_usuario = {}", payload.id_usuario);
        info!("🔢 numero_turno = {}", payload.numero_turno);
        info!("📐 jugada = {}", payload.jugada);

        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            let json = serde_json::to_string(&payload).unwrap();
            let req = Request::post("/api/jugada")
                .header("Content-Type", "application/json")
                .body(json.clone());

            let res = match req {
                Ok(r) => r.send().await,
                Err(e) => {
                    error!("❌ Error al construir petición POST /api/jugada: {:?}", e);
                    return;
                }
            };

            match res {
                Ok(resp) => {
                    let status = resp.status();
                    let text = resp.text().await.unwrap_or_else(|_| "❌ Sin cuerpo en respuesta".to_string());

                    if status >= 200 && status < 300 {
                        info!("✅ POST /api/jugada registrado con éxito ({}): {}", status, text);
                    } else {
                        error!("⚠️ POST /api/jugada falló ({}): {}", status, text);
                    }
                }
                Err(err) => {
                    error!("❌ Error de red al enviar jugada: {:?}", err);
                }
            }
        });
    }
}
