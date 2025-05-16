use bevy::prelude::*;
use bevy::input::keyboard::KeyCode;
use bevy_rapier2d::prelude::{RigidBody, Velocity};

use crate::components::*;
use crate::powerup::PowerUpControl;
use crate::resources::*;

/* ───────────────────────────────────────────────────────────── */
/* Seleccionar automáticamente la primera ficha de tu turno      */
/* ───────────────────────────────────────────────────────────── */
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
                // ✔ solo insertamos si la entidad sigue viva
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
    disks: Query<(Entity, &PlayerDisk), With<RigidBody>>,
    mut sprites: Query<&mut Sprite>,
    mut turn_state: ResMut<TurnState>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::Tab) && !turn_state.in_motion {
        // 1. recopila fichas del jugador activo
        let mut player_disks: Vec<_> = disks
            .iter()
            .filter(|(_, d)| d.player_id == turn_state.current_turn)
            .collect();
        player_disks.sort_by_key(|(e, _)| e.index());

        if player_disks.is_empty() {
            return;
        }

        // 2. quita selección actual (si existe)
        if let Some(current) = turn_state.selected_entity {
            if let Ok(mut sprite) = sprites.get_mut(current) {
                sprite.color = Color::WHITE;
            }
            if let Some(mut ecmd) = commands.get_entity(current) {
                ecmd.remove::<TurnControlled>();
            }
        }

        // 3. decide el índice del siguiente disco
        let current_index = turn_state.selected_entity.and_then(|cur| {
            player_disks.iter().position(|(e, _)| *e == cur)
        });
        let next_index = current_index
            .map(|i| (i + 1) % player_disks.len())
            .unwrap_or(0);

        // 4. aplica nueva selección
        let (new_entity, _) = player_disks[next_index];
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
/* (sin cambios sustanciales; solo conserva la lógica original)  */
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
) {
    if !turn_state.in_motion {
        return;
    }

    let threshold = 0.5;
    let all_stopped = velocities
        .iter()
        .all(|v| v.linvel.length_squared() < threshold);

    if all_stopped {
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
            turn_state.current_turn = turn_state.current_turn % 2 + 1;
        }

        if !event_control.event_active {
            event_control.turns_since_last += 1;
        }
        powerup_control.turns_since_last += 1;

        println!("🔁 Fin de turno. Power-ups: {}, Eventos: {}",
                 powerup_control.turns_since_last, event_control.turns_since_last);
    }
}
