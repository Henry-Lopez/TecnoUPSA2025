use bevy::prelude::*;
use serde::Serialize;
use serde_json::json;

use crate::events::LocalTurnFinishedEvent;
use crate::{
    components::{PlayerDisk, Ball},      // Pelota incluida
    resources::BackendInfo,
    snapshot::{NextTurn, MyTurn},
};

/* ───── Recurso global para la jugada pendiente ───── */
#[derive(Resource, Default)]
pub struct PendingTurn(pub Option<TurnPayload>);

/* ───── Payload que viaja al backend ───── */
#[derive(Serialize, Clone, Debug)]
pub struct TurnPayload {
    pub id_partida:  i32,
    pub numero_turno: i32,
    pub id_usuario:  i32,
    pub jugada:      serde_json::Value,
}

/* ──────────────────────────────────────────
   1. Armar payload al terminar el movimiento
   ────────────────────────────────────────── */
pub fn send_turn_to_backend(
    mut ev_end : EventReader<LocalTurnFinishedEvent>,
    backend    : Res<BackendInfo>,
    next_turn  : Option<Res<NextTurn>>,
    q_disks    : Query<(Entity, &Transform, &PlayerDisk)>,
    q_ball     : Query<&Transform, With<Ball>>,
    mut commands: Commands,
) {
    let Some(next_turn) = next_turn else { return };

    for _ in ev_end.read() {
        info!("📤 TurnFinished — UID {}", backend.my_uid);

        // 1-A) Obtener fichas con posiciones y dueño real
        let mut piezas: Vec<_> = q_disks
            .iter()
            .map(|(e, tf, disk)| json!({
                "id"             : e.index(),
                "id_usuario_real": disk.id_usuario_real,
                "x"              : tf.translation.x,
                "y"              : tf.translation.y
            }))
            .collect();

        // 1-B) Añadir pelota si existe en escena
        if let Ok(tf) = q_ball.get_single() {
            piezas.push(json!({
                "id"             : -1,      // id reservado para pelota
                "id_usuario_real": 0,
                "x"              : tf.translation.x,
                "y"              : tf.translation.y
            }));
        }

        if piezas.is_empty() {
            warn!("⚠️ No se encontraron piezas; no se enviará jugada.");
            return;
        }

        // 1-C) Construir payload JSON completo
        let payload = TurnPayload {
            id_partida   : backend.partida_id,
            numero_turno : next_turn.0,
            id_usuario   : backend.my_uid,
            jugada       : json!({ "piezas": piezas }),
        };

        info!("✅ Payload armado: {:?}", payload);

        // Encolar para envío inmediato
        commands.insert_resource(PendingTurn(Some(payload)));
        // Bloquear input local hasta recibir snapshot del backend
        commands.insert_resource(MyTurn(false));
    }
}

/* ──────────────────────────────────────────
   2. Enviar el payload encolado inmediatamente
   ────────────────────────────────────────── */
#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

pub fn maybe_send_pending_turn(mut pending: ResMut<PendingTurn>) {
    if let Some(payload) = pending.0.take() {
        info!("📬 Enviando jugada al backend…");

        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            let body = serde_json::to_string(&payload).unwrap();

            match Request::post("/api/jugada")
                .header("Content-Type", "application/json")
                .body(body)
                .unwrap()
                .send()
                .await
            {
                Ok(resp) if resp.status() < 300 => {
                    let txt = resp.text().await.unwrap_or_default();
                    info!("✅ POST /api/jugada OK: {txt}");
                }
                Ok(resp) => {
                    let txt = resp.text().await.unwrap_or_default();
                    error!("⚠️ POST /api/jugada {}: {txt}", resp.status());
                }
                Err(e) => error!("❌ Error de red /api/jugada: {e:?}"),
            }
        });
    }
}
