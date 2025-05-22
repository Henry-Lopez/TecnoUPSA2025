//! --------------------------------------------------------------
//! Envía la jugada al backend (`POST /api/jugada`) cuando
//! TurnFinishedEvent se dispara.
//!
//! Ahora usa el recurso `NextTurn` para mandar *el número de turno
//! lógico* (1-N) en lugar del UID del jugador.
//! --------------------------------------------------------------

use bevy::prelude::*;
use serde::Serialize;
use serde_json::json;

use crate::{
    components::PlayerDisk,
    events::TurnFinishedEvent,
    resources::{BackendInfo, TurnState, NextTurn},
};

#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

/* ------ Payload que espera el backend --------------------------------- */
#[derive(Serialize)]
struct TurnPayload {
    id_partida:   i32,
    numero_turno: i32,                  // ← número secuencial 1-N
    id_usuario:   i32,
    jugada:       serde_json::Value,
}

/* ------ Sistema -------------------------------------------------------- */
pub fn send_turn_to_backend(
    mut ev_end: EventReader<TurnFinishedEvent>,
    backend      : Res<BackendInfo>,
    _turn_state  : Res<TurnState>,
    next_turn    : Res<NextTurn>,         // ← contador correcto
    query        : Query<(&Transform, &PlayerDisk)>,
) {
    for _ in ev_end.read() {
        let piezas = query.iter()
            .map(|(t, disk)| json!({
                "id_usuario_real": disk.id_usuario_real,  // ✅ UID real
                "x": t.translation.x,
                "y": t.translation.y
            }))
            .collect::<Vec<_>>();

        let payload = TurnPayload {
            id_partida   : backend.partida_id,
            numero_turno : next_turn.0,      // ✅ 1-N
            id_usuario   : backend.my_uid,
            jugada       : json!({ "piezas": piezas }),
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


        /* En nativo simplemente imprimimos para depurar                    */
        #[cfg(not(target_arch = "wasm32"))]
        info!("▶️  (nativo) Se habría enviado turno #{:?}", payload.numero_turno);
    }
}
