// src/systems/send_goal.rs

use bevy::prelude::*;
use serde::Serialize;

use crate::events::GoalEvent;
use crate::resources::BackendInfo;

#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

#[cfg(not(target_arch = "wasm32"))]
use tokio::task;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::Client;

#[derive(Serialize)]
struct GolPayload {
    id_partida:  i32,
    id_goleador: i32,
}

/// Escucha `GoalEvent` y notifica el gol al backend.
pub fn send_goal_to_backend(
    mut ev_goal: EventReader<GoalEvent>,
    backend:     Res<BackendInfo>,
) {
    for ev in ev_goal.read() {
        let id_goleador = if ev.scored_by_left {
            backend.id_left
        } else {
            backend.id_right
        };

        let payload = GolPayload {
            id_partida:  backend.partida_id,
            id_goleador,
        };

        // ——— WebAssembly: gloo-net + wasm-bindgen-futures ———
        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            // Construimos la Request y la desempaquetamos
            let req = Request::post("/api/gol")
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&payload).unwrap())
                .unwrap();

            // Enviamos
            let _ = req.send().await;
        });

        // ——— Nativo (desktop): reqwest + tokio ———
        #[cfg(not(target_arch = "wasm32"))]
        {
            task::spawn(async move {
                let _ = Client::new()
                    .post("http://127.0.0.1:3000/api/gol")
                    .json(&payload)
                    .send()
                    .await;
            });
        }
    }
}
