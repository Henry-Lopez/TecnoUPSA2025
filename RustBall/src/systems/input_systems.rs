use bevy::prelude::*;
use bevy::input::keyboard::KeyCode;
use bevy_rapier2d::prelude::*;

use crate::resources::*;
use crate::components::*;

pub fn aim_with_keyboard(
    keys: Res<Input<KeyCode>>,
    mut turn_state: ResMut<TurnState>,
) {
    let mut direction = turn_state.aim_direction;
    if keys.pressed(KeyCode::Left) {
        direction.x -= 0.1;
    }
    if keys.pressed(KeyCode::Right) {
        direction.x += 0.1;
    }
    if keys.pressed(KeyCode::Up) {
        direction.y += 0.1;
    }
    if keys.pressed(KeyCode::Down) {
        direction.y -= 0.1;
    }
    turn_state.aim_direction = direction.clamp_length_max(1.0);
}

pub fn charge_shot_power(
    keys: Res<Input<KeyCode>>,
    mut turn_state: ResMut<TurnState>,
) {
    if keys.pressed(KeyCode::Space) {
        turn_state.power = (turn_state.power + 0.02).min(1.0);
    }
}

use crate::powerup::{PendingDoubleBounce, PendingSpeedBoost, PendingDoubleTurn, PowerUpType};

// ---------------------------------------------------------------------------
// En tu módulo de disparo (fire_selected_disk)
// ---------------------------------------------------------------------------
pub fn fire_selected_disk(
    keys: Res<Input<KeyCode>>,
    mut turn_state: ResMut<TurnState>,
    mut velocities: Query<(Entity, &mut Velocity), With<TurnControlled>>,
    mut commands: Commands,
    boost_q: Query<(), With<PendingSpeedBoost>>,
    bounce_q: Query<(), With<PendingDoubleBounce>>,
    turn_q: Query<(), With<PendingDoubleTurn>>,
    mut colliders: Query<&mut Restitution>,
) {
    if !keys.just_released(KeyCode::Space) || turn_state.in_motion {
        return;
    }

    let dir = turn_state.aim_direction.normalize_or_zero();
    let mut base_speed = turn_state.power * 800.0;
    let mut any_fired = false;

    for (entity, mut vel) in &mut velocities {
        // ⚡ velocidad
        if boost_q.get(entity).is_ok() {
            base_speed *= 1.5;
            commands.entity(entity).remove::<PendingSpeedBoost>();
            commands.entity(entity).remove::<PowerUpType>();
        }
        // 🎾 rebote doble
        if bounce_q.get(entity).is_ok() {
            if let Ok(mut rest) = colliders.get_mut(entity) {
                rest.coefficient = 2.0;
            }
            commands.entity(entity).remove::<PendingDoubleBounce>();
            commands.entity(entity).remove::<PowerUpType>();
        }
        // 🔁 doble turno
        if turn_q.get(entity).is_ok() {
            turn_state.skip_turn_switch = true;
            commands.entity(entity).remove::<PendingDoubleTurn>();
            commands.entity(entity).remove::<PowerUpType>();
        }

        vel.linvel = dir * base_speed;
        commands.entity(entity).remove::<Sleeping>();
        any_fired = true;
    }

    if any_fired {
        turn_state.in_motion = true;
        turn_state.power = 0.0;
    }
}


