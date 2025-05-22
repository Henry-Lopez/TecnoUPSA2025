//! --------------------------------------------------------------
//! Manejo de snapshots recibidos desde el backend / WebSocket.
//!
//! • Aplica el tablero, marcador y jugador en turno.
//! • Mantiene el recurso `NextTurn`, que indica el número de turno
//!   (1-N) que el frontend debe enviar la próxima vez.
//! --------------------------------------------------------------

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{
    components::PlayerDisk,
    formation::spawn_formation_for,
    systems::apply_board_snapshot,
    resources::{
        AppState, Scores, TurnState, UltimoTurnoAplicado, BackendInfo,
        CurrentPlayerId, PlayerNames,
    },
};

/* ───────────── Recurso NUEVO ───────────── */
/// Próximo número de turno 1-N que debe enviarse a `/api/jugada`
#[derive(Resource, Default)]
pub struct NextTurn(pub i32);

/* ───────────── modelos JSON ───────────── */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PiezaPos {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    #[serde(default)]
    pub id_usuario_real: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BoardSnapshot {
    pub piezas: Vec<PiezaPos>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FormacionData {
    pub id_usuario: i32,
    pub formacion: String,
    pub turno_inicio: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TurnoData {
    pub numero_turno: i32,
    pub id_usuario: i32,
    pub jugada: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnapshotFromServer {
    pub marcador: (u32, u32),
    pub formaciones: Vec<FormacionData>,
    pub turnos: Vec<TurnoData>,
    pub proximo_turno: i32,
    pub nombre_jugador_1: String,
    pub nombre_jugador_2: String,
}

/* ───────────── Turno propio ───────────── */
#[derive(Resource, Default)]
pub struct MyTurn(pub bool);

/* ───── Guarda temporalmente el snapshot que llega de JS ───── */
thread_local! {
    static APP_STATE: std::cell::RefCell<Option<(SnapshotFromServer, i32)>> =
        const { std::cell::RefCell::new(None) };
}

static LAST_TURNO: std::sync::Mutex<i32> = std::sync::Mutex::new(0);

#[wasm_bindgen]
pub fn set_game_state(json: String, my_uid: i32) {
    let snap: SnapshotFromServer =
        serde_json::from_str(&json).expect("snapshot JSON malformado");

    let mut last = LAST_TURNO.lock().unwrap();

    info!(
        "📥 Recibido snapshot con turno: {}, actual: {}",
        snap.proximo_turno, *last
    );

    if snap.proximo_turno > *last {
        *last = snap.proximo_turno;
        APP_STATE.with(|c| *c.borrow_mut() = Some((snap, my_uid)));
        info!("✅ Snapshot guardado en APP_STATE.");
    } else {
        warn!(
            "📛 Snapshot descartado por ser viejo. Turno recibido: {}",
            snap.proximo_turno
        );
    }
}

/* ───────────── Sistema principal ───────────── */
pub fn snapshot_apply_system(
    mut commands: Commands,
    mut scores: ResMut<Scores>,
    mut ts: ResMut<TurnState>,
    mut ultimo_turno: ResMut<UltimoTurnoAplicado>,
    mut current_player_id: ResMut<CurrentPlayerId>,
    q_disks: Query<Entity, With<PlayerDisk>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
    backend_info: Res<BackendInfo>,
    player_names: Option<Res<PlayerNames>>,
) {
    /* — 0. ¿Tenemos snapshot nuevo? — */
    let Some((snap, my_uid)) = APP_STATE.with(|c| c.borrow_mut().take()) else {
        info!("⏳ Sin snapshot nuevo, esperando…");
        return;
    };

    /* — 1. Actualizar nombres (para overlays, etc.) — */
    commands.insert_resource(PlayerNames {
        left_name:  snap.nombre_jugador_1.clone(),
        right_name: snap.nombre_jugador_2.clone(),
    });

    /* — 2. Evitar aplicar el mismo turno dos veces — */
    if snap.proximo_turno == ultimo_turno.0 {
        return;
    }
    ultimo_turno.0 = snap.proximo_turno;

    /* — 3. Tablero / formaciones — */
    if let Some(last) = snap.turnos.last() {
        match serde_json::from_value::<BoardSnapshot>(last.jugada.clone()) {
            Ok(board) => {
                let mapped = BoardSnapshot {
                    piezas: board
                        .piezas
                        .into_iter()
                        .map(|pieza| PiezaPos {
                            id: if pieza.id == backend_info.id_left as u32 {
                                1
                            } else if pieza.id == backend_info.id_right as u32 {
                                2
                            } else {
                                0
                            },
                            x: pieza.x,
                            y: pieza.y,
                            id_usuario_real: pieza.id as i32,
                        })
                        .collect(),
                };

                apply_board_snapshot(
                    mapped,
                    &mut commands,
                    backend_info.clone(),
                    q_disks,
                    snap.proximo_turno,
                    player_names.map(|r| (*r).clone()),
                    &asset_server,
                );

                /* ►►►  ACTUALIZAR NextTurn  ◄◄◄ */
                commands.insert_resource(NextTurn(last.numero_turno + 1));
            }
            Err(e) => warn!("⚠️ Falló la deserialización del snapshot: {e:?}"),
        }
    } else if snap.formaciones.len() >= 2 {
        /* snapshot sin jugadas previas ⇒ ambos eligieron formación      */
        for form in &snap.formaciones {
            spawn_formation_for(form, &mut commands, &asset_server, &backend_info);
        }
        commands.insert_resource(NextTurn(1)); // primer turno de la partida
    }

    /* — 4. Marcador y estado de turno — */
    *scores = Scores {
        left:  snap.marcador.0,
        right: snap.marcador.1,
    };

    ts.in_motion        = false;
    ts.selected_entity  = None;
    ts.skip_turn_switch = false;

    let is_my_turn = snap.proximo_turno == my_uid;
    commands.insert_resource(MyTurn(is_my_turn));

    ts.current_turn_id  = snap.proximo_turno;
    current_player_id.0 = snap.proximo_turno;

    /* — 5. Cambiar a estado InGame si aún no lo estamos — */
    if *state != AppState::InGame && snap.proximo_turno != 0 {
        next_state.set(AppState::InGame);
    }
}

/* ───── WASM: polling de snapshot durante la pantalla de formaciones ───── */
#[cfg(target_arch = "wasm32")]
#[derive(Resource)]
pub struct SnapshotPollTimer(pub Timer);

#[cfg(target_arch = "wasm32")]
impl Default for SnapshotPollTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Resource, Default)]
pub struct LatestSnapshot;

#[cfg(target_arch = "wasm32")]
pub fn poll_snapshot_when_forming(
    time: Res<Time>,
    mut timer: ResMut<SnapshotPollTimer>,
    backend: Option<Res<BackendInfo>>,
) {
    use gloo_net::http::Request;
    use wasm_bindgen_futures::spawn_local;

    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    if let Some(backend) = backend {
        let partida_id = backend.partida_id;
        let my_uid     = backend.my_uid;

        spawn_local(async move {
            if let Ok(resp) = Request::get(&format!("/api/snapshot/{partida_id}")).send().await {
                if let Ok(snapshot) = resp.json::<SnapshotFromServer>().await {
                    if snapshot.proximo_turno != 0 {
                        crate::snapshot::set_game_state(
                            serde_json::to_string(&snapshot).unwrap(),
                            my_uid,
                        );
                    }
                }
            }
        });
    }
}
