pub mod camera;
pub mod ui;
pub mod field;
pub mod players;
pub mod ball;
pub mod goals;

use bevy::prelude::*;
use crate::resources::PlayerFormations;
use crate::components::PlayerDisk;

pub use camera::{spawn_camera_and_background, cleanup_cameras};
use ui::spawn_ui;
use field::spawn_walls;
use players::spawn_players_from_selection;
pub use ball::spawn_ball;
pub use goals::spawn_goals;

/// Configura todo el entorno del juego: cámara, fondo, UI, jugadores, balón, y goles.
pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_formations: Res<PlayerFormations>,
    player_query: Query<Entity, With<PlayerDisk>>,
    camera_query: Query<Entity, With<Camera>>,
) {
    cleanup_cameras(&mut commands, camera_query);

    spawn_camera_and_background(&mut commands, &asset_server);
    spawn_ui(&mut commands, &asset_server);
    spawn_walls(&mut commands);
    spawn_players_from_selection(&mut commands, &asset_server, player_formations, player_query);
    spawn_ball(&mut commands, &asset_server);
    spawn_goals(&mut commands, &asset_server);
}
