//! src/systems/poll_turn.rs   (nuevo nombre sugerido)

// ╭─────────────────────────── Imports ───────────────────────────╮
use bevy::prelude::*;
use gloo_net::http::Request;
use std::{
    sync::{
        Arc, Mutex,
    },
    time::Duration,
};
use wasm_bindgen_futures::spawn_local;

use crate::{
    resources::BackendInfo,
    snapshot::{MyTurn, TurnoData},
};

#[cfg(target_arch = "wasm32")]
use crate::snapshot::{set_game_state, SnapshotFromServer};
// ╰───────────────────────────────────────────────────────────────╯

/* ────────────── Recurso global ────────────── */
#[derive(Resource, Clone)]
pub struct PollState {
    timer: Timer,                        // ⏲️ Temporizador de 3 s
    last_turn_number: Arc<Mutex<i32>>,   // 🔁 Último turno procesado
}

impl Default for PollState {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_secs(3), TimerMode::Repeating),
            last_turn_number: Arc::new(Mutex::new(0)),
        }
    }
}

/* ────────────── Sistema de polling ──────────────
   • Si ES mi turno  → resetea timer y no consulta nada.
   • Si NO es mi turno
        – cada 3 s pide /api/estado
        – si detecta turno nuevo ⇒ pide /api/snapshot
        – aplica el snapshot con set_game_state
   Nótese que **ya no emite eventos**: el snapshot es suficiente
   para que el resto de la app (snapshot_apply_system) actualice
   MyTurn, tablero, marcador, etc.
   -------------------------------------------------------------- */
pub fn poll_turn_tick_system(
    mut state: ResMut<PollState>,
    time: Res<Time>,
    my_turn: Res<MyTurn>,
    backend_opt: Option<Res<BackendInfo>>,
) {
    // 1) Sin datos de backend ⇒ salir.
    let backend = match backend_opt {
        Some(b) => b,
        None => return,
    };

    // 2) Si es MI turno ⇒ reinicio timer y no hago polling.
    if my_turn.0 {
        state.timer.reset();
        return;
    }

    // 3) Avanzar el timer; si aún no venció, nada que hacer.
    state.timer.tick(time.delta());
    if !state.timer.finished() {
        return;
    }

    // 4) Polling asíncrono.
    let pid            = backend.partida_id;
    let uid            = backend.my_uid;
    let last_turn_ref  = Arc::clone(&state.last_turn_number);

    spawn_local(async move {
        // 4-A) Consultar estado resumido de los turnos.
        if let Ok(resp) = Request::get(&format!("/api/estado/{pid}")).send().await {
            if let Ok(turnos) = resp.json::<Vec<TurnoData>>().await {
                if let Some(ultimo) = turnos.last() {
                    let mut last = last_turn_ref.lock().unwrap();
                    if ultimo.numero_turno > *last {
                        *last = ultimo.numero_turno; // actualizar memoria

                        // 4-B) Hay turno nuevo ⇒ pedir snapshot completo.
                        if let Ok(r) = Request::get(&format!("/api/snapshot/{pid}")).send().await {
                            if let Ok(snap) = r.json::<SnapshotFromServer>().await {
                                if let Ok(json_str) = serde_json::to_string(&snap) {
                                    set_game_state(&json_str, uid);
                                }
                            }
                        }
                    }
                }
            }
        }
    });
}
