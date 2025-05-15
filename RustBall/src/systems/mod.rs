pub mod goal_systems;
pub mod input_systems;
pub mod turn_systems;
pub mod ui_systems;
pub mod visual_systems;
pub mod reset_for_formation;
mod random_event_system; // ✅ módulo interno privado

// ✅ Reexportamos función para que esté disponible fuera del módulo
pub use random_event_system::trigger_random_event_system;

pub use goal_systems::{
    detect_goal,
    handle_goal,
    goal_banner_fadeout,
    setup_goal_timer,
    wait_and_change_state,
    despawn_game_entities,
};

pub use input_systems::*;
pub use turn_systems::*;
pub use ui_systems::*;
pub use visual_systems::*;
pub use reset_for_formation::*;
