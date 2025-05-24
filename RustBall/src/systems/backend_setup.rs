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

        let pid_str      = ls_val("rb_pid");
        let uid_str      = ls_val("rb_uid");
        let id_left_str  = ls_val("rb_id_left");
        let id_right_str = ls_val("rb_id_right");

        let pid      = pid_str.parse::<i32>().unwrap_or(0);
        let uid_me   = uid_str.parse::<i32>().unwrap_or(0);
        let id_left  = id_left_str.parse::<i32>().unwrap_or(0);
        let id_right = id_right_str.parse::<i32>().unwrap_or(0);

        if pid == 0 || uid_me == 0 || id_left == 0 || id_right == 0 {
            warn!(
                "⚠️ BackendInfo inválido. pid={}, uid={}, id_left={}, id_right={}",
                pid, uid_me, id_left, id_right
            );
            return;
        }

        let info = BackendInfo::new(pid, uid_me, id_left, id_right);
        commands.insert_resource(info.clone());
        info!("✅ BackendInfo registrado: {:?}", info);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let info = BackendInfo::new(1, 1, 1, 2);
        commands.insert_resource(info.clone());
        info!("⚠️ BackendInfo dummy (native build): {:?}", info);
    }
}
