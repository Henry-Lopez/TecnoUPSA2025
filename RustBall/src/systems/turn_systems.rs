use bevy::prelude::*;
use bevy_rapier2d::prelude::{RigidBody, Velocity};
use bevy::input::keyboard::KeyCode;

use crate::components::*;
use crate::powerup::PowerUpControl;
use crate::resources::*;

pub fn auto_select_first_disk(
    mut turn_state: ResMut<TurnState>,
    disks: Query<(Entity, &PlayerDisk), Without<TurnControlled>>,
    mut commands: Commands,
    mut sprites: Query<&mut Sprite>,
) {
    if turn_state.selected_entity.is_none() && !turn_state.in_motion {
        for (entity, disk) in &disks {
            if disk.player_id == turn_state.current_turn {
                if let Ok(mut sprite) = sprites.get_mut(entity) {
                    sprite.color = Color::WHITE;
                }
                commands.entity(entity).insert(TurnControlled);
                turn_state.selected_entity = Some(entity);
                break;
            }
        }
    }
}

pub fn cycle_disk_selection(
    keys: Res<Input<KeyCode>>,
    disks: Query<(Entity, &PlayerDisk), With<RigidBody>>,
    mut sprites: Query<&mut Sprite>,
    mut turn_state: ResMut<TurnState>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::Tab) && !turn_state.in_motion {
        let mut player_disks: Vec<_> = disks
            .iter()
            .filter(|(_, d)| d.player_id == turn_state.current_turn)
            .collect();

        player_disks.sort_by_key(|(e, _)| e.index());

        if !player_disks.is_empty() {
            let current_index = turn_state.selected_entity.and_then(|current| {
                player_disks.iter().position(|(e, _)| *e == current)
            });

            if let Some(current) = turn_state.selected_entity {
                if let Ok(mut sprite) = sprites.get_mut(current) {
                    sprite.color = Color::WHITE;
                }
                commands.entity(current).remove::<TurnControlled>();
            }

            let next_index = match current_index {
                Some(i) => (i + 1) % player_disks.len(),
                None => 0,
            };

            let (new_entity, _) = player_disks[next_index];
            if let Ok(mut sprite) = sprites.get_mut(new_entity) {
                sprite.color = Color::WHITE;
            }
            commands.entity(new_entity).insert(TurnControlled);
            turn_state.selected_entity = Some(new_entity);
            turn_state.aim_direction = Vec2::ZERO;
            turn_state.power = 0.0;
        }
    }
}

pub fn check_turn_end(
    mut turn_state: ResMut<TurnState>,
    velocities: Query<&Velocity, With<RigidBody>>,
    mut commands: Commands,
    entities: Query<Entity, With<TurnControlled>>,
    mut sprites: Query<&mut Sprite>,
    disks: Query<&PlayerDisk>,
    mut powerup_control: ResMut<PowerUpControl>,
    mut event_control: ResMut<EventControl>, // ✅ Correctamente añadido
) {
    if !turn_state.in_motion {
        return;
    }

    let threshold = 0.5;
    let all_stopped = velocities.iter().all(|v| v.linvel.length_squared() < threshold);

    if all_stopped {
        turn_state.in_motion = false;

        for entity in &entities {
            if let Ok(_) = disks.get(entity) {
                if let Ok(mut sprite) = sprites.get_mut(entity) {
                    sprite.color = Color::WHITE;
                }
            }
            commands.entity(entity).remove::<TurnControlled>();
        }

        turn_state.selected_entity = None;

        if turn_state.skip_turn_switch {
            println!("⏭️ Power-Up de DOBLE TURNO: turno no cambiado");
            turn_state.skip_turn_switch = false;
        } else {
            turn_state.current_turn = turn_state.current_turn % 2 + 1;
        }

        // ✅ Incrementa solo si no hay evento activo
        if !event_control.event_active {
            event_control.turns_since_last += 1;
        }

        powerup_control.turns_since_last += 1;

        println!("🔁 Turno terminado, contador power-ups: {}", powerup_control.turns_since_last);
        println!("🎲 Turnos desde último evento: {}", event_control.turns_since_last);
    }
}





