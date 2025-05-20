//! Punto único de entrada para todos los *systems* del juego.
//!
//! • Los módulos **públicos** (`pub mod …`) exponen su API completa al resto
//!   del crate (agrupan varios systems y helpers).
//! • Los módulos **privados** (`mod …`) sólo exponen explícitamente las
//!   funciones que re-exportamos más abajo; el resto queda encapsulado.

// ────────────────────────── MÓDULOS PÚBLICOS ────────────────────────────
pub mod goal_systems;
pub mod input_systems;
pub mod turn_systems;
pub mod ui_systems;
pub mod visual_systems;
pub mod reset_for_formation;
pub mod poll_turn;          // ⟵ el polling es público

// ────────────────────────── MÓDULOS PRIVADOS ────────────────────────────
mod random_event_system;      // eventos aleatorios
mod backend_setup;            // lee localStorage → BackendInfo
mod send_goal;                // POST /api/gol
mod send_formacion;           // POST /api/formacion
mod send_turn;                // POST /api/jugada
mod apply_snapshot;           // aplica la foto del tablero
mod process_ws;               // ⬅️ procesa mensajes recibidos por WebSocket

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

// — Polling (sólo un sistema público) ───────────────────────────────────
pub use poll_turn::{
    poll_turn_tick_system,
    handle_turn_finished_event, // ✅ nuevo sistema para aplicar snapshot al recibir evento
};

// — WebSocket (mensajes entrantes) ──────────────────────────────────────
pub use process_ws::process_ws_messages;

// — Goles ────────────────────────────────────────────────────────────────
pub use goal_systems::{
    detect_goal,
    handle_goal,
    goal_banner_fadeout,
    setup_goal_timer,
    wait_and_change_state,
    despawn_game_entities,
};

// — Resto de systems “genéricos” ────────────────────────────────────────
pub use input_systems::*;
pub use turn_systems::*;
pub use ui_systems::*;
pub use visual_systems::*;
pub use reset_for_formation::*;
