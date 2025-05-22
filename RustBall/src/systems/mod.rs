//! src/systems/mod.rs
//! Punto único de entrada para todos los *systems* del juego.

// ────────────────────────── MÓDULOS PÚBLICOS ────────────────────────────
pub mod goal_systems;
pub mod input_systems;      // ← sigue existiendo, pero ya sin exportar lógica de turno
pub mod turn_systems;
pub mod ui_systems;
pub mod visual_systems;
pub mod reset_for_formation;
pub mod poll_turn;          // ⟵ el polling es público

// ────────────────────────── MÓDULOS PRIVADOS ────────────────────────────
mod random_event_system;
mod backend_setup;
mod send_goal;
mod send_formacion;
mod send_turn;
mod apply_snapshot;
mod process_ws;

// ────────────────────────── RE-EXPORTES ÚTILES ──────────────────────────
// Basta con:   use systems::*;

pub use random_event_system::trigger_random_event_system;
pub use backend_setup::insert_backend_info;

// — Envíos al backend ───────────────────────────────────────────────────
pub use send_goal::send_goal_to_backend;
pub use send_formacion::send_formacion_to_backend;
pub use send_turn::send_turn_to_backend;

// — Snapshot al tablero ─────────────────────────────────────────────────
pub use apply_snapshot::apply_board_snapshot;

// — Polling ─────────────────────────────────────────────────────────────
pub use poll_turn::{poll_turn_tick_system, handle_turn_finished_event};

// — WebSocket (entrantes) ───────────────────────────────────────────────
pub use process_ws::process_ws_messages;

// — Goles ───────────────────────────────────────────────────────────────
pub use goal_systems::{
    detect_goal,
    handle_goal,
    goal_banner_fadeout,
    setup_goal_timer,
    wait_and_change_state,
    despawn_game_entities,
};

// — Systems de TURNO (incluyen aim/charge/fire actualizados) ────────────
pub use turn_systems::*;

// — HUD / UI / Visual —──────────────────────────────────────────────────
pub use ui_systems::*;
pub use visual_systems::*;
pub use reset_for_formation::*;
