use bevy::prelude::*;
use web_sys::window;

use crate::resources::BackendInfo;

pub fn insert_backend_info(world: &mut World) {
    let Some(storage) = window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    else {
        error!("❌ No se pudo acceder a localStorage.");
        return;
    };

    let Some(partida_id) = storage.get_item("rb_pid").ok().flatten() else {
        error!("❌ Falta `rb_pid` en localStorage.");
        return;
    };
    let Some(id_left) = storage.get_item("rb_uid").ok().flatten() else {
        error!("❌ Falta `rb_uid` en localStorage.");
        return;
    };

    // Simula que el jugador contrario es id + 1 (temporal)
    let id_r = id_left.parse::<i32>().unwrap_or(0) + 1;

    let info = BackendInfo::new(
        partida_id.parse().unwrap_or(0),
        id_left.parse().unwrap_or(0),
        id_r,
    );

    info!("✅ BackendInfo registrado: {:?}", info);

    world.insert_resource(info);
}
