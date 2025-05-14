use bevy::prelude::*;
use bevy::gizmos::gizmos::Gizmos;



use crate::components::*;
use crate::resources::*;

pub fn draw_aim_direction_gizmo(
    mut gizmos: Gizmos,
    turn_state: Res<TurnState>,
    query: Query<&Transform, With<TurnControlled>>,
) {
    if let Some(entity) = turn_state.selected_entity {
        if let Ok(transform) = query.get(entity) {
            let start = transform.translation.truncate();
            let end = start + turn_state.aim_direction * 100.0;
            gizmos.line_2d(start, end, Color::YELLOW);
        }
    }
}

pub fn animate_selected_disk(
    time: Res<Time>,
    turn_state: Res<TurnState>,
    mut query: Query<&mut Sprite>,
) {
    if let Some(selected) = turn_state.selected_entity {
        if let Ok(mut sprite) = query.get_mut(selected) {
            let t = (time.elapsed_seconds() * 6.0).sin() * 0.5 + 0.5;
            let mut color = sprite.color;
            color.set_a(0.2 + 0.8 * t);
            sprite.color = color;
        }
    }
}


