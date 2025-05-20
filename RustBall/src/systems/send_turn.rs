use bevy::prelude::*;
use serde::Serialize;
use serde_json::json;

use crate::events::TurnFinishedEvent;
use crate::resources::{BackendInfo, TurnState};
use crate::components::PlayerDisk;

#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

#[derive(Serialize)]
struct TurnPayload {
    id_partida:   i32,
    numero_turno: i32,
    id_usuario:   i32,
    jugada:       serde_json::Value,
}

pub fn send_turn_to_backend(
    mut ev_end: EventReader<TurnFinishedEvent>,
    backend: Res<BackendInfo>,
    turn_state: Res<TurnState>,
    query: Query<(Entity, &Transform), With<PlayerDisk>>,
) {
    for _ in ev_end.read() {
        let piezas: Vec<_> = query
            .iter()
            .map(|(e, t)| {
                json!({
                    "id": e.index(),
                    "x": t.translation.x,
                    "y": t.translation.y
                })
            })
            .collect();

        let payload = TurnPayload {
            id_partida: backend.partida_id,
            numero_turno: turn_state.current_turn as i32,
            id_usuario: backend.my_uid, // ✅ ahora correcto
            jugada: json!({ "piezas": piezas }),
        };

        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            let _ = Request::post("/api/jugada")
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&payload).unwrap())
                .unwrap()
                .send()
                .await;
        });
    }
}
