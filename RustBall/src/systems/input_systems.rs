use bevy::prelude::*;
use bevy::input::keyboard::KeyCode;
use bevy_rapier2d::prelude::*;

use crate::resources::*;
use crate::components::*;
use crate::powerup::{PendingDoubleBounce, PendingSpeedBoost, PendingDoubleTurn, PowerUpType};
use crate::snapshot::MyTurn;

/// 🎯 Control de dirección
pub fn aim_with_keyboard(
    keys: Res<Input<KeyCode>>,
    my_turn: Res<MyTurn>,
    backend_info: Res<BackendInfo>,
    controlled: Query<&OwnedBy, With<TurnControlled>>,
    mut turn_state: ResMut<TurnState>,
) {
    if !my_turn.0 {
        return;
    }

    // Asegurar que el disco controlado sea nuestro
    for owned_by in &controlled {
        if owned_by.0 != backend_info.my_uid {
            return;
        }
    }

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

/// 🔋 Cargar disparo
pub fn charge_shot_power(
    keys: Res<Input<KeyCode>>,
    my_turn: Res<MyTurn>,
    backend_info: Res<BackendInfo>,
    controlled: Query<&OwnedBy, With<TurnControlled>>,
    mut turn_state: ResMut<TurnState>,
) {
    if !my_turn.0 {
        return;
    }

    for owned_by in &controlled {
        if owned_by.0 != backend_info.my_uid {
            return;
        }
    }

    if keys.pressed(KeyCode::Space) {
        turn_state.power = (turn_state.power + 0.02).min(1.0);
    }
}

/// 🚀 Disparo principal
pub fn fire_selected_disk(
    keys: Res<Input<KeyCode>>,
    my_turn: Res<MyTurn>,
    backend_info: Res<BackendInfo>,
    mut turn_state: ResMut<TurnState>,
    mut velocities: Query<(Entity, &mut Velocity, &OwnedBy), With<TurnControlled>>,
    mut commands: Commands,
    boost_q: Query<(), With<PendingSpeedBoost>>,
    bounce_q: Query<(), With<PendingDoubleBounce>>,
    turn_q: Query<(), With<PendingDoubleTurn>>,
    mut colliders: Query<&mut Restitution>,
) {
    if !keys.just_released(KeyCode::Space) || turn_state.in_motion || !my_turn.0 {
        return;
    }

    let dir = turn_state.aim_direction.normalize_or_zero();
    let mut base_speed = turn_state.power * 800.0;
    let mut any_fired = false;

    for (entity, mut vel, owned_by) in &mut velocities {
        if owned_by.0 != backend_info.my_uid {
            continue;
        }

        // ⚡ PowerUp: velocidad
        if boost_q.get(entity).is_ok() {
            base_speed *= 1.5;
            commands.entity(entity).remove::<PendingSpeedBoost>();
            commands.entity(entity).remove::<PowerUpType>();
        }

        // 🎾 PowerUp: rebote doble
        if bounce_q.get(entity).is_ok() {
            if let Ok(mut rest) = colliders.get_mut(entity) {
                rest.coefficient = 2.0;
            }
            commands.entity(entity).remove::<PendingDoubleBounce>();
            commands.entity(entity).remove::<PowerUpType>();
        }

        // 🔁 PowerUp: doble turno
        if turn_q.get(entity).is_ok() {
            turn_state.skip_turn_switch = true;
            commands.entity(entity).remove::<PendingDoubleTurn>();
            commands.entity(entity).remove::<PowerUpType>();
        }

        // 🚀 Aplicar impulso
        vel.linvel = dir * base_speed;
        commands.entity(entity).remove::<Sleeping>();
        any_fired = true;
    }

    if any_fired {
        turn_state.in_motion = true;
        turn_state.power = 0.0;
    }
}
