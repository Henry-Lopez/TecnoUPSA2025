use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{
    components::PlayerDisk,
    formation::spawn_formation_for,
    systems::apply_board_snapshot,
    resources::{AppState, Scores, TurnState, UltimoTurnoAplicado},
    resources::BackendInfo,
};

/* ───────────── modelos JSON ───────────── */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PiezaPos {
    pub id: u32,             // 🎨 ID visual (1 = izq, 2 = der, 0 = otro)
    pub x: f32,
    pub y: f32,
    pub id_usuario_real: i32, // 👤 ID real del jugador
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
}

/* ───── Recurso “¿es mi turno?” ───── */
#[derive(Resource, Default)]
pub struct MyTurn(pub bool);

/* ───── Recurso para saber a quién le toca el turno ───── */
#[derive(Resource, Default)]
pub struct CurrentPlayerId(pub i32);

/* ───── Guarda temporalmente el snapshot que llega de JS ───── */
thread_local! {
    static APP_STATE: std::cell::RefCell<Option<(SnapshotFromServer, i32)>> =
        const { std::cell::RefCell::new(None) };
}

/* ───── función pública que llama JS ───── */
#[wasm_bindgen]
pub fn set_game_state(json: String, my_uid: i32) {
    let snap: SnapshotFromServer =
        serde_json::from_str(&json).expect("snapshot JSON malformado");
    APP_STATE.with(|c| *c.borrow_mut() = Some((snap, my_uid)));
}

/* ───── sistema que aplica el snapshot cuando exista ───── */
pub fn snapshot_apply_system(
    mut commands: Commands,
    mut scores: ResMut<Scores>,
    mut ts: ResMut<TurnState>,
    mut my_turn: ResMut<MyTurn>,
    mut ultimo_turno: ResMut<UltimoTurnoAplicado>,
    mut current_player_id: ResMut<CurrentPlayerId>,
    q_disks: Query<Entity, With<PlayerDisk>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
    backend_info: Res<BackendInfo>,
) {
    let Some((snap, my_uid)) = APP_STATE.with(|c| c.borrow_mut().take()) else {
        return;
    };

    if snap.proximo_turno == ultimo_turno.0 {
        return;
    }
    ultimo_turno.0 = snap.proximo_turno;

    for entity in &q_disks {
        commands.entity(entity).despawn_recursive();
    }

    if let Some(last) = snap.turnos.last() {
        if let Ok(board) = serde_json::from_value::<BoardSnapshot>(last.jugada.clone()) {
            let mapped = BoardSnapshot {
                piezas: board
                    .piezas
                    .into_iter()
                    .map(|pieza| {
                        let id_visual = if pieza.id == backend_info.id_left as u32 {
                            1
                        } else if pieza.id == backend_info.id_right as u32 {
                            2
                        } else {
                            0
                        };

                        PiezaPos {
                            id: id_visual,
                            x: pieza.x,
                            y: pieza.y,
                            id_usuario_real: pieza.id as i32,
                        }
                    })
                    .collect(),
            };
            apply_board_snapshot(mapped, &mut commands);
        }
    } else if snap.formaciones.len() >= 2 {
        for form in &snap.formaciones {
            spawn_formation_for(form, &mut commands, &asset_server, &backend_info);
        }
    }

    *scores = Scores {
        left: snap.marcador.0,
        right: snap.marcador.1,
    };

    ts.in_motion = false;
    ts.selected_entity = None;
    ts.skip_turn_switch = false;

    my_turn.0 = snap.proximo_turno == my_uid;
    current_player_id.0 = snap.proximo_turno;

    if *state != AppState::InGame && snap.proximo_turno != 0 {
        info!("🎮 El juego ha comenzado. Transicionando a AppState::InGame.");
        next_state.set(AppState::InGame);
    }
}

/* ───── WASM: sistema de polling para formaciones ───── */
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
    mut _latest: ResMut<LatestSnapshot>,
) {
    use gloo_net::http::Request;
    use wasm_bindgen_futures::spawn_local;

    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    if let Some(backend) = backend {
        let partida_id = backend.partida_id;
        let my_uid = backend.my_uid;

        spawn_local(async move {
            if let Ok(resp) = Request::get(&format!("/api/snapshot/{}", partida_id)).send().await {
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
