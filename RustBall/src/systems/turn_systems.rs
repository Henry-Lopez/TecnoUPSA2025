//! src/systems/turn_system.rs
//! --------------------------------------------------------------
//! Control de turnos, selección de fichas e input de teclado
//! --------------------------------------------------------------
/// etiqueta-set para todo lo que ocurre AL FINAL de un turno
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct CheckTurnEndSet;

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;


use crate::components::*;
use crate::events::TurnFinishedEvent;
use crate::powerup::PowerUpControl;
use crate::resources::*;
use crate::snapshot::MyTurn;

/* ───────────────────────────────────────────────────────────── */
/* 1. Seleccionar automáticamente la primera ficha de tu turno   */
/* ───────────────────────────────────────────────────────────── */

pub fn auto_select_first_disk(
    mut turn_state: ResMut<TurnState>,
    disks: Query<(Entity, &OwnedBy), (Without<TurnControlled>, With<PlayerDisk>)>,
    mut commands: Commands,
    mut sprites: Query<&mut Sprite>,
    backend_info: Res<BackendInfo>,
) {
    // ‼️ Solo si todavía no hay ficha seleccionada, el turno es mío
    //    y ninguna ficha está en movimiento
    if turn_state.selected_entity.is_none()
        && !turn_state.in_motion
        && turn_state.current_turn_id == backend_info.my_uid
    {
        for (entity, owned_by) in &disks {
            if owned_by.0 == backend_info.my_uid {
                // resalta la ficha
                if let Ok(mut sprite) = sprites.get_mut(entity) {
                    sprite.color = Color::WHITE;
                }
                // le damos el componente de control
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
/* 2. Pulsar TAB para alternar ficha                             */
/* ───────────────────────────────────────────────────────────── */

pub fn cycle_disk_selection(
    keys: Res<Input<KeyCode>>,
    disks: Query<(Entity, &OwnedBy), (With<RigidBody>, With<PlayerDisk>)>,
    mut sprites: Query<&mut Sprite>,
    mut turn_state: ResMut<TurnState>,
    mut commands: Commands,
    backend_info: Res<BackendInfo>,
) {
    if !(keys.just_pressed(KeyCode::Tab) && !turn_state.in_motion) {
        return;
    }

    // Todas las fichas que me pertenecen
    let mut my_disks: Vec<Entity> = disks
        .iter()
        .filter(|(_, o)| o.0 == backend_info.my_uid)
        .map(|(e, _)| e)
        .collect();

    my_disks.sort_by_key(|e| e.index());
    if my_disks.is_empty() {
        return;
    }

    // Des-seleccionar la ficha actual
    if let Some(current) = turn_state.selected_entity {
        if let Ok(mut sprite) = sprites.get_mut(current) {
            sprite.color = Color::WHITE;
        }
        if let Some(mut ecmd) = commands.get_entity(current) {
            ecmd.remove::<TurnControlled>();
        }
    }

    // Elegir la siguiente
    let current_idx = turn_state
        .selected_entity
        .and_then(|cur| my_disks.iter().position(|&e| e == cur));
    let next_idx = current_idx.map(|i| (i + 1) % my_disks.len()).unwrap_or(0);

    let new_entity = my_disks[next_idx];
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

/* ───────────────────────────────────────────────────────────── */
/* 3. Control de dirección con flechas                           */
/* ───────────────────────────────────────────────────────────── */

pub fn aim_with_keyboard(
    keys: Res<Input<KeyCode>>,
    my_turn: Res<MyTurn>,
    mut turn_state: ResMut<TurnState>,
) {
    if !my_turn.0 {
        return;
    }

    let mut dir = turn_state.aim_direction;
    if keys.pressed(KeyCode::Left) {
        dir.x -= 0.1;
    }
    if keys.pressed(KeyCode::Right) {
        dir.x += 0.1;
    }
    if keys.pressed(KeyCode::Up) {
        dir.y += 0.1;
    }
    if keys.pressed(KeyCode::Down) {
        dir.y -= 0.1;
    }
    turn_state.aim_direction = dir.clamp_length_max(1.0);
}

/* ───────────────────────────────────────────────────────────── */
/* 4. Cargar potencia                                            */
/* ───────────────────────────────────────────────────────────── */

pub fn charge_shot_power(
    keys: Res<Input<KeyCode>>,
    my_turn: Res<MyTurn>,
    mut turn_state: ResMut<TurnState>,
) {
    if !my_turn.0 {
        return;
    }

    if keys.pressed(KeyCode::Space) {
        turn_state.power = (turn_state.power + 0.02).min(1.0);
    }
}

/* ───────────────────────────────────────────────────────────── */
/* 5. Disparar ficha seleccionada                                */
/* ───────────────────────────────────────────────────────────── */

use bevy_rapier2d::prelude::{RigidBody, Velocity, Sleeping};   // ① importa Sleeping

pub fn fire_selected_disk(
    keys: Res<Input<KeyCode>>,
    my_turn: Res<MyTurn>,
    mut turn_state: ResMut<TurnState>,
    // ② ahora pedimos también el Entity
    mut velocities: Query<(Entity, &mut Velocity), With<TurnControlled>>,
    mut commands: Commands,
) {
    if !my_turn.0 || !keys.just_released(KeyCode::Space) || turn_state.in_motion {
        return;
    }

    let dir   = turn_state.aim_direction.normalize_or_zero();
    let speed = turn_state.power * 800.0;

    let mut any_fired = false;

    // ③ iteramos con (entity, vel)
    for (entity, mut vel) in &mut velocities {
        vel.linvel = dir * speed;
        commands.entity(entity).remove::<Sleeping>();   // despierta el rigid-body
        any_fired = true;
    }

    if any_fired {
        turn_state.in_motion = true;
        turn_state.power = 0.0;
    }
}


/* ───────────────────────────────────────────────────────────── */
/* 6. Comprobar fin de turno                                     */
/* ───────────────────────────────────────────────────────────── */

// turn_systems.rs (o donde tengas check_turn_end)
// --------------------------------------------------------------
// Ahora marca MyTurn(false) en cuanto las fichas se detienen
// --------------------------------------------------------------

pub fn check_turn_end(
    mut turn_state:  ResMut<TurnState>,
    velocities:      Query<&Velocity, With<RigidBody>>,
    mut commands:    Commands,
    controlled:      Query<Entity, With<TurnControlled>>,
    mut sprites:     Query<&mut Sprite>,
    mut powerup_control: ResMut<PowerUpControl>,
    mut event_control:   ResMut<EventControl>,
    mut turn_finished:   EventWriter<TurnFinishedEvent>,
) {
    // 1) Si no estamos en movimiento, salimos
    if !turn_state.in_motion {
        return;
    }

    // 2) ¿Al menos una ficha aún se mueve?
    const THRESHOLD: f32 = 0.5;
    if velocities
        .iter()
        .any(|v| v.linvel.length_squared() >= THRESHOLD)
    {
        return;
    }

    // 3) Se detuvieron todas → termina el movimiento
    turn_state.in_motion = false;

    //    ⏸ Deshabilitar input local hasta que llegue el próximo snapshot
    commands.insert_resource(MyTurn(false));

    // 4) Quitar selección y devolver color
    for entity in &controlled {
        if let Ok(mut sprite) = sprites.get_mut(entity) {
            sprite.color = Color::WHITE;
        }
        commands.entity(entity).remove::<TurnControlled>();
    }
    turn_state.selected_entity = None;

    // 5) Contadores de power-ups / eventos
    if !event_control.event_active {
        event_control.turns_since_last += 1;
    }
    powerup_control.turns_since_last += 1;

    info!(
        "🔁 Fin de turno — PU: {}, EV: {}",
        powerup_control.turns_since_last,
        event_control.turns_since_last
    );

    // 6) Notificamos que el turno terminó.
    //    El backend enviará el próximo snapshot con el nuevo turno_actual.
    turn_finished.send(TurnFinishedEvent);
}

