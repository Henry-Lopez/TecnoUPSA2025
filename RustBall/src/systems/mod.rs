pub mod goal_systems;
pub mod input_systems;
pub mod turn_systems;
pub mod ui_systems;
pub mod visual_systems;
pub mod reset_for_formation; // ✅ lo registramos

// Exportamos todos los sistemas relevantes desde goal_systems
pub use goal_systems::{
    detect_goal,
    handle_goal,
    goal_banner_fadeout,
    setup_goal_timer,
    wait_and_change_state,
    despawn_game_entities, // ✅ nuevo sistema agregado
};

pub use input_systems::*;
pub use turn_systems::*;
pub use ui_systems::*;
pub use visual_systems::*;
pub use reset_for_formation::*;
