use bevy::prelude::*;
use gloo_net::http::Request;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::Duration,
};
use wasm_bindgen_futures::spawn_local;

use crate::{
    events::TurnFinishedEvent,
    resources::BackendInfo,
    snapshot::{MyTurn, TurnoData},
};

/* ────────────── Recurso global ────────────── */
#[derive(Resource, Clone)]
pub struct PollState {
    timer: Timer,                        // ⏲️ Temporizador de 3s
    last_turn_number: Arc<Mutex<i32>>,   // 🔁 Último turno recibido
    notify: Arc<AtomicBool>,             // 🚩 Flag para notificar nuevo turno
}

impl Default for PollState {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_secs(3), TimerMode::Repeating),
            last_turn_number: Arc::new(Mutex::new(0)),
            notify: Arc::new(AtomicBool::new(false)),
        }
    }
}

/* ────────────── Sistema de polling ────────────── */
pub fn poll_turn_tick_system(
    mut state: ResMut<PollState>,
    time: Res<Time>,
    my_turn: Res<MyTurn>,
    backend_opt: Option<Res<BackendInfo>>,
    mut writer: EventWriter<TurnFinishedEvent>,
) {
    // 0) Si hay una nueva jugada detectada por el async, disparamos el evento
    if state.notify.swap(false, Ordering::Acquire) {
        writer.send(TurnFinishedEvent);
    }

    // 1) Sin información del backend, salimos
    let backend = match backend_opt {
        Some(b) => b,
        None => return,
    };

    // 2) Si es mi turno, reinicio el timer pero no hago polling
    if my_turn.0 {
        state.timer.reset();
        return;
    }

    // 3) Avanzar el timer
    state.timer.tick(time.delta());
    if !state.timer.finished() {
        return;
    }

    // 4) Ejecutar el polling async
    let pid = backend.partida_id;
    let notify_flag = Arc::clone(&state.notify);
    let last_turn_ref = Arc::clone(&state.last_turn_number);

    spawn_local(async move {
        if let Ok(resp) = Request::get(&format!("/api/estado/{pid}")).send().await {
            if let Ok(turnos) = resp.json::<Vec<TurnoData>>().await {
                if let Some(ultimo) = turnos.last() {
                    let mut last = last_turn_ref.lock().unwrap();
                    if ultimo.numero_turno > *last {
                        notify_flag.store(true, Ordering::Release);
                        *last = ultimo.numero_turno;
                    }
                }
            }
        }
    });
}

/* ────────────── Aplicar snapshot al recibir TurnFinishedEvent ────────────── */
#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn handle_turn_finished_event(
    mut reader: EventReader<TurnFinishedEvent>,
    backend: Option<Res<BackendInfo>>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        use crate::snapshot::{set_game_state, SnapshotFromServer};

        if reader.read().next().is_some() {
            if let Some(b) = backend {
                let pid = b.partida_id;
                let uid = b.my_uid;

                spawn_local(async move {
                    if let Ok(resp) = Request::get(&format!("/api/snapshot/{}", pid)).send().await {
                        if let Ok(snapshot) = resp.json::<SnapshotFromServer>().await {
                            let json = serde_json::to_string(&snapshot).unwrap();
                            set_game_state(&json, uid); // ✅ ← uso correcto con referencia
                        }
                    }
                });
            }
        }
    }
}
