use bevy::prelude::*;
use crate::resources::{WsInbox, AppState, BackendInfo};
use crate::snapshot::SnapshotFromServer;

#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
#[cfg(target_arch = "wasm32")]
use crate::snapshot::set_game_state;

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn process_ws_messages(
    mut inbox: ResMut<WsInbox>,
    backend: Option<Res<BackendInfo>>, // solo se usa en WASM
    mut next_state: ResMut<NextState<AppState>>,
) {
    for msg in inbox.0.drain(..) {
        match msg.as_str() {
            "turno_finalizado" => {
                info!("🟢 WebSocket: turno_finalizado → Transición a AppState::FormationChange");
                next_state.set(AppState::FormationChange);

                // Solo para WASM: hace fetch del nuevo snapshot y lo guarda para que lo lea Bevy
                #[cfg(target_arch = "wasm32")]
                if let Some(ref backend_info) = backend {
                    let pid = backend_info.partida_id;
                    let uid = backend_info.my_uid;

                    spawn_local(async move {
                        if let Ok(resp) = Request::get(&format!("/api/snapshot/{}", pid)).send().await {
                            if let Ok(snapshot) = resp.json::<SnapshotFromServer>().await {
                                let json = serde_json::to_string(&snapshot).unwrap();
                                set_game_state(json, uid); // ← Esto activa `snapshot_apply_system`
                            }
                        }
                    });
                }
            }

            other => {
                warn!("❓ WebSocket: mensaje no reconocido: {}", other);
            }
        }
    }
}
