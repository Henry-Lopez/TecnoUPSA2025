use bevy::prelude::*;
use crate::components::{PlayerDisk, FormationMenu, Ball, GoalZone};
use crate::resources::PlayerFormations;
use crate::formation_selection::show_formation_ui;

/// 🔁 Limpia jugadores, arcos, pelotas y formaciones para seleccionar nuevas tras un gol.
pub fn reset_for_formation(
    mut commands: Commands,
    disks: Query<Entity, With<PlayerDisk>>,
    menus: Query<Entity, With<FormationMenu>>,
    balls: Query<Entity, With<Ball>>,
    goals: Query<Entity, With<GoalZone>>,
    mut formations: ResMut<PlayerFormations>,
    asset_server: Res<AssetServer>,
) {
    // 🧹 Eliminar jugadores
    for entity in &disks {
        commands.entity(entity).despawn_recursive();
    }

    // 🧹 Eliminar menú de selección anterior
    for entity in &menus {
        commands.entity(entity).despawn_recursive();
    }

    // 🧹 Eliminar pelotas
    for entity in &balls {
        commands.entity(entity).despawn_recursive();
    }

    // 🧹 Eliminar arcos
    for entity in &goals {
        commands.entity(entity).despawn_recursive();
    }

    // 🔄 Resetear formaciones anteriores
    formations.player1 = None;
    formations.player2 = None;

    // 🎮 Volver a mostrar menú de selección de formaciones
    show_formation_ui(&mut commands, &asset_server);

}
