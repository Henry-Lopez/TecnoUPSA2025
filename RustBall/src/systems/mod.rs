// ─────────────────── Sub-módulos públicos ───────────────────
pub mod goal_systems;
pub mod input_systems;
pub mod turn_systems;
pub mod ui_systems;
pub mod visual_systems;
pub mod reset_for_formation;

// ─────────────────── Sub-módulos internos ───────────────────
//  Estos quedan privados; solo re-exportamos las funciones que interesan.
mod random_event_system;
mod send_goal;
mod backend_setup;          // <-- nuevo módulo con Startup system

// ─────────────────── Re-exportes ─────────────────────────────
//   →  Así tu crate principal importa todo desde `systems::*`

// 1) Evento aleatorio
pub use random_event_system::trigger_random_event_system;

// 2) Sistema que envía /api/gol
pub use send_goal::send_goal_to_backend;

// 3) Startup-system que lee localStorage y crea BackendInfo
pub use backend_setup::insert_backend_info;

// 4) Utilidades del módulo de goles
pub use goal_systems::{
    detect_goal,
    handle_goal,
    goal_banner_fadeout,
    setup_goal_timer,
    wait_and_change_state,
    despawn_game_entities,
};

// 5) Todo lo demás que ya tenías
pub use input_systems::*;
pub use turn_systems::*;
pub use ui_systems::*;
pub use visual_systems::*;
pub use reset_for_formation::*;
