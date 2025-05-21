use bevy::prelude::*;
use bevy::input::keyboard::KeyCode;
use bevy_rapier2d::prelude::{RigidBody, Velocity};

use crate::components::*;
use crate::powerup::PowerUpControl;
use crate::resources::*;
use crate::events::TurnFinishedEvent;

/* ───────────────────────────────────────────────────────────── */
/* Seleccionar automáticamente la primera ficha de tu turno      */
/* ───────────────────────────────────────────────────────────── */
pub fn auto_select_first_disk(
    mut turn_state: ResMut<TurnState>,
    disks: Query<(Entity, &PlayerDisk, &OwnedBy), Without<TurnControlled>>,
    mut commands: Commands,
    mut sprites: Query<&mut Sprite>,
    backend_info: Res<BackendInfo>,
) {
    if turn_state.selected_entity.is_none() && !turn_state.in_motion {
        for (entity, disk, owned_by) in &disks {
            if disk.player_id == turn_state.current_turn_id && owned_by.0 == backend_info.my_uid {
                if let Ok(mut sprite) = sprites.get_mut(entity) {
                    sprite.color = Color::WHITE;
                }
                if let Some(mut ecmd) = commands.get_entity(entity) {
                    ecmd.insert(TurnControlled);
                }
                turn_state.selected_entity = Some(entity);
                break;
            }
        }
    }
}

/* ───────────────────────────────────────────────────────────── */
/* Pulsar TAB para alternar ficha                               */
/* ───────────────────────────────────────────────────────────── */
pub fn cycle_disk_selection(
    keys: Res<Input<KeyCode>>,
    disks: Query<(Entity, &PlayerDisk, &OwnedBy), With<RigidBody>>,
    mut sprites: Query<&mut Sprite>,
    mut turn_state: ResMut<TurnState>,
    mut commands: Commands,
    backend_info: Res<BackendInfo>,
) {
    if keys.just_pressed(KeyCode::Tab) && !turn_state.in_motion {
        let mut player_disks: Vec<_> = disks
            .iter()
            .filter(|(_, d, o)| d.player_id == turn_state.current_turn_id && o.0 == backend_info.my_uid)
            .map(|(e, _, _)| e)
            .collect();

        player_disks.sort_by_key(|e| e.index());

        if player_disks.is_empty() {
            return;
        }

        if let Some(current) = turn_state.selected_entity {
            if let Ok(mut sprite) = sprites.get_mut(current) {
                sprite.color = Color::WHITE;
            }
            if let Some(mut ecmd) = commands.get_entity(current) {
                ecmd.remove::<TurnControlled>();
            }
        }

        let current_index = turn_state.selected_entity.and_then(|cur| {
            player_disks.iter().position(|&e| e == cur)
        });
        let next_index = current_index.map(|i| (i + 1) % player_disks.len()).unwrap_or(0);

        let new_entity = player_disks[next_index];
        if let Ok(mut sprite) = sprites.get_mut(new_entity) {
            sprite.color = Color::WHITE;
        }
        if let Some(mut ecmd) = commands.get_entity(new_entity) {
            ecmd.insert(TurnControlled);
        }
        turn_state.selected_entity = Some(new_entity);
        turn_state.aim_direction = Vec2::ZERO;
        turn_state.power = 0.0;
    }
}

/* ───────────────────────────────────────────────────────────── */
/* Comprobar fin de turno                                       */
/* ───────────────────────────────────────────────────────────── */
pub fn check_turn_end(
    mut turn_state: ResMut<TurnState>,
    velocities: Query<&Velocity, With<RigidBody>>,
    mut commands: Commands,
    entities: Query<Entity, With<TurnControlled>>,
    mut sprites: Query<&mut Sprite>,
    disks: Query<&PlayerDisk>,
    mut powerup_control: ResMut<PowerUpControl>,
    mut event_control: ResMut<EventControl>,
    mut turn_finished: EventWriter<TurnFinishedEvent>,
    backend_info: Res<BackendInfo>,
) {
    if !turn_state.in_motion {
        return;
    }

    let threshold = 0.5;
    let all_stopped = velocities.iter().all(|v| v.linvel.length_squared() < threshold);
    if !all_stopped {
        return;
    }

    turn_state.in_motion = false;

    for entity in &entities {
        if disks.get(entity).is_ok() {
            if let Ok(mut sprite) = sprites.get_mut(entity) {
                sprite.color = Color::WHITE;
            }
        }
        if let Some(mut ecmd) = commands.get_entity(entity) {
            ecmd.remove::<TurnControlled>();
        }
    }

    turn_state.selected_entity = None;

    if turn_state.skip_turn_switch {
        println!("⏭️ Power-Up DOBLE TURNO: se mantiene el jugador.");
        turn_state.skip_turn_switch = false;
    } else {
        if turn_state.current_turn_id == backend_info.id_left {
            turn_state.current_turn_id = backend_info.id_right;
        } else {
            turn_state.current_turn_id = backend_info.id_left;
        }
    }

    if !event_control.event_active {
        event_control.turns_since_last += 1;
    }
    powerup_control.turns_since_last += 1;

    println!(
        "🔁 Fin de turno. Power-ups: {}, Eventos: {}",
        powerup_control.turns_since_last, event_control.turns_since_last
    );

    turn_finished.send(TurnFinishedEvent);
}
