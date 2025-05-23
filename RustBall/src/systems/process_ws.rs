use bevy::prelude::*;
use crate::resources::{WsInbox, AppState, BackendInfo};

/* —––––––––– SECCIÓN WASM (web_sys) —––––––––––––––––––––––––––––––––– */
#[cfg(target_arch = "wasm32")]
mod wasm_ws {
    use super::*;
    use crate::snapshot::{set_game_state, SnapshotFromServer};
    use gloo_net::http::Request;
    use wasm_bindgen::{closure::Closure, JsCast};
    use wasm_bindgen_futures::spawn_local;
    use web_sys::{MessageEvent, WebSocket};

    thread_local! {
        static WS_CONN : std::cell::RefCell<Option<WebSocket>> = const { std::cell::RefCell::new(None) };
    }

    pub fn ensure_ws_connected(backend: &BackendInfo) {
        WS_CONN.with(|cell| {
            if cell.borrow().is_some() {
                return;
            }

            let loc = web_sys::window().unwrap().location();
            let host = loc.host().unwrap(); // ej: localhost:10000
            let proto = if loc.protocol().unwrap() == "https:" { "wss" } else { "ws" };
            let url = format!("{proto}://{host}/ws/{}/{}", backend.partida_id, backend.my_uid);

            let ws = WebSocket::new(&url).expect("No se pudo abrir WebSocket");
            ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

            let pid = backend.partida_id;
            let uid = backend.my_uid;

            let on_msg = Closure::<dyn FnMut(MessageEvent)>::wrap(Box::new(move |ev: MessageEvent| {
                if let Ok(txt) = ev.data().dyn_into::<js_sys::JsString>() {
                    let msg = txt.as_string().unwrap_or_default();

                    // 1. Empujar a la bandeja
                    if let Some(lock) = crate::WS_INBOX.get() {
                        if let Ok(mut inbox) = lock.lock() {
                            inbox.0.push(msg.clone());
                        }
                    }

                    // 2. Si es “start” o “turno_finalizado” ⇒ fetch + aplicar snapshot
                    if msg == "start" || msg == "turno_finalizado" {
                        web_sys::console::log_1(&format!("⚡ WSS: {msg}").into());

                        spawn_local(async move {
                            if let Ok(resp) = Request::get(&format!("/api/snapshot/{pid}")).send().await {
                                if let Ok(snap) = resp.json::<SnapshotFromServer>().await {
                                    if let Ok(json) = serde_json::to_string(&snap) {
                                        set_game_state(&json, uid); // ✅ ← usar referencia &json
                                    }
                                }
                            }
                        });
                    }
                }
            }) as Box<dyn FnMut(_)>);

            ws.set_onmessage(Some(on_msg.as_ref().unchecked_ref()));
            on_msg.forget();

            cell.replace(Some(ws));
            web_sys::console::log_1(&format!("🔗 WS conectado a {url}").into());
        });
    }
}

/* —––––––––– SISTEMA BEVY —––––––––––––––––––––––––––––––––––––––––––– */

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn process_ws_messages(
    mut inbox: ResMut<WsInbox>,
    backend: Option<Res<BackendInfo>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // 1. En WASM: garantizar que el WebSocket esté conectado
    #[cfg(target_arch = "wasm32")]
    if let Some(ref be) = backend {
        wasm_ws::ensure_ws_connected(be);
    }

    // 2. Procesar mensajes empujados desde JS → Bandeja
    for msg in inbox.0.drain(..) {
        match msg.as_str() {
            "turno_finalizado" => {
                info!("🟢 WsInbox: turno_finalizado → AppState::FormationChange");
                next_state.set(AppState::FormationChange);
            }
            "start" => {
                info!("🟢 WsInbox: start recibido → posiblemente cambiar estado");
            }
            other => warn!("❓ WsInbox: mensaje no reconocido: {other}"),
        }
    }
}
