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

use crate::powerup::{PendingDoubleBounce, PendingSpeedBoost, PendingDoubleTurn};

pub fn fire_selected_disk(
    keys: Res<Input<KeyCode>>,
    mut turn_state: ResMut<TurnState>,
    mut velocities: Query<(Entity, &mut Velocity), With<TurnControlled>>,
    mut commands: Commands,
    boost_query: Query<(), With<PendingSpeedBoost>>,
    bounce_query: Query<(), With<PendingDoubleBounce>>,
    turn_query: Query<(), With<PendingDoubleTurn>>,
    mut colliders: Query<&mut Restitution>,
) {
    if keys.just_released(KeyCode::Space) && !turn_state.in_motion {
        let direction = turn_state.aim_direction.normalize_or_zero();
        let mut speed = turn_state.power * 800.0;

        let mut applied = false;

        for (entity, mut velocity) in &mut velocities {
            // Velocidad extra
            if boost_query.get(entity).is_ok() {
                speed *= 1.5;
                commands.entity(entity).remove::<PendingSpeedBoost>();
                println!("⚡ Power-Up de velocidad aplicado a {:?}", entity);
            }

            // Rebote doble
            if bounce_query.get(entity).is_ok() {
                if let Ok(mut restitution) = colliders.get_mut(entity) {
                    restitution.coefficient = 2.0;
                    println!("🎾 Power-Up de REBOTE DOBLE aplicado a {:?}", entity);
                }
                commands.entity(entity).remove::<PendingDoubleBounce>();
            }

            // Doble turno
            if turn_query.get(entity).is_ok() {
                turn_state.skip_turn_switch = true;
                commands.entity(entity).remove::<PendingDoubleTurn>();
                println!("🔁 Power-Up de DOBLE TURNO aplicado a {:?}", entity);
            }

            let force = direction * speed;
            velocity.linvel = force;
            commands.entity(entity).remove::<Sleeping>();
            println!("→ Velocidad aplicada: {:?} a {:?}", force, entity);
            applied = true;
        }

        if applied {
            turn_state.in_motion = true;
            turn_state.power = 0.0;
        } else {
            println!("⚠️ No se aplicó velocidad: no hay entidad con TurnControlled");
        }
    }
}

