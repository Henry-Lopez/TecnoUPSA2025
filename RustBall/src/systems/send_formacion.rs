use bevy::prelude::*;
use serde::Serialize;

use crate::events::FormationChosenEvent;
use crate::resources::BackendInfo;

#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
#[cfg(target_arch = "wasm32")]
use web_sys::console;

/* ——— payload que espera el backend ——— */
#[derive(Serialize)]
struct FormacionPayload {
    id_partida: i32,
    id_usuario: i32,
    formacion: String,
    turno_inicio: i32,
}

/* ——— sistema que envía la elección ——— */
pub fn send_formacion_to_backend(
    mut ev_form: EventReader<FormationChosenEvent>,
    backend: Res<BackendInfo>,
) {
    let my_uid = backend.my_uid;

    for ev in ev_form.read() {
        let payload = FormacionPayload {
            id_partida: backend.partida_id,
            id_usuario: my_uid,
            formacion: ev.formacion.clone(),
            turno_inicio: 0, // ⬅️ el servidor decide quién arranca
        };

        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            match Request::post("/api/formacion")
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&payload).unwrap())
                .unwrap()
                .send()
                .await
            {
                Ok(response) => {
                    let status = response.status();
                    if status >= 400 {
                        let text = response.text().await.unwrap_or_default();
                        console::log_1(&format!("❌ Error /api/formacion: status={} body={}", status, text).into());
                    } else {
                        console::log_1(&"✅ Formación enviada correctamente.".into());
                    }
                }
                Err(e) => {
                    console::log_1(&format!("❌ Fallo de red /api/formacion: {e:?}").into());
                }
            }
        });
    }
}
