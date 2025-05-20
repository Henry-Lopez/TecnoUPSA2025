use bevy::prelude::*;
use crate::resources::BackendInfo;

#[cfg(target_arch = "wasm32")]
use web_sys::window;

pub fn insert_backend_info(mut commands: Commands) {
    #[cfg(target_arch = "wasm32")]
    {
        let ls_val = |key: &str| -> String {
            window()
                .and_then(|w| w.local_storage().ok().flatten())
                .and_then(|s| s.get_item(key).ok().flatten())
                .unwrap_or_default()
        };

        let pid      = ls_val("rb_pid");
        let uid_me   = ls_val("rb_uid");
        let id_left  = ls_val("rb_id_left");
        let id_right = ls_val("rb_id_right");

        let info = BackendInfo::new(
            pid      .parse().unwrap_or(0),
            uid_me   .parse().unwrap_or(0),
            id_left  .parse().unwrap_or(0),
            id_right .parse().unwrap_or(0),
        );

        commands.insert_resource(info.clone()); // ✅ usa clone aquí
        info!("✅ BackendInfo registrado: {:?}", info);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let info = BackendInfo::new(1, 1, 1, 2);
        commands.insert_resource(info.clone()); // ✅ usa clone aquí también
        info!("⚠️ BackendInfo dummy (native build): {:?}", info);
    }
}
