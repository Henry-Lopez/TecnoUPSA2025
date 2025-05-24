use bevy::prelude::*;
use serde::Serialize;
use serde_json::json;

use crate::{
    components::PlayerDisk,
    events::TurnFinishedEvent,
    resources::{BackendInfo, TurnState},
    snapshot::{NextTurn, MyTurn},
};

// 🔄 NUEVO recurso global para guardar la jugada pendiente
#[derive(Resource, Default)]
pub struct PendingTurn(pub Option<TurnPayload>);

// 📦 Estructura del payload que se envía al backend
#[derive(Serialize, Clone)]
pub struct TurnPayload {
    pub id_partida: i32,
    pub numero_turno: i32,
    pub id_usuario: i32,
    pub jugada: serde_json::Value,
}

// 📨 En lugar de enviar, solo guarda la jugada
pub fn send_turn_to_backend(
    mut ev_end: EventReader<TurnFinishedEvent>,
    backend: Res<BackendInfo>,
    _turn_state: Res<TurnState>,
    next_turn: Res<NextTurn>,
    query: Query<(Entity, &Transform, &PlayerDisk)>,
    mut commands: Commands,
) {
    for _ in ev_end.read() {
        let piezas = query.iter()
            .map(|(entity, transform, disk)| json!({
                "id": entity.index(),
                "id_usuario_real": disk.id_usuario_real,
                "x": transform.translation.x,
                "y": transform.translation.y
            }))
            .collect::<Vec<_>>();

        let payload = TurnPayload {
            id_partida: backend.partida_id,
            numero_turno: next_turn.0,
            id_usuario: backend.my_uid,
            jugada: json!({ "piezas": piezas }),
        };

        // ⛔ No enviar aún — solo guardar
        commands.insert_resource(PendingTurn(Some(payload)));

        // 📴 Desactivar turno hasta nuevo snapshot
        commands.insert_resource(MyTurn(false));
    }
}

#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

/// Envía la jugada pendiente cuando vuelva a ser tu turno
pub fn maybe_send_pending_turn(
    my_turn: Res<MyTurn>,
    mut pending: ResMut<PendingTurn>,
) {
    if !my_turn.0 {
        return;
    }

    if let Some(payload) = pending.0.take() {
        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            let _ = Request::post("/api/jugada")
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&payload).unwrap())
                .unwrap()
                .send()
                .await;
        });

        #[cfg(not(target_arch = "wasm32"))]
        info!("▶️  (nativo) Jugada enviada desde PendingTurn");
    }
}
