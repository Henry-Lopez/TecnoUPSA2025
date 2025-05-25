//! src/lib.rs
//  -------------------------------------------------------------------
//  Punto de arranque de la aplicación: carga módulos, crea la App y
//  registra todos los sistemas / recursos.
//  -------------------------------------------------------------------

// ──────────────── MÓDULOS DEL JUEGO ────────────────────────────────
pub mod components;
pub mod resources;
pub mod events;
pub mod systems;          // contenedor re-export
pub mod setup;
pub mod formation;
pub mod formation_selection;
pub mod game_over;
mod powerup;
pub mod zone;
mod snapshot;

// ──────────────── USE GENÉRICOS ────────────────────────────────────
use bevy::asset::AssetMetaCheck;
use once_cell::sync::OnceCell;
use powerup::*;

use crate::resources::WsInbox;

// 🔐 Caja estática para pasar mensajes JS → Bevy (sólo WASM)
#[cfg(target_arch = "wasm32")]
static WS_INBOX: OnceCell<Mutex<WsInbox>> = OnceCell::new();

// WASM bootstrap
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

// ─────────────── USE ESPECÍFICOS DEL JUEGO ─────────────────────────
use crate::components::{PlayerDisk, PowerUpLabel};
use crate::events::{FormationChosenEvent, TurnFinishedEvent};
use crate::zone::apply_zone_effects;

#[cfg(target_arch = "wasm32")]
use std::sync::Mutex;

#[cfg(target_arch = "wasm32")]
pub static INIT: once_cell::sync::OnceCell<Mutex<Option<crate::resources::BackendInfo>>> = once_cell::sync::OnceCell::new();

#[cfg(target_arch = "wasm32")]
pub static FORMACIONES: once_cell::sync::OnceCell<Mutex<Option<Vec<crate::snapshot::FormacionData>>>> = once_cell::sync::OnceCell::new();

// ──────────────────────────── WASM entry ───────────────────────────
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    main_internal();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    main_internal();
}

// Función JS para enviar texto por WebSocket
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = sendOverWS)]
    fn send_over_ws(msg: &str);
}

// Entrada JS → cola Bevy
#[wasm_bindgen]
pub fn receive_ws_message(msg: String) {
    web_sys::console::log_1(&format!("📥 Mensaje desde JS: {msg}").into());

    #[cfg(target_arch = "wasm32")]
    if let Some(lock) = WS_INBOX.get() {
        if let Ok(mut inbox) = lock.lock() {
            inbox.0.push(msg);
        }
    }
}

// 🎮 Juego real
pub fn main_internal() {
    use bevy::prelude::*;
    use bevy::audio::{PlaybackSettings, GlobalVolume};
    use bevy_rapier2d::prelude::*;
    use crate::resources::*;
    use crate::events::GoalEvent;
    use crate::setup::{setup, cleanup_cameras};
    use crate::systems::*;
    use crate::formation_selection::{handle_formation_click, cleanup_formation_ui};
    use crate::setup::ui::cleanup_power_bar;
    use crate::game_over::{show_game_over_screen, cleanup_game_over_ui};
    use crate::zone::{update_zone_lifetime, update_active_effect_text, hide_effect_text_if_none};

    // ... (resto del código como ya proporcionado) ...
}
