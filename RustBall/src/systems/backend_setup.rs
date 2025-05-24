use bevy::prelude::*;
use crate::resources::BackendInfo;

#[cfg(target_arch = "wasm32")]
use web_sys::window;

#[cfg(target_arch = "wasm32")]
use crate::INIT;

#[cfg(target_arch = "wasm32")]
pub fn insert_backend_info(mut commands: Commands) {
    let ls_val = |key: &str| -> String {
        window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item(key).ok().flatten())
            .unwrap_or_default()
    };

    let pid_str = ls_val("rb_pid");
    let uid_str = ls_val("rb_uid");
    let id_left_str = ls_val("rb_id_left");
    let id_right_str = ls_val("rb_id_right");

    let pid = pid_str.parse::<i32>().unwrap_or(0);
    let uid_me = uid_str.parse::<i32>().unwrap_or(0);
    let id_left = id_left_str.parse::<i32>().unwrap_or(0);
    let id_right = id_right_str.parse::<i32>().unwrap_or(0);

    if pid == 0 || uid_me == 0 || id_left == 0 || id_right == 0 {
        warn!(
            "⚠️ BackendInfo inválido. pid={}, uid={}, id_left={}, id_right={}",
            pid, uid_me, id_left, id_right
        );
        return;
    }

    let info = BackendInfo::new_with_snapshot(pid, uid_me, id_left, id_right, None); // ✅ con snapshot opcional
    commands.insert_resource(info.clone());
    info!("✅ BackendInfo registrado: {:?}", info);
}

#[cfg(target_arch = "wasm32")]
pub fn load_backend_info_if_available(world: &mut World) {
    use crate::INIT;

    web_sys::console::log_1(&"🔄 Intentando cargar BackendInfo desde INIT...".into());

    if let Some(lock) = INIT.get() {
        if let Some(data) = lock.lock().unwrap().take() {
            web_sys::console::log_1(&"✅ BackendInfo cargado e insertado en Bevy".into());
            world.insert_resource(data);
        } else {
            web_sys::console::warn_1(&"⚠️ INIT estaba vacío (sin datos)".into());
        }
    } else {
        web_sys::console::warn_1(&"⚠️ INIT no estaba inicializado".into());
    }
}

