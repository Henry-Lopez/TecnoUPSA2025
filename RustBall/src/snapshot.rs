use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use crate::resources::BackendInfo;
use crate::{
    components::{PlayerDisk, Ball},   // ← corchetes y dos nombres
    formation::spawn_formation_for,
    resources::{
        AppState, CurrentPlayerId, PlayerNames, Scores, TurnState,
        UltimoTurnoAplicado, WsInbox,
    },
    systems::apply_board_snapshot,
};

/* ───────────── etiqueta SystemSet ───────────── */
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ApplySnapshotSet;

/* ───────────── Recurso: próximo nº de turno ───────────── */
#[derive(Resource, Default, Debug)]
pub struct NextTurn(pub i32);

/* ───────────── Recurso: ¿es mi turno? ───────────── */
#[derive(Resource, Default, Debug)]
pub struct MyTurn(pub bool);

/* ───────────── Modelos JSON que llegan del backend ───────────── */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PiezaPos {
    pub id: i32,
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
    pub estado: String,
    pub marcador: (u32, u32),
    pub formaciones: Vec<FormacionData>,
    pub turnos: Vec<TurnoData>,
    pub proximo_turno: i32,
    pub nombre_jugador_1: String,
    pub nombre_jugador_2: String,
}

/* ───────────── Buffer local (snapshot en cola) ───────────── */
thread_local! {
    static APP_STATE: std::cell::RefCell<Option<(SnapshotFromServer, i32)>> =
        const { std::cell::RefCell::new(None) };
}
static LAST_TURNO: std::sync::Mutex<i32> = std::sync::Mutex::new(0);

/* ───────────── Callback JS → Rust ───────────── */
#[wasm_bindgen]
pub fn set_game_state(json_str: &str, uid: i32) {
    web_sys::console::log_1(&"🧠 set_game_state() fue llamado".into());

    match serde_json::from_str::<SnapshotFromServer>(json_str) {
        Ok(snap) => {
            web_sys::console::log_1(&"✅ SnapshotFromServer parseado con éxito".into());

            if snap.estado != "playing" || snap.proximo_turno == 0 {
                warn!("⏳ Partida aún no está en estado 'playing' o turno inválido. Ignorando snapshot.");
                return;
            }

            let last = LAST_TURNO.lock().unwrap();

            info!(
                "📥 Recibido snapshot turno {} (último aplicado {})",
                snap.proximo_turno, *last
            );

            if snap.proximo_turno > *last {
                APP_STATE.with(|c| *c.borrow_mut() = Some((snap, uid)));
                info!("✅ Snapshot en cola para ser aplicado");
            } else {
                warn!("📛 Snapshot descartado (antiguo)");
            }
        }
        Err(e) => {
            web_sys::console::error_1(&format!("❌ Error al parsear snapshot JSON: {:?}", e).into());
        }
    }
}


#[allow(clippy::too_many_arguments)]
pub fn snapshot_apply_system(
    mut commands         : Commands,
    mut scores           : ResMut<Scores>,
    mut ts               : ResMut<TurnState>,
    mut ultimo_turno     : ResMut<UltimoTurnoAplicado>,
    mut current_player_id: ResMut<CurrentPlayerId>,
    q_ball              : Query<Entity, With<Ball>>,
    q_disks              : Query<Entity, With<PlayerDisk>>,
    state                : Res<State<AppState>>,
    mut next_state       : ResMut<NextState<AppState>>,
    asset_server         : Res<AssetServer>,
    backend_info         : Res<BackendInfo>,
    player_names         : Option<Res<PlayerNames>>,
) {
    let Some((snap, my_uid)) = APP_STATE.with(|c| c.borrow_mut().take()) else {
        warn!("📭 No hay snapshot en cola para aplicar.");
        return;
    };

    info!("🔄 Aplicando snapshot – turno {}", snap.proximo_turno);

    commands.insert_resource(PlayerNames {
        left_name : snap.nombre_jugador_1.clone(),
        right_name: snap.nombre_jugador_2.clone(),
    });

    if snap.proximo_turno == ultimo_turno.0 {
        warn!("⏩ Snapshot ignorado: mismo turno que el último aplicado ({})", snap.proximo_turno);
        return;
    }

    ultimo_turno.0 = snap.proximo_turno;

    if let Some(last) = snap.turnos.last() {
        if let Ok(board_raw) = serde_json::from_value::<BoardSnapshot>(last.jugada.clone()) {
            let mapped = BoardSnapshot {
                piezas: board_raw.piezas.into_iter().map(|p| PiezaPos {
                    id: p.id,
                    x: p.x,
                    y: p.y,
                    id_usuario_real: p.id_usuario_real,
                }).collect(),
            };

            apply_board_snapshot(
                mapped,
                &mut commands,
                backend_info.clone(),
                q_disks,          // fichas
                q_ball,           // ← nuevo
                snap.proximo_turno,
                player_names.map(|r| (*r).clone()),
                &asset_server,
            );


            commands.insert_resource(NextTurn(last.numero_turno + 1));
        } else {
            warn!("📛 Snapshot recibido con jugada inválida o corrupta.");
        }
    } else if snap.formaciones.len() >= 2 {
        for f in &snap.formaciones {
            spawn_formation_for(f, &mut commands, &asset_server, &backend_info);
        }
        commands.insert_resource(NextTurn(1));
    } else {
        warn!("📛 Snapshot recibido sin jugadas ni formaciones válidas.");
    }

    *scores = Scores { left: snap.marcador.0, right: snap.marcador.1 };

    ts.in_motion        = false;
    ts.selected_entity  = None;
    ts.skip_turn_switch = false;
    ts.current_turn_id  = snap.proximo_turno;
    current_player_id.0 = snap.proximo_turno;

    let is_my_turn = snap.proximo_turno == my_uid;
    commands.insert_resource(MyTurn(is_my_turn));
    info!("🕑 MyTurn = {}", is_my_turn);

    if *state != AppState::InGame && snap.proximo_turno != 0 {
        next_state.set(AppState::InGame);
        info!("🎮 Estado cambiado a InGame.");
    }

    // 🔄 ACTUALIZAR LAST_TURNO DESPUÉS DE APLICAR CORRECTAMENTE EL SNAPSHOT
    let mut last = LAST_TURNO.lock().unwrap();
    *last = snap.proximo_turno;
    info!("✅ LAST_TURNO actualizado a {}", *last);
}

/* ======================================================================= */
#[cfg(target_arch = "wasm32")]
pub fn fetch_snapshot_on_ws_message(mut inbox : ResMut<WsInbox>) {
    inbox.0.clear();
}

#[cfg(target_arch = "wasm32")]
pub fn poll_snapshot_when_forming(
    time  : Res<Time>,
    mut timer: ResMut<crate::resources::SnapshotPollTimer>,
    backend: Option<Res<BackendInfo>>,
) {
    use gloo_net::http::Request;
    use wasm_bindgen_futures::spawn_local;

    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    if let Some(b) = backend {
        let pid = b.partida_id;
        let uid = b.my_uid;

        spawn_local(async move {
            if let Ok(resp) = Request::get(&format!("/api/snapshot/{pid}")).send().await {
                if let Ok(snap) = resp.json::<SnapshotFromServer>().await {
                    if snap.proximo_turno != 0 {
                        crate::snapshot::set_game_state(&serde_json::to_string(&snap).unwrap(), uid);
                    }
                }
            }
        });
    }
}
